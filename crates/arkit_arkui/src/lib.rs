//! Dioxus → ArkUI renderer (HostTree + Projection model).
//!
//! [`ArkUIRenderer`] implements [`dioxus_core::WriteMutations`]. It maintains a
//! renderer-owned **host tree** that mirrors the dioxus RealDOM (including text
//! nodes and placeholders), and projects that host tree onto a live ArkUI
//! native tree.
//!
//! ## Why a host tree?
//!
//! Dioxus `ElementId` identity, text nodes, placeholders, and mutation paths
//! are logical concepts that do not map 1:1 onto ArkUI native nodes:
//! - A `text { "..." }` element's child text node contributes to the parent's
//!   `TextContent` attribute — it has **0 native nodes**.
//! - A `button { "..." }` element is a semantic button backed by a stylable
//!   native container, so its child text projects as a normal native `Text`.
//! - A `Placeholder` exists only as a path anchor for `replace_placeholder`.
//! - A native child index therefore cannot equal a dioxus logical child index.
//!
//! The host tree is the single source of truth; the native tree is a
//! *projection*. Mutations update the host tree, then `sync_native` reconciles
//! the projection. Text nodes are never deleted or "swallowed" — they remain in
//! the host tree and are merged into the parent's content attribute at
//! projection time.

use std::cell::RefCell;
use std::ffi::c_void;
use std::rc::Rc;

use dioxus_core::{ElementId, Template, TemplateNode, WriteMutations};
use ohos_arkui_binding::common::error::ArkUIResult;
use ohos_arkui_binding::common::handle::ArkUIHandle;
use ohos_arkui_binding::common::node::ArkUINode;
use ohos_arkui_binding::component::attribute::{ArkUICommonAttribute, ArkUIEvent, ArkUIGesture};
use ohos_arkui_binding::component::built_in_component::{Row, Stack, Text};
use ohos_arkui_binding::component::root::RootNode;
use ohos_arkui_binding::event::inner_event::Event as ArkNativeEvent;
use ohos_arkui_binding::gesture::gesture_data::GestureEventData;
use ohos_arkui_binding::gesture::inner_gesture::Gesture;
use ohos_arkui_binding::types::attribute::ArkUINodeAttributeType;
use ohos_arkui_binding::types::event::NodeEventType;
use ohos_arkui_binding::types::gesture_event::GestureEventAction;
use rustc_hash::FxHashMap;
// Re-export the shared event-payload types (owned by `arkit_elements`, whose
// lib name is `dioxus_elements`).
use dioxus_elements::event::{classify_event_name, ArkEventKind};
pub use dioxus_elements::event::{
    ArkEventData, ArkEventPayload, LayoutPayload, PointerAction, PointerPayload,
    ScrollIndexPayload, ScrollOffsetPayload,
};

mod native;
use native::parse_color;
pub use native::{canonical_tag, create_node, create_node_by_tag, kind_from_tag, NodeKind};

pub mod image;
pub use image::{ArkImageSource, RetainedImage};

pub mod virtual_adapter;
pub use virtual_adapter::{RenderItem, VirtualKind, VirtualListAdapter, VirtualNodeAdapter};

pub mod node_builder;
pub use node_builder::NodeBuilder;

mod attributes;
use attributes::{AttrMutation, DesiredAttrs};

fn log_arkui_result<T, E: ToString>(context: &str, result: Result<T, E>) -> Option<T> {
    match result {
        Ok(value) => Some(value),
        Err(error) => {
            ohos_hilog_binding::error(format!(
                "arkit_arkui: {context} failed: {}",
                error.to_string()
            ));
            None
        }
    }
}

/// Sink for forwarding native ArkUI events back into the dioxus VirtualDom.
///
/// When `create_event_listener` registers a native event, the callback calls
/// `sink.dispatch(name, element, payload)`. The runtime implementation is
/// responsible for constructing the `Event<ArkEventData>` and calling
/// `Runtime::handle_event`, then scheduling a rerender.
pub trait EventSink {
    /// Forward a native event. `payload` carries the typed value extracted from
    /// the ArkUI `Event` (slider value, text, scroll index, ...); it is wrapped
    /// in an `ArkEventData` before re-injection.
    fn dispatch(&self, name: &'static str, element: ElementId, payload: ArkEventPayload);
}

type NodeRef = Rc<RefCell<ArkUINode>>;

const TEXT_ALIGN_START: i32 = 0;
const LONG_PRESS_DURATION_MS: i32 = 500;

struct RegisteredEventListener {
    name: &'static str,
    id: ElementId,
    native_wrapper: usize,
    active: Rc<std::cell::Cell<bool>>,
}

impl Drop for RegisteredEventListener {
    fn drop(&mut self) {
        self.active.set(false);
    }
}

struct LongPressEventContext {
    sink: Rc<dyn EventSink>,
    name: &'static str,
    id: ElementId,
}

struct GestureNode<'a>(&'a mut ArkUINode);

impl ohos_arkui_binding::component::attribute::ArkUIAttributeBasic for GestureNode<'_> {
    fn raw(&self) -> &ArkUINode {
        self.0
    }

    fn borrow_mut(&mut self) -> &mut ArkUINode {
        self.0
    }
}

impl ArkUIGesture for GestureNode<'_> {}

struct RegisteredGestureListener {
    name: &'static str,
    id: ElementId,
    node: NodeRef,
    gesture: Gesture,
    _context: Box<LongPressEventContext>,
}

impl Drop for RegisteredGestureListener {
    fn drop(&mut self) {
        let mut node = self.node.borrow_mut();
        let gesture_node = GestureNode(&mut node);
        log_arkui_result(
            "long_press remove_gesture",
            gesture_node.remove_gesture(&self.gesture),
        );
        log_arkui_result("long_press dispose_gesture", self.gesture.dispose());
    }
}

enum EventRegistration {
    Node(Rc<std::cell::Cell<bool>>),
    Gesture(RegisteredGestureListener),
}

// ---------------------------------------------------------------------------
// Host tree
// ---------------------------------------------------------------------------

/// Arena id for a host node.
type HostId = usize;

/// Logical kind of a host node — mirrors the dioxus RealDOM.
#[derive(Debug)]
enum HostKind {
    /// The synthetic root (ElementId 0) — a full-screen `Stack` container
    /// mounted to the NodeContent slot.
    Root,
    /// A dioxus element (`column`, `text`, `button`, ...). `tag` is the
    /// canonical lowercase rsx tag.
    Element { tag: &'static str },
    /// A logical text node. Projects onto the parent's content attribute when
    /// the parent is `text`/`button`; otherwise onto its own native `Text`.
    Text { value: String },
    /// A dioxus placeholder (anchor for `replace_placeholder`). Projects onto
    /// nothing natively by default.
    Placeholder,
}

/// One node in the renderer-owned host tree.
struct HostNode {
    kind: HostKind,
    parent: Option<HostId>,
    children: Vec<HostId>,
    native: Option<NodeRef>,
    native_attached: bool,
    /// Renderer-managed content container for composite native projections.
    ///
    /// Dioxus still sees this host as one logical/native root. For `button`,
    /// the native root is the stylable/clickable outer container and dioxus
    /// children are attached under this internal Row so inline icon/text layout
    /// stays native.
    content_native: Option<NodeRef>,
    event_listeners: Vec<(&'static str, ElementId)>,
    registered_event_listeners: Vec<RegisteredEventListener>,
    registered_gesture_listeners: Vec<RegisteredGestureListener>,
    /// Declarative desired attributes (dioxus state). Replayed onto the native
    /// node at lifecycle points (after create, after attach) so ArkUI's
    /// internal control skin does not clobber declarative styles.
    desired_attrs: Rc<RefCell<DesiredAttrs>>,
    /// Declarative image source carried through Dioxus `AttributeValue::Any`.
    ///
    /// Native image resources are not normal scalar attrs: applying the same
    /// `DrawableDescriptor*` on every width/height/layout replay can trip
    /// ArkUI's native image lifetime handling. The host owns the resource slot
    /// and applies it only when the source or native node changes.
    image_source: Option<ArkImageSource>,
    retained_image_src: Option<Rc<RetainedImage>>,
    /// Element ids currently bound to this arena slot. Cleared before the slot
    /// enters the free list so stale ElementId mappings cannot alias reuse.
    bound_elements: Vec<ElementId>,
}

impl HostNode {
    fn new(kind: HostKind) -> Self {
        Self {
            kind,
            parent: None,
            children: Vec::new(),
            native: None,
            native_attached: false,
            content_native: None,
            event_listeners: Vec::new(),
            registered_event_listeners: Vec::new(),
            registered_gesture_listeners: Vec::new(),
            desired_attrs: Rc::new(RefCell::new(DesiredAttrs::default())),
            image_source: None,
            retained_image_src: None,
            bound_elements: Vec::new(),
        }
    }

    fn tag(&self) -> &'static str {
        match self.kind {
            HostKind::Root => "stack",
            HostKind::Element { tag } => tag,
            HostKind::Text { .. } => "text",
            HostKind::Placeholder => "stack",
        }
    }
}

/// The attribute a host element's text children merge into, if any.
fn text_content_attr(tag: &str) -> Option<ArkUINodeAttributeType> {
    match tag {
        "text" => Some(ArkUINodeAttributeType::TextContent),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Renderer
// ---------------------------------------------------------------------------

/// A dioxus renderer that materializes mutations onto live ArkUI nodes via a
/// host-tree projection.
pub struct ArkUIRenderer {
    /// Host-node arena. Index 0 is the synthetic root.
    hosts: Vec<HostNode>,
    free_hosts: Vec<HostId>,
    /// ElementId → HostId.
    element_to_host: Vec<Option<HostId>>,
    /// The mutation stack (host ids, top = last).
    stack: Vec<HostId>,
    /// Cached static-template host subtrees, keyed by template address. Each
    /// entry is a ready-to-clone host subtree (kinds + structure) that
    /// `load_template` instantiates.
    templates: FxHashMap<usize, Vec<TemplateHostNode>>,
    /// The NodeContent slot root, owning the mounted base node.
    root_node: RootNode,
    /// Event sink (set by the runtime after construction).
    sink: Option<Rc<dyn EventSink>>,
}

/// A template's static structure, mirroring [`TemplateNode`] but in host terms.
/// `load_template` clones this and instantiates native nodes for the projected
/// children.
#[derive(Clone)]
enum TemplateHostNode {
    Element {
        tag: &'static str,
        /// Static attribute name/value pairs (applied to the native node).
        attrs: Vec<(&'static str, String)>,
        children: Vec<TemplateHostNode>,
    },
    Text {
        value: String,
    },
    Dynamic,
}

impl TemplateHostNode {
    fn from_template(def: &TemplateNode) -> Self {
        match def {
            TemplateNode::Element {
                tag,
                attrs,
                children,
                ..
            } => {
                let canonical = canonical_tag(tag);
                let mut static_attrs = Vec::new();
                for attr in attrs.iter() {
                    if let dioxus_core::TemplateAttribute::Static { name, value, .. } = attr {
                        static_attrs.push((*name, (*value).to_string()));
                    }
                }
                let children = children
                    .iter()
                    .map(TemplateHostNode::from_template)
                    .collect();
                TemplateHostNode::Element {
                    tag: canonical,
                    attrs: static_attrs,
                    children,
                }
            }
            TemplateNode::Text { text } => TemplateHostNode::Text {
                value: (*text).to_string(),
            },
            TemplateNode::Dynamic { .. } => TemplateHostNode::Dynamic,
        }
    }
}

impl ArkUIRenderer {
    /// Create a renderer mounted on the given NodeContent slot.
    ///
    /// A root `Stack` container is created and mounted to the slot; it becomes
    /// the host root (HostId 0 / ElementId 0).
    pub fn new(slot: ArkUIHandle) -> ArkUIResult<Self> {
        let mut root_node = RootNode::new(slot);
        let root_ark = NodeBuilder::from_node(Stack::new()?.into())
            .percent_width(1.0)?
            .percent_height(1.0)?
            .build();
        if let Err(error) = root_node.mount(root_ark.clone()) {
            // `RootNode::mount` retains a clone before calling native APIs.
            // Let it release that clone when possible; otherwise dispose our
            // still-unmounted owner so constructor failure cannot leak it.
            if root_node.unmount().is_err() {
                let mut root_ark = root_ark;
                let _ = root_ark.dispose();
            }
            return Err(error);
        }
        let root = Rc::new(RefCell::new(root_ark));

        let mut hosts = Vec::new();
        let mut root_host = HostNode::new(HostKind::Root);
        root_host.native = Some(root);
        root_host.native_attached = true;
        root_host.bound_elements.push(ElementId(0));
        hosts.push(root_host);

        let element_to_host = vec![Some(0)];

        Ok(Self {
            hosts,
            free_hosts: Vec::new(),
            element_to_host,
            stack: Vec::new(),
            templates: FxHashMap::default(),
            root_node,
            sink: None,
        })
    }

    /// Install the event sink used to forward native events into the VirtualDom.
    pub fn set_sink(&mut self, sink: Rc<dyn EventSink>) {
        self.sink = Some(sink);
    }

    // -- host arena helpers ------------------------------------------------

    fn alloc_host(&mut self, kind: HostKind) -> HostId {
        if let Some(id) = self.free_hosts.pop() {
            self.hosts[id] = HostNode::new(kind);
            id
        } else {
            let id = self.hosts.len();
            self.hosts.push(HostNode::new(kind));
            id
        }
    }

    fn ensure_element_capacity(&mut self, id: ElementId) {
        if id.0 >= self.element_to_host.len() {
            self.element_to_host.resize_with(id.0 + 1, || None);
        }
    }

    fn bind_element(&mut self, id: ElementId, host: HostId) {
        self.ensure_element_capacity(id);
        if let Some(previous) = self.element_to_host[id.0] {
            self.hosts[previous]
                .bound_elements
                .retain(|candidate| *candidate != id);
        }
        self.element_to_host[id.0] = Some(host);
        if !self.hosts[host].bound_elements.contains(&id) {
            self.hosts[host].bound_elements.push(id);
        }
    }

    fn release_host(&mut self, host: HostId) {
        debug_assert_ne!(host, 0, "the synthetic root cannot enter the free list");
        for element in std::mem::take(&mut self.hosts[host].bound_elements) {
            if self.element_to_host.get(element.0).and_then(|slot| *slot) == Some(host) {
                self.element_to_host[element.0] = None;
            }
        }
        self.hosts[host] = HostNode::new(HostKind::Placeholder);
        self.free_hosts.push(host);
    }

    fn host_of(&self, id: ElementId) -> HostId {
        self.element_to_host
            .get(id.0)
            .and_then(|s| *s)
            .unwrap_or_else(|| panic!("arkit_arkui: no host for ElementId({})", id.0))
    }

    // -- native projection -------------------------------------------------

    /// Create the native ArkUI projection for a host element kind
    /// (Root/Element). The returned root node is not yet attached to the tree.
    fn create_native_for(tag: &'static str) -> (NodeRef, Option<NodeRef>) {
        let node = create_node_by_tag(tag).expect("arkit_arkui: create node");
        let root = Rc::new(RefCell::new(node));
        if tag == "text" {
            Self::apply_text_defaults(&mut root.borrow_mut());
        }
        if tag == "button" {
            let content_node: ArkUINode =
                Row::new().expect("arkit_arkui: button content Row").into();
            let content = Rc::new(RefCell::new(content_node));
            root.borrow_mut()
                .insert_child(content.borrow().clone(), 0)
                .unwrap_or_else(|error| {
                    panic!("arkit_arkui: failed to create button content projection: {error}")
                });
            let mounted_content = root
                .borrow()
                .children()
                .first()
                .cloned()
                .expect("button content insertion succeeded without a mounted wrapper");
            return (root, Some(mounted_content));
        }
        (root, None)
    }

    fn apply_text_defaults(node: &mut ArkUINode) {
        let _ = node.set_attribute(ArkUINodeAttributeType::TextAlign, TEXT_ALIGN_START.into());
    }

    fn native_child_container(&self, host: HostId) -> Option<NodeRef> {
        self.hosts[host]
            .content_native
            .clone()
            .or_else(|| self.hosts[host].native.clone())
    }

    fn replay_composite_content(&self, host: HostId) {
        let tag = self.hosts[host].tag();
        let Some(content) = self.hosts[host].content_native.clone() else {
            return;
        };
        let attrs = self.hosts[host].desired_attrs.borrow();
        attrs.apply_content(&mut content.borrow_mut(), tag);
    }

    fn apply_host_image_source(&mut self, host: HostId) {
        if !self.hosts[host].native_attached {
            return;
        }
        let Some(source) = self.hosts[host].image_source.clone() else {
            return;
        };
        let Some(native) = self.hosts[host].native.clone() else {
            return;
        };

        match source.resolve() {
            Ok(retained) => {
                if let Err(error) = native
                    .borrow_mut()
                    .set_attribute(ArkUINodeAttributeType::ImageSrc, retained.drawable().into())
                {
                    ohos_hilog_binding::warn(format!(
                        "arkit_arkui: image src apply failed: {error}"
                    ));
                    return;
                }
                self.hosts[host].retained_image_src = Some(retained);
            }
            Err(error) => {
                ohos_hilog_binding::warn(format!("arkit_arkui: image src resolve failed: {error}"));
            }
        }
    }

    fn set_host_image_source(&mut self, host: HostId, source: ArkImageSource) {
        if self.hosts[host].image_source.as_ref() == Some(&source) {
            return;
        }
        self.hosts[host].image_source = Some(source);
        self.hosts[host].retained_image_src = None;
        if self.hosts[host].native_attached {
            self.apply_host_image_source(host);
        }
    }

    fn clear_host_image_source(&mut self, host: HostId) {
        self.hosts[host].image_source = None;
        self.hosts[host].retained_image_src = None;
    }

    fn rebind_composite_content(&mut self, host: HostId) {
        if self.hosts[host].tag() != "button" {
            return;
        }
        let Some(native) = self.hosts[host].native.clone() else {
            return;
        };
        self.hosts[host].content_native = native.borrow().children().first().cloned();
        self.replay_composite_content(host);
    }

    fn replay_mounted_host_state(&self, host: HostId) {
        let Some(native) = self.hosts[host].native.clone() else {
            return;
        };
        let tag = self.hosts[host].tag();
        let is_text_host = matches!(self.hosts[host].kind, HostKind::Text { .. });
        {
            let mut native = native.borrow_mut();
            if tag == "text" {
                Self::apply_text_defaults(&mut native);
            }
            let attrs = self.hosts[host].desired_attrs.borrow();
            attrs.apply_initial(&mut native, tag);
            attrs.apply_to(&mut native, tag);
            attrs.after_attach(&mut native, tag);
        }
        if is_text_host {
            self.apply_text_value(host);
        }
    }

    fn rebind_mounted_projection(&mut self, host: HostId) {
        self.rebind_composite_content(host);
        self.replay_mounted_host_state(host);
        let Some(container) = self.native_child_container(host) else {
            return;
        };
        let children = self.hosts[host].children.clone();
        let mut native_index = 0;
        for child in children {
            if self.native_roots(child).is_empty() {
                continue;
            }
            let mounted_child = container.borrow().children().get(native_index).cloned();
            if let Some(mounted_child) = mounted_child {
                self.hosts[child].native = Some(mounted_child);
                self.hosts[child].native_attached = self.hosts[host].native_attached;
                self.rebind_mounted_projection(child);
                self.apply_host_image_source(child);
            }
            native_index += 1;
        }
    }

    /// Does this host element merge its text children into a content attribute
    /// (i.e. it is `text`/`button`)? Such elements never project text children
    /// as nested native nodes.
    fn merges_text_children(tag: &'static str) -> bool {
        text_content_attr(tag).is_some()
    }

    /// The concatenated text content of a host element's logical children, or
    /// `None` if any child is not a text node (i.e. the children cannot be
    /// merged into a single content string).
    fn merged_text_of(&self, host: HostId) -> Option<String> {
        let mut out = String::new();
        for &child in self.hosts[host].children.iter() {
            match &self.hosts[child].kind {
                HostKind::Text { value } => out.push_str(value),
                _ => return None,
            }
        }
        Some(out)
    }

    /// Native nodes this host projects onto, in order. For a normal element
    /// that is just `[self.native]`; for text-under-container it is its own
    /// native text node; for text-under-text/button and placeholders it is `[]`.
    fn native_roots(&self, host: HostId) -> Vec<NodeRef> {
        let h = &self.hosts[host];
        match &h.kind {
            HostKind::Root | HostKind::Element { .. } => h.native.clone().into_iter().collect(),
            HostKind::Text { .. } => {
                // Text under text/button merges into parent → no native root.
                // Text under a normal container → its own native Text.
                match h.parent {
                    Some(parent) => {
                        let parent_tag = self.hosts[parent].tag();
                        if Self::merges_text_children(parent_tag) {
                            Vec::new()
                        } else {
                            h.native.clone().into_iter().collect()
                        }
                    }
                    None => h.native.clone().into_iter().collect(),
                }
            }
            HostKind::Placeholder => Vec::new(),
        }
    }

    fn native_raw_id(node: &NodeRef) -> usize {
        node.borrow().raw_handle() as usize
    }

    fn native_wrapper_id(node: &NodeRef) -> usize {
        Rc::as_ptr(node) as usize
    }

    fn remember_event_registration(
        &mut self,
        host: HostId,
        name: &'static str,
        id: ElementId,
        native_wrapper: usize,
        active: Rc<std::cell::Cell<bool>>,
    ) {
        self.hosts[host]
            .registered_event_listeners
            .retain(|listener| listener.name != name || listener.id != id);
        self.hosts[host]
            .registered_event_listeners
            .push(RegisteredEventListener {
                name,
                id,
                native_wrapper,
                active,
            });
    }

    fn remember_gesture_registration(
        &mut self,
        host: HostId,
        registration: RegisteredGestureListener,
    ) {
        self.hosts[host]
            .registered_gesture_listeners
            .retain(|listener| {
                listener.name != registration.name || listener.id != registration.id
            });
        self.hosts[host]
            .registered_gesture_listeners
            .push(registration);
    }

    /// Recompute the parent's content attribute from its text children, if the
    /// parent merges text. Called after text children change.
    fn sync_content_attribute(&self, parent: HostId) {
        let tag = self.hosts[parent].tag();
        let Some(attr) = text_content_attr(tag) else {
            return;
        };
        let Some(content) = self.merged_text_of(parent) else {
            return;
        };
        if let Some(native) = &self.hosts[parent].native {
            log_arkui_result(
                "sync_content_attribute set TextContent",
                native.borrow().set_attribute(attr, content.into()),
            );
        }
    }

    fn apply_button_text_inheritance(&self, parent: HostId, child: HostId) {
        if self.hosts[parent].tag() != "button" {
            return;
        }
        if !matches!(self.hosts[child].kind, HostKind::Text { .. }) {
            return;
        }
        let Some(native) = self.hosts[child].native.clone() else {
            return;
        };
        let attrs = self.hosts[parent].desired_attrs.borrow();
        attrs.apply_button_text_attrs(&mut native.borrow_mut());
    }

    fn sync_button_text_children(&self, host: HostId) {
        if self.hosts[host].tag() != "button" {
            return;
        }
        for &child in self.hosts[host].children.iter() {
            self.apply_button_text_inheritance(host, child);
        }
    }

    /// Attach a host node's native roots under its parent's native node.
    ///
    /// `native_index` is the index among the parent's *projected* native
    /// children (not the logical child index). The parent's existing projected
    /// children before/after this insertion point are unaffected because ArkUI
    /// `insert_child` is positional.
    fn attach_native(&mut self, parent: HostId, child: HostId) {
        // If the parent merges text children, a child text node contributes to
        // the parent's content attribute instead of a native child.
        let parent_tag = self.hosts[parent].tag();
        let child_is_text = matches!(self.hosts[child].kind, HostKind::Text { .. });
        if child_is_text && Self::merges_text_children(parent_tag) {
            self.sync_content_attribute(parent);
            return;
        }

        // Ensure the child has a native node if its projection requires one.
        self.ensure_native(child);
        self.apply_button_text_inheritance(parent, child);

        let Some(child_native) = self.hosts[child].native.clone() else {
            // Placeholder / non-projecting child: nothing to attach.
            return;
        };
        let Some(parent_native) = self.native_child_container(parent) else {
            return;
        };

        // Compute the native insertion index: the count of native roots
        // contributed by logical children preceding `child`.
        let native_index = self.projected_native_len_before(parent, child);
        let inserted = {
            let mut parent_mut = parent_native.borrow_mut();
            log_arkui_result(
                "attach_native insert_child",
                parent_mut.insert_child(child_native.borrow().clone(), native_index),
            )
            .is_some()
        };
        if !inserted {
            // Do not bind this logical child to whatever node happened to be
            // at the requested index when native insertion failed.
            return;
        }

        // `insert_child` consumes the child and wraps it in a *new* `Rc` inside
        // the parent's `children` — the wrapper we held (`child_native`) is NOT
        // the one now mounted. Event callbacks (`on_event`) are stored on the
        // wrapper's `event_handle` field, so we must rebind `hosts[child].native`
        // to the actually-mounted wrapper, else event registration silently
        // targets a detached wrapper and clicks never fire. (Doc §"Native
        // wrapper 处理计划".)
        let mounted = parent_native.borrow().children().get(native_index).cloned();
        if let Some(mounted) = mounted {
            self.hosts[child].native = Some(mounted);
            self.hosts[child].native_attached = self.hosts[parent].native_attached;
            self.rebind_mounted_projection(child);
        }

        // After attach, replay desired attrs so ArkUI control defaults cannot
        // clobber declarative styles.
        let child_tag = self.hosts[child].tag();
        if let Some(native) = self.hosts[child].native.clone() {
            let attrs = self.hosts[child].desired_attrs.borrow();
            attrs.after_attach(&mut native.borrow_mut(), child_tag);
        }
        self.apply_host_image_source(child);
        self.replay_composite_content(child);
        self.replay_event_listeners(child);
    }

    /// Ensure a host node has a native node allocated if its kind projects one.
    /// Text-under-container needs its own native Text; elements need their
    /// native node; placeholders/text-under-textbutton need none.
    fn ensure_native(&mut self, host: HostId) {
        if self.hosts[host].native.is_some() {
            return;
        }
        let tag = self.hosts[host].tag();
        match self.hosts[host].kind {
            HostKind::Root | HostKind::Element { .. } => {
                let (native, content_native) = Self::create_native_for(tag);
                self.hosts[host].native = Some(native);
                self.hosts[host].content_native = content_native;
            }
            HostKind::Text { .. } => {
                // Only allocate a native Text if the parent does not merge text.
                let needs_native = match self.hosts[host].parent {
                    Some(parent) => !Self::merges_text_children(self.hosts[parent].tag()),
                    None => true,
                };
                if needs_native {
                    let node: ArkUINode = Text::new().expect("arkit_arkui: Text").into();
                    let native = Rc::new(RefCell::new(node));
                    Self::apply_text_defaults(&mut native.borrow_mut());
                    self.hosts[host].native = Some(native);
                    self.apply_text_value(host);
                }
            }
            HostKind::Placeholder => {}
        }

        // After native creation, apply element defaults and replay all desired
        // attrs so nothing is lost when native was
        // created lazily after set_attribute calls.
        if let Some(native) = self.hosts[host].native.clone() {
            let attrs = self.hosts[host].desired_attrs.borrow();
            attrs.apply_initial(&mut native.borrow_mut(), tag);
        }
        if let Some(native) = self.hosts[host].native.clone() {
            let attrs = self.hosts[host].desired_attrs.borrow();
            attrs.apply_to(&mut native.borrow_mut(), tag);
        }
        self.apply_host_image_source(host);
        self.replay_composite_content(host);
        if let Some(parent) = self.hosts[host].parent {
            self.apply_button_text_inheritance(parent, host);
        }
        self.replay_event_listeners(host);
    }

    fn replay_event_listeners(&mut self, host: HostId) {
        let Some(sink) = self.sink.clone() else {
            return;
        };
        let Some(native) = self.hosts[host].native.clone() else {
            return;
        };
        let tag = self.hosts[host].tag();
        let native_wrapper = Self::native_wrapper_id(&native);
        let listeners = self.hosts[host].event_listeners.clone();
        for (name, id) in listeners {
            if self.hosts[host]
                .registered_event_listeners
                .iter()
                .any(|listener| {
                    listener.name == name
                        && listener.id == id
                        && listener.native_wrapper == native_wrapper
                })
            {
                continue;
            }
            if let Some(registration) = register_event(&native, name, tag, sink.clone(), id) {
                let active = match registration {
                    EventRegistration::Node(active) => active,
                    EventRegistration::Gesture(registration) => {
                        self.remember_gesture_registration(host, registration);
                        Rc::new(std::cell::Cell::new(true))
                    }
                };
                self.remember_event_registration(host, name, id, native_wrapper, active);
            }
        }
    }

    /// Write a text host node's value to its native TextContent (when it owns a
    /// native Text node).
    fn apply_text_value(&self, host: HostId) {
        let tag = self.hosts[host].tag();
        debug_assert_eq!(tag, "text");
        if let (Some(native), HostKind::Text { value }) =
            (&self.hosts[host].native, &self.hosts[host].kind)
        {
            log_arkui_result(
                "apply_text_value set TextContent",
                native
                    .borrow()
                    .set_attribute(ArkUINodeAttributeType::TextContent, value.clone().into()),
            );
        }
    }

    /// Number of native roots contributed by logical children of `parent` that
    /// precede `child` in the children list. This is the native insertion index
    /// for `child`.
    fn projected_native_len_before(&self, parent: HostId, child: HostId) -> usize {
        let mut count = 0;
        for &c in self.hosts[parent].children.iter() {
            if c == child {
                break;
            }
            count += self.native_roots(c).len();
        }
        count
    }

    /// Reconcile a parent's native child list from its final HostTree children.
    ///
    /// Dioxus can replace one logical node with multiple nodes in a single
    /// mutation. Applying that as a sequence of native detach/attach operations
    /// exposes intermediate child lists to ArkUI. Composite projections
    /// (`button` outer container + internal content Row) make that especially
    /// fragile, so structural mutations use this final-state sync instead.
    fn sync_native_children(&mut self, parent: HostId) {
        let parent_tag = self.hosts[parent].tag();
        if Self::merges_text_children(parent_tag) {
            self.sync_content_attribute(parent);
            return;
        }
        let Some(parent_native) = self.native_child_container(parent) else {
            return;
        };

        let children = self.hosts[parent].children.clone();
        let mut desired = Vec::<(HostId, NodeRef)>::new();
        for child in children {
            self.ensure_native(child);
            self.apply_button_text_inheritance(parent, child);

            let Some(child_native) = self.hosts[child].native.clone() else {
                continue;
            };
            desired.push((child, child_native));
        }

        let desired_raws = desired
            .iter()
            .map(|(_, native)| Self::native_raw_id(native))
            .collect::<Vec<_>>();

        let child_count = parent_native.borrow().children().len();
        for index in (0..child_count).rev() {
            let should_remove = parent_native
                .borrow()
                .children()
                .get(index)
                .map(|child| !desired_raws.contains(&Self::native_raw_id(child)))
                .unwrap_or(false);
            if should_remove
                && log_arkui_result(
                    "sync_native_children remove_child",
                    parent_native.borrow_mut().remove_child(index),
                )
                .is_none()
            {
                return;
            }
        }

        for (native_index, (child, child_native)) in desired.into_iter().enumerate() {
            let parent_attached = self.hosts[parent].native_attached;
            let desired_raw = Self::native_raw_id(&child_native);
            let mounted_at_index = parent_native.borrow().children().get(native_index).cloned();

            if mounted_at_index
                .as_ref()
                .is_some_and(|mounted| Self::native_raw_id(mounted) == desired_raw)
            {
                if let Some(mounted) = mounted_at_index {
                    self.hosts[child].native = Some(mounted);
                    self.hosts[child].native_attached = parent_attached;
                    self.rebind_mounted_projection(child);
                }
            } else {
                // The desired node may already be mounted later in the same
                // parent (a Dioxus reorder). Move that native node instead of
                // inserting a duplicate and leaving a stale tail child.
                let mounted_elsewhere = parent_native
                    .borrow()
                    .children()
                    .iter()
                    .position(|mounted| Self::native_raw_id(mounted) == desired_raw);
                let node_to_insert = if let Some(index) = mounted_elsewhere {
                    let removed = log_arkui_result(
                        "sync_native_children detach reordered child",
                        parent_native.borrow_mut().remove_child(index),
                    );
                    let Some(Some(removed)) = removed else {
                        return;
                    };
                    let node = removed.borrow().clone();
                    node
                } else {
                    child_native.borrow().clone()
                };
                let inserted = {
                    let mut parent_mut = parent_native.borrow_mut();
                    log_arkui_result(
                        "sync_native_children insert_child",
                        parent_mut.insert_child(node_to_insert, native_index),
                    )
                    .is_some()
                };
                if !inserted {
                    return;
                }
                let mounted = parent_native.borrow().children().get(native_index).cloned();
                if let Some(mounted) = mounted {
                    self.hosts[child].native = Some(mounted);
                    self.hosts[child].native_attached = parent_attached;
                    self.rebind_mounted_projection(child);
                }
            }

            self.apply_host_image_source(child);
            let child_tag = self.hosts[child].tag();
            if let Some(native) = self.hosts[child].native.clone() {
                let attrs = self.hosts[child].desired_attrs.borrow();
                attrs.after_attach(&mut native.borrow_mut(), child_tag);
            }
            self.replay_composite_content(child);
            self.replay_event_listeners(child);
        }
    }

    /// Deactivate native event callbacks and remove gesture recognizers before
    /// ArkUI disposes their native nodes.
    fn clear_subtree_native_listeners(&mut self, host: HostId) {
        let children = self.hosts[host].children.clone();
        for child in children {
            self.clear_subtree_native_listeners(child);
        }
        self.hosts[host].registered_gesture_listeners.clear();
        self.hosts[host].registered_event_listeners.clear();
    }

    /// Clear renderer-owned state for a subtree whose native root has already
    /// been disposed by an ancestor's `disposeNode`.
    fn clear_subtree_state(&mut self, host: HostId) {
        let children = self.hosts[host].children.clone();
        for c in children {
            self.clear_subtree_state(c);
        }
        self.hosts[host].native = None;
        self.hosts[host].native_attached = false;
        self.hosts[host].content_native = None;
        self.hosts[host].event_listeners.clear();
        self.hosts[host].registered_event_listeners.clear();
        self.hosts[host].registered_gesture_listeners.clear();
        self.hosts[host].children.clear();
        self.hosts[host].parent = None;
        self.clear_host_image_source(host);
        self.release_host(host);
    }

    /// Dispose a host subtree and clear renderer state.
    ///
    /// ArkUI `disposeNode` owns the native subtree. Once a host has a native
    /// root, disposing descendants separately can double free raw handles,
    /// especially for composite projections such as `button` where the
    /// renderer owns an internal content Row not represented in the HostTree.
    fn dispose_subtree(&mut self, host: HostId) {
        // Callback registrations retain renderer-owned state. Deactivate
        // node-event tokens and remove every recognizer before ArkUI disposes
        // the native subtree.
        self.clear_subtree_native_listeners(host);
        if let Some(native) = self.hosts[host].native.take() {
            log_arkui_result("dispose_subtree dispose", native.borrow_mut().dispose());
            let children = self.hosts[host].children.clone();
            for c in children {
                self.clear_subtree_state(c);
            }
        } else {
            let children = self.hosts[host].children.clone();
            for c in children {
                self.dispose_subtree(c);
            }
        }
        self.hosts[host].content_native = None;
        self.hosts[host].native_attached = false;
        self.hosts[host].event_listeners.clear();
        self.hosts[host].registered_event_listeners.clear();
        self.hosts[host].registered_gesture_listeners.clear();
        self.hosts[host].children.clear();
        self.hosts[host].parent = None;
        self.clear_host_image_source(host);
        self.release_host(host);
    }

    fn discard_detached_hosts(&mut self, hosts: impl IntoIterator<Item = HostId>) {
        for host in hosts {
            debug_assert_ne!(host, 0, "the synthetic root cannot be a detached mutation");
            self.dispose_subtree(host);
        }
    }

    // -- template instantiation -------------------------------------------

    /// Instantiate a cached template host subtree, returning the root host id.
    /// Native nodes for projected children are created and static attributes
    /// applied; the subtree's `parent`/attachment is wired by the caller.
    fn instantiate_template(&mut self, tpl: &[TemplateHostNode], root_index: usize) -> HostId {
        let root_def = &tpl[root_index];
        self.instantiate_template_node(root_def)
    }

    fn instantiate_template_node(&mut self, def: &TemplateHostNode) -> HostId {
        match def {
            TemplateHostNode::Element {
                tag,
                attrs,
                children,
            } => {
                let host = self.alloc_host(HostKind::Element { tag });
                let (native, content_native) = Self::create_native_for(tag);
                self.hosts[host].native = Some(native);
                self.hosts[host].content_native = content_native;
                // Apply element defaults before static attrs.
                if let Some(native) = self.hosts[host].native.clone() {
                    DesiredAttrs::default().apply_initial(&mut native.borrow_mut(), tag);
                }
                // Apply static attributes — both to native and desired_attrs.
                if let Some(native) = self.hosts[host].native.clone() {
                    for (name, value) in attrs {
                        let av = dioxus_core::AttributeValue::Text(value.clone());
                        let _ = self.hosts[host]
                            .desired_attrs
                            .borrow_mut()
                            .set(tag, name, &av);
                    }
                    let desired_attrs = self.hosts[host].desired_attrs.borrow();
                    desired_attrs.apply_to(&mut native.borrow_mut(), tag);
                    desired_attrs.after_patch(&mut native.borrow_mut(), tag);
                }
                self.replay_composite_content(host);
                // Instantiate static children (Dynamic children are NOT
                // instantiated here — they arrive via create_text_node/etc.
                // and replace the placeholder slot).
                for child_def in children {
                    // If this element merges text children and the child is a
                    // static text node, do not allocate a native Text for it;
                    // just record it as a logical child and sync the content
                    // attribute.
                    let child_host = self.instantiate_template_node(child_def);
                    self.hosts[child_host].parent = Some(host);
                    self.hosts[host].children.push(child_host);
                    // Attach native (handles text-merge + native index).
                    self.attach_native(host, child_host);
                }
                host
            }
            TemplateHostNode::Text { value } => {
                // Native allocation is deferred until the parent determines
                // whether this text needs its own projection.
                self.alloc_host(HostKind::Text {
                    value: value.clone(),
                })
            }
            TemplateHostNode::Dynamic => {
                // A dynamic template slot is a placeholder until
                // replace_placeholder_with_nodes fills it.
                self.alloc_host(HostKind::Placeholder)
            }
        }
    }
}

impl WriteMutations for ArkUIRenderer {
    fn append_children(&mut self, id: ElementId, m: usize) {
        let parent = self.host_of(id);
        let children: Vec<HostId> = (0..m)
            .map(|_| self.stack.pop().expect("stack underflow"))
            .collect();
        // children popped in reverse document order; reverse to restore order.
        let children: Vec<HostId> = children.into_iter().rev().collect();
        for child in children {
            self.hosts[child].parent = Some(parent);
            self.hosts[parent].children.push(child);
            self.attach_native(parent, child);
        }
    }

    fn assign_node_id(&mut self, path: &'static [u8], id: ElementId) {
        // The target is the host node at `path` under the top-of-stack host.
        let top = *self
            .stack
            .last()
            .expect("arkit_arkui: assign_node_id with empty stack");
        let target = self.walk_host_path(top, path);
        self.bind_element(id, target);
    }

    fn create_placeholder(&mut self, id: ElementId) {
        let host = self.alloc_host(HostKind::Placeholder);
        self.bind_element(id, host);
        self.stack.push(host);
    }

    fn create_text_node(&mut self, value: &str, id: ElementId) {
        let host = self.alloc_host(HostKind::Text {
            value: value.to_string(),
        });
        self.bind_element(id, host);
        self.stack.push(host);
    }

    fn load_template(&mut self, template: Template, index: usize, id: ElementId) {
        // Cache the template structure by its roots pointer identity.
        let key = template.roots.as_ptr() as usize;
        let tpl = self
            .templates
            .entry(key)
            .or_insert_with(|| {
                template
                    .roots
                    .iter()
                    .map(TemplateHostNode::from_template)
                    .collect()
            })
            .clone();
        let host = self.instantiate_template(&tpl, index);
        self.bind_element(id, host);
        self.stack.push(host);
    }

    fn replace_node_with(&mut self, id: ElementId, m: usize) {
        let target = self.host_of(id);
        if target == 0 {
            ohos_hilog_binding::warn("arkit_arkui: replace_node_with on root not supported");
            // Consume and dispose pending nodes to keep both the mutation stack
            // and host arena consistent after rejecting the operation.
            let discarded = (0..m).filter_map(|_| self.stack.pop()).collect::<Vec<_>>();
            self.discard_detached_hosts(discarded);
            return;
        }
        let new_hosts: Vec<HostId> = (0..m)
            .map(|_| self.stack.pop().expect("stack underflow"))
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        let parent = self.hosts[target].parent;
        let Some(parent) = parent else {
            ohos_hilog_binding::warn("arkit_arkui: replace_node_with: no parent");
            self.discard_detached_hosts(new_hosts);
            return;
        };
        let Some(logical_index) = self.hosts[parent]
            .children
            .iter()
            .position(|&c| c == target)
        else {
            ohos_hilog_binding::warn(
                "arkit_arkui: replace_node_with target is not a child of its recorded parent",
            );
            self.discard_detached_hosts(new_hosts);
            return;
        };

        self.hosts[parent].children.remove(logical_index);

        // Insert the new hosts at the same logical position.
        for (offset, &child) in new_hosts.iter().enumerate() {
            self.hosts[child].parent = Some(parent);
            self.hosts[parent]
                .children
                .insert(logical_index + offset, child);
        }
        self.sync_native_children(parent);
        self.dispose_subtree(target);
    }

    fn replace_placeholder_with_nodes(&mut self, path: &'static [u8], m: usize) {
        let new_hosts: Vec<HostId> = (0..m)
            .map(|_| self.stack.pop().expect("stack underflow"))
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        let top = *self
            .stack
            .last()
            .expect("arkit_arkui: replace_placeholder_with_nodes with empty stack");
        // The placeholder is at `path` under the top-of-stack host.
        let placeholder = self.walk_host_path(top, path);
        let parent = self.hosts[placeholder].parent;
        let Some(parent) = parent else {
            ohos_hilog_binding::warn("arkit_arkui: replace_placeholder: no parent");
            self.discard_detached_hosts(new_hosts);
            return;
        };
        let Some(logical_index) = self.hosts[parent]
            .children
            .iter()
            .position(|&c| c == placeholder)
        else {
            ohos_hilog_binding::warn(
                "arkit_arkui: placeholder is not a child of its recorded parent",
            );
            self.discard_detached_hosts(new_hosts);
            return;
        };

        // Clear the placeholder (no native to dispose for a bare placeholder).
        self.hosts[parent].children.remove(logical_index);
        self.dispose_subtree(placeholder);

        for (offset, &child) in new_hosts.iter().enumerate() {
            self.hosts[child].parent = Some(parent);
            self.hosts[parent]
                .children
                .insert(logical_index + offset, child);
        }
        self.sync_native_children(parent);
    }

    fn insert_nodes_after(&mut self, id: ElementId, m: usize) {
        let new_hosts: Vec<HostId> = (0..m)
            .map(|_| self.stack.pop().expect("stack underflow"))
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        let sibling = self.host_of(id);
        let Some(parent) = self.hosts[sibling].parent else {
            ohos_hilog_binding::warn("arkit_arkui: insert_nodes_after: no parent");
            self.discard_detached_hosts(new_hosts);
            return;
        };
        let Some(logical_index) = self.hosts[parent]
            .children
            .iter()
            .position(|&c| c == sibling)
        else {
            ohos_hilog_binding::warn(
                "arkit_arkui: insert_nodes_after sibling is not in its recorded parent",
            );
            self.discard_detached_hosts(new_hosts);
            return;
        };
        for (offset, &child) in new_hosts.iter().enumerate() {
            self.hosts[child].parent = Some(parent);
            self.hosts[parent]
                .children
                .insert(logical_index + 1 + offset, child);
            self.attach_native(parent, child);
        }
    }

    fn insert_nodes_before(&mut self, id: ElementId, m: usize) {
        let new_hosts: Vec<HostId> = (0..m)
            .map(|_| self.stack.pop().expect("stack underflow"))
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        let sibling = self.host_of(id);
        let Some(parent) = self.hosts[sibling].parent else {
            ohos_hilog_binding::warn("arkit_arkui: insert_nodes_before: no parent");
            self.discard_detached_hosts(new_hosts);
            return;
        };
        let Some(logical_index) = self.hosts[parent]
            .children
            .iter()
            .position(|&c| c == sibling)
        else {
            ohos_hilog_binding::warn(
                "arkit_arkui: insert_nodes_before sibling is not in its recorded parent",
            );
            self.discard_detached_hosts(new_hosts);
            return;
        };
        for (offset, &child) in new_hosts.iter().enumerate() {
            self.hosts[child].parent = Some(parent);
            self.hosts[parent]
                .children
                .insert(logical_index + offset, child);
            self.attach_native(parent, child);
        }
    }

    fn set_attribute(
        &mut self,
        name: &'static str,
        _ns: Option<&'static str>,
        value: &dioxus_core::AttributeValue,
        id: ElementId,
    ) {
        let host = self.host_of(id);
        let tag = self.hosts[host].tag();

        if name == "src" {
            match value {
                dioxus_core::AttributeValue::Any(any_val) => {
                    if let Some(source) = any_val.as_any().downcast_ref::<ArkImageSource>() {
                        self.set_host_image_source(host, source.clone());
                    }
                    return;
                }
                dioxus_core::AttributeValue::None => {
                    self.clear_host_image_source(host);
                    return;
                }
                _ => {}
            }
        }

        // Store in desired_attrs (the source of truth for replay).
        let mutation = self.hosts[host]
            .desired_attrs
            .borrow_mut()
            .set(tag, name, value);

        if let Some(native) = self.hosts[host].native.clone() {
            let desired_attrs = self.hosts[host].desired_attrs.borrow();
            if !matches!(mutation, AttrMutation::Unchanged) {
                let mut native = native.borrow_mut();
                desired_attrs.apply_mutation(&mut native, tag, name, mutation);
                desired_attrs.after_patch(&mut native, tag);
            }
        }
        if name == "src" {
            self.clear_host_image_source(host);
        }
        self.replay_composite_content(host);
        if tag == "button" {
            self.sync_button_text_children(host);
        }
    }

    fn set_node_text(&mut self, value: &str, id: ElementId) {
        let host = self.host_of(id);
        // Update the host text node's value.
        if let HostKind::Text { value: v } = &mut self.hosts[host].kind {
            *v = value.to_string();
        }
        // If this text node owns a native Text (text under a container), update
        // it directly. If it merges into a parent's content attribute, re-sync
        // the parent.
        let parent = self.hosts[host].parent;
        match parent {
            Some(parent) if Self::merges_text_children(self.hosts[parent].tag()) => {
                self.sync_content_attribute(parent);
            }
            _ => {
                if self.hosts[host].native.is_some() {
                    self.apply_text_value(host);
                }
            }
        }
    }

    fn create_event_listener(&mut self, name: &'static str, id: ElementId) {
        let Some(sink) = self.sink.clone() else {
            return;
        };
        let host = self.host_of(id);
        if !self.hosts[host]
            .event_listeners
            .iter()
            .any(|&(existing, existing_id)| existing == name && existing_id == id)
        {
            self.hosts[host].event_listeners.push((name, id));
        }
        let tag = self.hosts[host].tag();
        let Some(native) = self.hosts[host].native.clone() else {
            // Logical-only node (e.g. text child under text/button) has no
            // native target to register an event on.
            return;
        };
        let native_wrapper = Self::native_wrapper_id(&native);
        if !self.hosts[host]
            .registered_event_listeners
            .iter()
            .any(|listener| {
                listener.name == name
                    && listener.id == id
                    && listener.native_wrapper == native_wrapper
            })
        {
            if let Some(registration) = register_event(&native, name, tag, sink, id) {
                let active = match registration {
                    EventRegistration::Node(active) => active,
                    EventRegistration::Gesture(registration) => {
                        self.remember_gesture_registration(host, registration);
                        Rc::new(std::cell::Cell::new(true))
                    }
                };
                self.remember_event_registration(host, name, id, native_wrapper, active);
            }
        }
    }

    fn remove_event_listener(&mut self, name: &'static str, id: ElementId) {
        let host = self.host_of(id);
        self.hosts[host]
            .event_listeners
            .retain(|&(existing, existing_id)| existing != name || existing_id != id);
        self.hosts[host]
            .registered_gesture_listeners
            .retain(|listener| listener.name != name || listener.id != id);
        self.hosts[host]
            .registered_event_listeners
            .retain(|listener| listener.name != name || listener.id != id);
        // ArkUI event listeners are tied to node lifetime; already-registered
        // native node callbacks remain until the node is disposed. Renderer-
        // owned gesture recognizers are removed immediately above.
    }

    fn remove_node(&mut self, id: ElementId) {
        let host = self.host_of(id);
        if host == 0 {
            return;
        }
        let Some(parent) = self.hosts[host].parent else {
            return;
        };
        let Some(logical_index) = self.hosts[parent].children.iter().position(|&c| c == host)
        else {
            return;
        };
        self.hosts[parent].children.remove(logical_index);
        self.sync_native_children(parent);
        self.dispose_subtree(host);
    }

    fn push_root(&mut self, id: ElementId) {
        let host = self.host_of(id);
        self.stack.push(host);
    }
}

impl ArkUIRenderer {
    /// Replay all declarative attrs onto currently mounted native nodes.
    ///
    /// Some ArkUI controls (notably `Button`) perform native skin writes after
    /// insertion into the mounted tree. Initial mutation replay can therefore
    /// happen before the control's own first-frame defaults settle. The runtime
    /// calls this once on the next UI-loop tick after the initial mount, and
    /// composite elements get the same late replay used by normal patches.
    pub fn replay_declarative_attrs(&mut self) {
        for host in 0..self.hosts.len() {
            let tag = self.hosts[host].tag();
            let Some(native) = self.hosts[host].native.clone() else {
                continue;
            };
            let attrs = self.hosts[host].desired_attrs.borrow();
            attrs.apply_to(&mut native.borrow_mut(), tag);
            attrs.after_patch(&mut native.borrow_mut(), tag);
            drop(attrs);
            self.apply_host_image_source(host);
            self.replay_composite_content(host);
        }
    }

    /// Unmount the root from the NodeContent slot.
    pub fn unmount(&mut self) -> ArkUIResult<()> {
        // Gesture Drop calls into the mounted node, and node-event callbacks
        // rely on active tokens. Tear both down before RootNode destroys the
        // native subtree.
        self.clear_subtree_native_listeners(0);
        self.root_node.unmount()
    }

    /// Look up the native ArkUI node backing the given dioxus [`ElementId`], if
    /// one is currently mounted.
    ///
    /// Returns the host's native projection as a shared `Rc` handle — the
    /// **same** `Rc` that is mounted in the ArkUI tree (the one stored as the
    /// node's user-data for event dispatch). This matters: ArkUI event
    /// dispatch reads the callback from this exact wrapper's `event_handle`, so
    /// event registration (e.g. `onAreaChange`) must target this `Rc`, not a
    /// cloned `ArkUINode` value (whose `event_handle` is a separate copy).
    ///
    /// For logical-only nodes (text children under `text`/`button`,
    /// placeholders) returns `None` so layout hooks are not misled.
    pub fn node_for_element(&self, id: ElementId) -> Option<Rc<RefCell<ArkUINode>>> {
        let host = self.element_to_host.get(id.0).and_then(|s| *s)?;
        self.hosts[host].native.clone()
    }

    /// Walk the host tree from `root` following `path` (logical child indices),
    /// returning the target host id.
    fn walk_host_path(&self, root: HostId, path: &[u8]) -> HostId {
        let mut current = root;
        for &idx in path {
            current = *self.hosts[current]
                .children
                .get(idx as usize)
                .expect("arkit_arkui: host path index out of range");
        }
        current
    }
}

impl Drop for ArkUIRenderer {
    fn drop(&mut self) {
        // Explicit `unmount` remains the error-reporting API; Drop provides
        // the same ordering and best-effort cleanup for early-return paths.
        let _ = self.unmount();
    }
}

// ---------------------------------------------------------------------------
// Event registration + payload extraction
// ---------------------------------------------------------------------------

/// Register a native ArkUI event or gesture that forwards into the dioxus
/// runtime.
///
/// `ArkUINode` implements `ArkUICommonAttribute` directly, but `ArkUIEvent` is
/// only implemented for wrapper types (the binding's RuntimeNode). We use a
/// local wrapper to opt into the trait's default event-registration methods.
fn register_event(
    node: &NodeRef,
    name: &'static str,
    tag: &'static str,
    sink: Rc<dyn EventSink>,
    id: ElementId,
) -> Option<EventRegistration> {
    if classify_event_name(name) == Some(ArkEventKind::LongPress) {
        return log_arkui_result(
            "create_event_listener long_press",
            register_long_press(node, name, sink, id),
        )
        .map(EventRegistration::Gesture);
    }

    struct EventNode<'a>(&'a mut ArkUINode);
    impl ohos_arkui_binding::component::attribute::ArkUIAttributeBasic for EventNode<'_> {
        fn raw(&self) -> &ArkUINode {
            self.0
        }
        fn borrow_mut(&mut self) -> &mut ArkUINode {
            self.0
        }
    }
    impl ohos_arkui_binding::component::attribute::ArkUIEvent for EventNode<'_> {}

    let Some(event_type) = event_type_for_name(name, tag) else {
        ohos_hilog_binding::warn(format!(
            "arkit_arkui: create_event_listener: unsupported event `{name}` on `{tag}`"
        ));
        return None;
    };

    let mut borrowed = node.borrow_mut();
    let mut event_node = EventNode(&mut borrowed);
    if event_type == NodeEventType::OnClick {
        event_node.on_event(NodeEventType::OnClickEvent, |_| {});
    }
    let node_for_payload = node.clone();
    let active = Rc::new(std::cell::Cell::new(true));
    let callback_active = active.clone();
    event_node.on_event(event_type, move |event: &ArkNativeEvent| {
        if !callback_active.get() {
            return;
        }
        let payload = extract_payload(event_type, event, Some(&node_for_payload));
        sink.dispatch(name, id, payload);
    });
    Some(EventRegistration::Node(active))
}

fn register_long_press(
    node: &NodeRef,
    name: &'static str,
    sink: Rc<dyn EventSink>,
    id: ElementId,
) -> ArkUIResult<RegisteredGestureListener> {
    let gesture = Gesture::create_long_gesture(1, false, LONG_PRESS_DURATION_MS)?;
    let mut context = Box::new(LongPressEventContext { sink, name, id });
    let context_ptr = std::ptr::from_mut(context.as_mut()).cast::<c_void>();

    if let Err(error) =
        gesture.on_gesture_with_data(GestureEventAction::Accept, context_ptr, dispatch_long_press)
    {
        let _ = gesture.dispose();
        return Err(error);
    }
    let add_result = {
        let mut node = node.borrow_mut();
        GestureNode(&mut node).add_gesture_ref(&gesture, None, None)
    };
    if let Err(error) = add_result {
        let _ = gesture.dispose();
        return Err(error);
    }

    Ok(RegisteredGestureListener {
        name,
        id,
        node: node.clone(),
        gesture,
        _context: context,
    })
}

fn dispatch_long_press(event: GestureEventData) {
    if event.event_action_type != GestureEventAction::Accept {
        return;
    }
    let Some(context) = event.data else {
        return;
    };
    // SAFETY: `context` points to the boxed context retained by the matching
    // `RegisteredGestureListener`. Its Drop implementation removes and
    // disposes the native recognizer before the box is released.
    let context = unsafe { &*context.cast::<LongPressEventContext>() };
    context
        .sink
        .dispatch(context.name, context.id, ArkEventPayload::None);
}

/// Extract a typed [`ArkEventPayload`] from a native ArkUI [`ArkNativeEvent`].
///
/// The value accessor + index differ per event type. This mirrors the legacy
/// extraction logic (each ArkUI event stores its value(s) at a known numeric
/// index accessible via `i32_value`/`f32_value`/`string_value`/`async_string`).
fn extract_payload(
    event_type: NodeEventType,
    event: &ArkNativeEvent,
    node: Option<&NodeRef>,
) -> ArkEventPayload {
    use NodeEventType::*;
    match event_type {
        OnFocus => ArkEventPayload::Bool(true),
        OnBlur => ArkEventPayload::Bool(false),
        // Checkbox / radio checked state: i32(0) != 0.
        CheckboxEventOnChange | RadioEventOnChange | ToggleOnChange => {
            ArkEventPayload::Bool(event.i32_value(0).unwrap_or(0) != 0)
        }
        // Slider value: f32(0).
        SliderEventOnChange => ArkEventPayload::Float(event.f32_value(0).unwrap_or(0.0)),
        // Text input/area change: the new text via async_string.
        TextInputOnChange | TextAreaOnChange => {
            ArkEventPayload::String(event.async_string().unwrap_or_default())
        }
        // Date/calendar picker change: the selected date string.
        DatePickerEventOnDateChange | CalendarPickerEventOnChange => {
            ArkEventPayload::String(event.string_value(0).unwrap_or_default())
        }
        // Submit: return code i32(0).
        TextInputOnSubmit | TextAreaOnSubmit => {
            ArkEventPayload::Int(event.i32_value(0).unwrap_or(0))
        }
        // List scroll index: first/last/center at i32(0/1/2).
        ListOnScrollIndex => ArkEventPayload::ScrollIndex(ScrollIndexPayload {
            first: event.i32_value(0).unwrap_or(0),
            last: event.i32_value(1).unwrap_or(0),
            center: event.i32_value(2).unwrap_or(0),
        }),
        // Water-flow scroll index: start/end at i32(0/1).
        WaterFlowOnScrollIndex => ArkEventPayload::ScrollIndex(ScrollIndexPayload {
            first: event.i32_value(0).unwrap_or(0),
            last: event.i32_value(1).unwrap_or(0),
            center: 0,
        }),
        ScrollEventOnScroll => ArkEventPayload::ScrollOffset(ScrollOffsetPayload {
            x: event.f32_value(0).unwrap_or_default(),
            y: event.f32_value(1).unwrap_or_default(),
        }),
        EventOnAreaChange => node
            .and_then(extract_layout_payload)
            .map(ArkEventPayload::Layout)
            .unwrap_or_default(),
        // Swiper change: new index i32(0).
        SwiperEventOnChange | SwiperEventOnAnimationEnd => {
            ArkEventPayload::Int(event.i32_value(0).unwrap_or(0))
        }
        // Hover: i32(0) is the is-hovering boolean (1 = entered, 0 = exited).
        OnHover => ArkEventPayload::Bool(event.i32_value(0).unwrap_or(0) != 0),
        OnClick | OnClickEvent | TouchEvent | OnHoverMove | OnDragStart | OnDragMove
        | OnDragEnd | OnDragEnter | OnDragLeave => extract_pointer_payload(event)
            .map(ArkEventPayload::Pointer)
            .unwrap_or_default(),
        _ => ArkEventPayload::None,
    }
}

fn extract_pointer_payload(event: &ArkNativeEvent) -> Option<PointerPayload> {
    use ohos_arkui_binding::arkui_input_binding::UIInputAction;

    let input = event.input_event()?;
    let pointer_id = input.get_changed_pointer_id().map_or_else(
        |_| input.pointer_id(0),
        |id| i32::try_from(id).unwrap_or_default(),
    );
    let mut pressed_buttons = [0_i32; 8];
    let pressed_count = input
        .mouse_pressed_buttons(&mut pressed_buttons)
        .unwrap_or_default();
    let buttons = pressed_buttons[..pressed_count]
        .iter()
        .filter_map(|button| u32::try_from(*button).ok())
        .filter(|button| *button < 64)
        .fold(0_u64, |mask, button| mask | (1_u64 << button));
    Some(PointerPayload {
        action: match input.action {
            UIInputAction::Cancel => PointerAction::Cancel,
            UIInputAction::Down => PointerAction::Down,
            UIInputAction::Move => PointerAction::Move,
            UIInputAction::Up => PointerAction::Up,
        },
        timestamp_nanos: u64::try_from(input.event_time()).unwrap_or_default(),
        pointer_id,
        buttons,
        pressure: input.pointer_pressure(0),
        x: input.pointer_x(),
        y: input.pointer_y(),
        window_x: input.pointer_window_x(),
        window_y: input.pointer_window_y(),
        target_x: input.event_target_global_position_x(),
        target_y: input.event_target_global_position_y(),
        target_width: input.event_target_width(),
        target_height: input.event_target_height(),
    })
}

fn extract_layout_payload(node: &NodeRef) -> Option<LayoutPayload> {
    let n = node.borrow();
    let size = n.layout_size().ok()?;
    let position = n
        .position_with_translate_in_window()
        .or_else(|_| n.layout_position_in_window())
        .ok()?;
    Some(LayoutPayload {
        x: position.x as f32,
        y: position.y as f32,
        width: size.width as f32,
        height: size.height as f32,
    })
}

/// Map an rsx event name (+ the component tag, for kind-specific events) to the
/// ArkUI [`NodeEventType`] that fires it.
///
/// `name` is matched in its **normalized** (dioxus-stripped) form — e.g.
/// `"click"`, `"change"`, `"submit"` — because dioxus core strips the `"on"`
/// prefix before calling `create_event_listener`. Full `on_*` forms are also
/// accepted for robustness.
fn event_type_for_name(name: &str, tag: &str) -> Option<NodeEventType> {
    use NodeEventType::*;
    let kind = classify_event_name(name)?;
    Some(match (kind, tag) {
        (ArkEventKind::Click, _) => OnClick,

        // Value change — component-specific.
        (ArkEventKind::Change, "checkbox") => CheckboxEventOnChange,
        (ArkEventKind::Change, "toggle") => ToggleOnChange,
        (ArkEventKind::Change, "radio") => RadioEventOnChange,
        (ArkEventKind::Change, "slider") => SliderEventOnChange,
        (ArkEventKind::Change, "textinput") => TextInputOnChange,
        (ArkEventKind::Change, "textarea") => TextAreaOnChange,
        (ArkEventKind::Change, "datepicker") => DatePickerEventOnDateChange,
        (ArkEventKind::Change, "calendar" | "calendarpicker") => CalendarPickerEventOnChange,

        // Submit — text input/area.
        (ArkEventKind::Submit, "textinput") => TextInputOnSubmit,
        (ArkEventKind::Submit, "textarea") => TextAreaOnSubmit,

        // Element-bound layout/area changes.
        (ArkEventKind::AreaChange, _) => EventOnAreaChange,

        (ArkEventKind::Focus, _) => OnFocus,
        (ArkEventKind::Blur, _) => OnBlur,

        // Grid scroll-index events were added after the workspace's API-20
        // contract. Do not register the unrelated WaterFlow `OnWillScroll`
        // event on a Grid: it succeeds inconsistently and carries a different
        // payload shape.
        (ArkEventKind::Scroll, "list") => ListOnScrollIndex,
        (ArkEventKind::Scroll, "waterflow") => WaterFlowOnScrollIndex,
        (ArkEventKind::Scroll, "scroll") => ScrollEventOnScroll,

        // Swiper change. Animation-end is used as the stable selection
        // boundary: unlike the early change callback, it fires after the
        // native viewport has committed its new index.
        (ArkEventKind::SwiperChange, "swiper") => SwiperEventOnAnimationEnd,

        // Refresh trigger.
        (ArkEventKind::Refresh, "refresh") => RefreshOnRefresh,

        (ArkEventKind::Hover, _) => OnHover,
        (ArkEventKind::HoverMove, _) => OnHoverMove,

        // Drag lifecycle (generic across components).
        (ArkEventKind::DragStart, _) => OnDragStart,
        (ArkEventKind::DragMove, _) => OnDragMove,
        (ArkEventKind::DragEnd, _) => OnDragEnd,
        (ArkEventKind::DragLeave, _) => OnDragLeave,
        (ArkEventKind::DragEnter, _) => OnDragEnter,

        // Raw touch (generic across components).
        (ArkEventKind::Touch, _) => TouchEvent,

        _ => return None,
    })
}

#[cfg(test)]
mod event_tests {
    use super::{event_type_for_name, NodeEventType};

    #[test]
    fn component_events_use_their_typed_native_event() {
        assert_eq!(
            event_type_for_name("change", "toggle"),
            Some(NodeEventType::ToggleOnChange)
        );
        assert_eq!(
            event_type_for_name("_hover_move", "column"),
            Some(NodeEventType::OnHoverMove)
        );
        assert_eq!(event_type_for_name("scroll", "grid"), None);
        assert_eq!(
            event_type_for_name("_swiper_change", "swiper"),
            Some(NodeEventType::SwiperEventOnAnimationEnd)
        );
        assert_eq!(
            event_type_for_name("focus", "textinput"),
            Some(NodeEventType::OnFocus)
        );
        assert_eq!(
            event_type_for_name("_blur", "textinput"),
            Some(NodeEventType::OnBlur)
        );
    }
}
