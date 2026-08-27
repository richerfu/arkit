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

use arkit_dom::{ElementKey, HostId, HostKind, HostTree, PortalLayer};
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
use ohos_arkui_binding::types::gesture_event::GestureEventAction;
use ohos_arkui_sys::ArkUI_NodeComponentEvent;
use rustc_hash::{FxHashMap, FxHashSet};
// Re-export the shared event-payload types (owned by `arkit_elements`, whose
// lib name is `dioxus_elements`).
use dioxus_elements::event::{classify_event_name, ArkEventKind};
pub use dioxus_elements::event::{
    ArkEventData, ArkEventPayload, LayoutPayload, PointerAction, PointerPayload,
    ScrollIndexPayload, ScrollOffsetPayload,
};

mod css_value;
mod element_ref;
mod native;
use element_ref::SharedNativeNode;
pub use element_ref::{
    LayoutFramePx, MountedNodeLease, NativeElementDelivery, NativeElementEvent, NativeElementRef,
    NativeElementSubscription, NativeVisibility,
};
use native::{canonical_tag, create_node_by_tag, parse_color};

mod owned_node;
pub use owned_node::OwnedNativeNode;

pub mod image;
pub use image::{ArkImagePixels, ArkImageSource, RetainedImage};

pub mod virtual_adapter;
pub use virtual_adapter::{MountItem, RenderItem, VirtualItemMount, VirtualKind, VirtualSource};

pub mod node_builder;
pub use node_builder::{NativeNodeEvent, NodeBuilder, NodeEventType, PreDragStatus};

mod attributes;
use attributes::{AttrMutation, DesiredAttrs, ScrollOffsetCommand};

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

    /// Queue an exact-element notification for delivery outside the native
    /// mutation phase.
    fn dispatch_native_ref(&self, delivery: NativeElementDelivery);
}

type NodeRef = SharedNativeNode;

const TEXT_ALIGN_START: i32 = 0;
const LONG_PRESS_DURATION_MS: i32 = 500;

struct RegisteredEventListener {
    event_type: NodeEventType,
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

/// Mutable node adapter for `ArkUIEvent` registrations.
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

struct NodeEventRoute {
    node: std::rc::Weak<RefCell<ArkUINode>>,
    sink: Rc<dyn EventSink>,
    listeners: Vec<(&'static str, ElementId)>,
    native_ref: Option<(NativeElementRef, u64)>,
}

struct RoutedNodeEvent {
    event_type: NodeEventType,
    route: Rc<RefCell<NodeEventRoute>>,
}

#[derive(Default)]
struct NativeHostState {
    native: Option<NodeRef>,
    native_attached: bool,
    /// One-shot `EventOnAppear` declarative replay armed for this host (see
    /// [`ArkUIRenderer::arm_appear_replay`]). Cleared when the replay fires or
    /// the host is released.
    appear_replay_armed: bool,
    /// Renderer-managed content container for composite native projections.
    ///
    /// Dioxus still sees this host as one logical/native root. For `button`,
    /// the native root is the stylable/clickable outer container and dioxus
    /// children are attached under this internal Row so inline icon/text layout
    /// stays native.
    content_native: Option<NodeRef>,
    event_listeners: Vec<(&'static str, ElementId)>,
    registered_event_listeners: Vec<RegisteredEventListener>,
    routed_node_events: Vec<RoutedNodeEvent>,
    registered_gesture_listeners: Vec<RegisteredGestureListener>,
    /// Declarative desired attributes (dioxus state). Replayed onto the native
    /// node at lifecycle points (after create, after attach) so ArkUI's
    /// internal control skin does not clobber declarative styles.
    desired_attrs: Rc<RefCell<DesiredAttrs>>,
    /// Pending one-shot scroll operation. Unlike `desired_attrs`, this is
    /// consumed after native attachment and is never replayed.
    pending_scroll_offset: Option<ScrollOffsetCommand>,
    /// Declarative image source carried through Dioxus `AttributeValue::Any`.
    ///
    /// Native image resources are not normal scalar attrs: applying the same
    /// `DrawableDescriptor*` on every width/height/layout replay can trip
    /// ArkUI's native image lifetime handling. The host owns the resource slot
    /// and applies it only when the source or native node changes.
    image_source: Option<ArkImageSource>,
    retained_image_src: Option<Rc<RetainedImage>>,
    native_ref: Option<NativeElementRef>,
    virtual_source: Option<VirtualSource>,
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
    hosts: HostTree<NativeHostState>,
    /// Cached static-template host subtrees, keyed by template address. Each
    /// entry is a ready-to-clone host subtree (kinds + structure) that
    /// `load_template` instantiates.
    templates: FxHashMap<usize, Vec<TemplateHostNode>>,
    /// Ownership boundary for the synthetic host root.
    root_mount: RendererRootMount,
    /// Event sink (set by the runtime after construction).
    sink: Option<Rc<dyn EventSink>>,
    /// Incremental native-projection state shared across one Dioxus mutation
    /// batch. Local host mutations never invalidate the renderer root directly;
    /// only active portal membership/order changes do.
    projection: ProjectionState,
    /// Detached subtrees are disposed only after Dioxus finishes the current
    /// mutation batch. ArkUI can retain transient references during a native
    /// child-list patch, so destroying a removed `FrameNode` in the middle of
    /// reconciliation is unsafe.
    pending_subtree_disposals: Vec<HostId>,
    /// Detached native subtrees isolated from the mounted projection.
    ///
    /// Some platform builds keep `FrameNode` and layout references beyond the
    /// removal callback, while `disposeNode` invalidates the opaque handle
    /// immediately. Keeping the detached root alive avoids a later-vsync use
    /// after free. High-frequency UI branches should remain mounted and switch
    /// visibility so this exceptional retirement list stays small.
    retired_native_subtrees: Vec<NodeRef>,
    /// First structural native failure. Once set, projection is no longer
    /// trustworthy and the owning runtime must stop after the mutation batch.
    fault: Option<RendererFault>,
    /// Native subtree destroyed outside this renderer: teardown must release
    /// only Rust state (see [`Self::make_inert`]).
    inert: bool,
    /// One-shot appear-replay dispatcher installed by the runtime. It is
    /// invoked from the native `EventOnAppear` callback so declarative attrs
    /// are reapplied after ArkUI control skins settle, before first paint.
    appear_replay_handler: Option<Rc<dyn Fn(ElementId)>>,
}

#[derive(Default)]
struct DirtyHostQueue {
    ordered: Vec<HostId>,
    members: FxHashSet<HostId>,
}

impl DirtyHostQueue {
    fn mark(&mut self, host: HostId) {
        if self.members.insert(host) {
            self.ordered.push(host);
        }
    }

    fn discard(&mut self, host: HostId) {
        if self.members.remove(&host) {
            self.ordered.retain(|candidate| *candidate != host);
        }
    }

    fn drain(&mut self) -> Vec<HostId> {
        self.members.clear();
        std::mem::take(&mut self.ordered)
    }
}

#[derive(Default)]
struct ProjectionState {
    /// Monotonic activation order for every portal currently projected at the
    /// root. Host ids are arena slots and can be reused after removal, so they
    /// cannot define stable same-layer z-order.
    active_portals: FxHashMap<HostId, u64>,
    next_portal_order: u64,
    root_dirty: bool,
    deferred_event_hosts: DirtyHostQueue,
}

impl ProjectionState {
    fn activate_portal(&mut self, host: HostId) -> bool {
        if self.active_portals.contains_key(&host) {
            return false;
        }
        self.next_portal_order = self
            .next_portal_order
            .checked_add(1)
            .expect("arkit_arkui: portal activation order space exhausted");
        self.active_portals.insert(host, self.next_portal_order);
        self.root_dirty = true;
        true
    }

    fn deactivate_portal(&mut self, host: HostId) -> bool {
        let changed = self.active_portals.remove(&host).is_some();
        self.root_dirty |= changed;
        changed
    }

    fn mark_portal_order_dirty(&mut self, host: HostId) {
        self.root_dirty |= self.active_portals.contains_key(&host);
    }

    fn take_root_dirty(&mut self) -> bool {
        std::mem::take(&mut self.root_dirty)
    }

    fn discard_host(&mut self, host: HostId) {
        self.deferred_event_hosts.discard(host);
        self.deactivate_portal(host);
    }
}

/// Irrecoverable failure while projecting the logical host tree to ArkUI.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RendererFault {
    operation: &'static str,
    message: String,
}

impl std::fmt::Display for RendererFault {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{} failed: {}", self.operation, self.message)
    }
}

impl std::error::Error for RendererFault {}

fn latch_renderer_fault(
    fault: &mut Option<RendererFault>,
    operation: &'static str,
    message: String,
) {
    if fault.is_none() {
        *fault = Some(RendererFault { operation, message });
    }
}

enum RendererRootMount {
    /// Normal application renderer: the root is mounted in and owned through a
    /// NodeContent slot.
    NodeContent(RootNode),
    /// Embedded renderer: the caller owns the supplied native root and keeps it
    /// alive until after this renderer is dropped.
    Embedded,
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
        let root_ark = NodeBuilder::from_raw(Stack::new()?.into())
            .percent_width(1.0)?
            .percent_height(1.0)?
            .build();
        if let Err(error) = root_node.mount(root_ark.as_raw().clone()) {
            // `RootNode::mount` retains a clone before calling native APIs.
            // If rollback succeeds it has already disposed the shared native
            // handle, so relinquish our wrapper without disposing it again.
            // If rollback fails, our unique owner remains responsible.
            if root_node.unmount().is_ok() {
                drop(root_ark.into_raw());
            } else {
                drop(root_ark);
            }
            return Err(error);
        }
        let root_ark = root_ark.into_raw();
        let root = Rc::new(RefCell::new(root_ark));
        Ok(Self::from_root(
            root,
            RendererRootMount::NodeContent(root_node),
        ))
    }

    /// Create a renderer that projects a Dioxus subtree directly into an
    /// existing native root.
    ///
    /// The caller owns `root` and must keep it alive until this renderer is
    /// dropped. Embedded renderers do not dispose or detach the root; they only
    /// tear down renderer-owned listeners. This is the mount boundary used by
    /// RSX-backed NodeAdapter items, whose ListItem/GridItem/FlowItem wrapper
    /// remains owned by ArkUI's adapter.
    pub fn new_embedded(root: Rc<RefCell<ArkUINode>>) -> Self {
        Self::from_root(root, RendererRootMount::Embedded)
    }

    fn from_root(root: Rc<RefCell<ArkUINode>>, root_mount: RendererRootMount) -> Self {
        let mut hosts = HostTree::new(NativeHostState::default());
        let root_host = hosts.root();
        hosts[root_host].native = Some(root);
        hosts[root_host].native_attached = true;

        Self {
            hosts,
            templates: FxHashMap::default(),
            root_mount,
            sink: None,
            projection: ProjectionState::default(),
            pending_subtree_disposals: Vec::new(),
            retired_native_subtrees: Vec::new(),
            fault: None,
            inert: false,
            appear_replay_handler: None,
        }
    }

    /// Install the event sink used to forward native events into the VirtualDom.
    pub fn set_sink(&mut self, sink: Rc<dyn EventSink>) {
        self.sink = Some(sink);
    }

    /// Switch this renderer into inert mode: the native subtree it projected
    /// onto has been destroyed outside the renderer. All further teardown
    /// (Drop, unmount) releases only Rust-side state and never calls native
    /// APIs on the dead handles.
    pub fn make_inert(&mut self) {
        if self.inert {
            return;
        }
        self.inert = true;
        if let RendererRootMount::NodeContent(root) =
            std::mem::replace(&mut self.root_mount, RendererRootMount::Embedded)
        {
            root.into_inert();
        }
    }

    /// Take the first structural projection failure, if any.
    ///
    /// Dioxus' mutation writer cannot return a `Result`, so the renderer
    /// latches the failure and the owning runtime checks it immediately after
    /// each mutation batch.
    pub fn take_fault(&mut self) -> Option<RendererFault> {
        self.fault.take()
    }

    fn latch_structural<T, E: ToString>(
        &mut self,
        operation: &'static str,
        result: Result<T, E>,
    ) -> Option<T> {
        match result {
            Ok(value) => Some(value),
            Err(error) => {
                let message = error.to_string();
                latch_renderer_fault(&mut self.fault, operation, message.clone());
                ohos_hilog_binding::error(format!("arkit_arkui: {operation} failed: {message}"));
                None
            }
        }
    }

    // -- host arena helpers ------------------------------------------------

    fn alloc_host(&mut self, kind: HostKind) -> HostId {
        self.hosts.alloc(kind)
    }

    fn bind_element(&mut self, id: ElementId, host: HostId) {
        self.hosts.bind_element(ElementKey::new(id.0), host);
    }

    fn release_host(&mut self, host: HostId) {
        self.projection.discard_host(host);
        self.hosts.release(host);
    }

    fn host_of(&self, id: ElementId) -> HostId {
        self.hosts
            .host_for_element(ElementKey::new(id.0))
            .unwrap_or_else(|| panic!("arkit_arkui: no host for ElementId({})", id.0))
    }

    fn activate_portals_in_subtree(&mut self, host: HostId) {
        if !self.hosts.is_connected_to_root(host) {
            return;
        }
        self.activate_portals_in_connected_subtree(host);
    }

    fn activate_portals_in_connected_subtree(&mut self, host: HostId) {
        if matches!(self.hosts[host].kind, HostKind::Portal { .. }) {
            self.projection.activate_portal(host);
        }
        let children = self.hosts[host].children.clone();
        for child in children {
            self.activate_portals_in_connected_subtree(child);
        }
    }

    fn deactivate_portals_in_subtree(&mut self, host: HostId) -> bool {
        let mut changed = false;
        if matches!(self.hosts[host].kind, HostKind::Portal { .. }) {
            changed |= self.projection.deactivate_portal(host);
        }
        let children = self.hosts[host].children.clone();
        for child in children {
            changed |= self.deactivate_portals_in_subtree(child);
        }
        changed
    }

    fn flush_root_projection(&mut self) {
        if !self.projection.take_root_dirty() {
            return;
        }
        self.sync_native_children(self.hosts.root());
    }

    fn mark_deferred_node_events(&mut self, host: HostId) {
        self.projection.deferred_event_hosts.mark(host);
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
                .insert_child(content.clone(), 0)
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

    fn apply_pending_scroll_offset(&mut self, host: HostId) {
        if !self.hosts[host].native_attached {
            return;
        }
        let Some(command) = self.hosts[host].pending_scroll_offset.take() else {
            return;
        };
        let Some(native) = self.hosts[host].native.clone() else {
            return;
        };
        let _ = command.apply(&mut native.borrow_mut());
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
    }

    fn replay_after_attach(&self, host: HostId) {
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
            attrs.after_attach(&mut native, tag);
        }
        if is_text_host {
            self.apply_text_value(host);
        }
    }

    /// Rebind renderer-owned `Rc` wrappers after ArkUI inserts or moves a
    /// native subtree. Wrapper rebinding is ownership bookkeeping; declarative
    /// attributes are replayed only when the subtree was physically attached.
    fn rebind_mounted_projection(&mut self, host: HostId, replay_after_attach: bool) {
        let replay_after_attach = replay_after_attach && self.hosts[host].native_attached;
        self.rebind_composite_content(host);
        if replay_after_attach {
            self.replay_after_attach(host);
            self.replay_composite_content(host);
            self.apply_host_image_source(host);
            // Arm the one-shot EventOnAppear replay so declarative styles are
            // reapplied after ArkUI control skins settle (single-frame
            // convergence instead of a default-skin first frame).
            self.arm_appear_replay(host);
        }
        self.bind_native_ref(host);
        self.attach_virtual_source(host);
        self.replay_event_listeners(host);
        if replay_after_attach {
            self.apply_pending_scroll_offset(host);
        }
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
                let wrapper_changed = self.hosts[child]
                    .native
                    .as_ref()
                    .is_none_or(|current| !Rc::ptr_eq(current, &mounted_child));
                let was_attached = self.hosts[child].native_attached;
                let is_attached = self.hosts[host].native_attached;
                self.hosts[child].native = Some(mounted_child);
                self.hosts[child].native_attached = is_attached;
                let child_attach_replay = replay_after_attach || (!was_attached && is_attached);
                if wrapper_changed || was_attached != is_attached || child_attach_replay {
                    self.rebind_mounted_projection(child, child_attach_replay);
                }
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
            // A portal has a logical parent but projects directly under the
            // renderer root. It contributes no native root to that logical
            // parent's child list.
            HostKind::Portal { .. } => Vec::new(),
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
        event_type: NodeEventType,
        native_wrapper: usize,
        active: Rc<std::cell::Cell<bool>>,
    ) {
        self.hosts[host]
            .registered_event_listeners
            .retain(|listener| listener.event_type != event_type);
        self.hosts[host]
            .registered_event_listeners
            .push(RegisteredEventListener {
                event_type,
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
        if self.fault.is_some() {
            return;
        }
        self.activate_portals_in_subtree(child);
        if matches!(self.hosts[child].kind, HostKind::Portal { .. }) {
            self.ensure_native(child);
            return;
        }
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
        let insert_result = {
            let mut parent_mut = parent_native.borrow_mut();
            parent_mut.insert_child(child_native.clone(), native_index)
        };
        let inserted = self
            .latch_structural("attach_native insert_child", insert_result)
            .is_some();
        if !inserted {
            // Do not bind this logical child to whatever node happened to be
            // at the requested index when native insertion failed.
            return;
        }

        // Keep renderer state synchronized with the parent's mounted wrapper.
        // ohos-arkui-binding preserves this `Rc` identity across insertion, so
        // listeners registered before or after mounting target the same node.
        let mounted = parent_native.borrow().children().get(native_index).cloned();
        if let Some(mounted) = mounted {
            let parent_attached = self.hosts[parent].native_attached;
            self.hosts[child].native = Some(mounted);
            self.hosts[child].native_attached = parent_attached;
            self.rebind_mounted_projection(child, parent_attached);
        }
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
            HostKind::Root | HostKind::Element { .. } | HostKind::Portal { .. } => {
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
        if self.replay_event_listeners_inner(host, false) {
            self.mark_deferred_node_events(host);
        }
    }

    /// Reconcile native event routes for one host.
    ///
    /// Returns `true` when an event requested by this host must wait until the
    /// current mutation batch has fully attached the native projection.
    fn replay_event_listeners_inner(&mut self, host: HostId, include_deferred: bool) -> bool {
        let Some(sink) = self.sink.clone() else {
            return false;
        };
        let Some(native) = self.hosts[host].native.clone() else {
            return false;
        };
        let tag = self.hosts[host].tag();
        let native_wrapper = Self::native_wrapper_id(&native);
        let listeners = self.hosts[host].event_listeners.clone();

        let long_press = listeners
            .iter()
            .copied()
            .filter(|(name, _)| classify_event_name(name) == Some(ArkEventKind::LongPress))
            .collect::<Vec<_>>();
        self.hosts[host]
            .registered_gesture_listeners
            .retain(|registration| {
                long_press
                    .iter()
                    .any(|(name, id)| registration.name == *name && registration.id == *id)
            });
        for (name, id) in long_press {
            let already_registered = self.hosts[host]
                .registered_gesture_listeners
                .iter()
                .any(|registration| registration.name == name && registration.id == id);
            if !already_registered {
                if let Some(registration) = log_arkui_result(
                    "create_event_listener long_press",
                    register_long_press(&native, name, sink.clone(), id),
                ) {
                    self.remember_gesture_registration(host, registration);
                }
            }
        }

        let mut requested_event_types = listeners
            .iter()
            .filter_map(|(name, _)| event_type_for_name(name, tag))
            .collect::<Vec<_>>();
        // A native_ref is a mount capability by default. Layout and
        // visibility hooks opt into their respective ArkUI events explicitly;
        // animation/canvas/native-component refs therefore do not pay for or
        // risk unrelated platform event registrations.
        if self.hosts[host].native_attached {
            if self.hosts[host]
                .native_ref
                .as_ref()
                .is_some_and(NativeElementRef::observes_layout)
            {
                requested_event_types.push(NodeEventType::EventOnAreaChange);
            }
            if self.hosts[host]
                .native_ref
                .as_ref()
                .is_some_and(NativeElementRef::observes_visibility)
            {
                requested_event_types.extend([
                    NodeEventType::EventOnAppear,
                    NodeEventType::EventOnDisappear,
                    NodeEventType::EventOnVisibleAreaChange,
                ]);
            }
        }
        let mut event_types = Vec::with_capacity(requested_event_types.len());
        for event_type in requested_event_types {
            if !event_types.contains(&event_type) {
                event_types.push(event_type);
            }
        }

        self.hosts[host]
            .registered_event_listeners
            .retain(|registration| event_types.contains(&registration.event_type));
        self.hosts[host]
            .routed_node_events
            .retain(|route| event_types.contains(&route.event_type));

        let mut has_deferred_work = false;
        for event_type in event_types {
            let event_listeners = listeners
                .iter()
                .copied()
                .filter(|(name, _)| event_type_for_name(name, tag) == Some(event_type))
                .collect::<Vec<_>>();
            let route = if let Some(route) = self.hosts[host]
                .routed_node_events
                .iter()
                .find(|route| route.event_type == event_type)
            {
                route.route.clone()
            } else {
                let route = Rc::new(RefCell::new(NodeEventRoute {
                    node: Rc::downgrade(&native),
                    sink: sink.clone(),
                    listeners: Vec::new(),
                    native_ref: None,
                }));
                self.hosts[host].routed_node_events.push(RoutedNodeEvent {
                    event_type,
                    route: route.clone(),
                });
                route
            };
            {
                let mut route = route.borrow_mut();
                route.node = Rc::downgrade(&native);
                route.sink = sink.clone();
                route.listeners = event_listeners;
                route.native_ref = self.hosts[host].native_ref.as_ref().and_then(|reference| {
                    reference
                        .current()
                        .map(|lease| (reference.clone(), lease.epoch()))
                });
            }

            let installed =
                self.hosts[host]
                    .registered_event_listeners
                    .iter()
                    .any(|registration| {
                        registration.event_type == event_type
                            && registration.native_wrapper == native_wrapper
                    });
            if installed {
                continue;
            }
            if is_deferred_node_event(event_type) && !include_deferred {
                has_deferred_work = true;
                continue;
            }
            self.hosts[host]
                .registered_event_listeners
                .retain(|registration| registration.event_type != event_type);
            if event_type == NodeEventType::EventOnVisibleAreaChange {
                // ArkUI requires a ratio before this event can be registered.
                // A small positive threshold reliably crosses after first
                // layout and avoids a one-off pre-layout 0% notification.
                log_arkui_result(
                    "visible area threshold",
                    native
                        .borrow()
                        .set_visible_area_change_ratio(vec![0.001_f32]),
                );
            }
            let active = register_routed_node_event(&native, event_type, route);
            self.remember_event_registration(host, event_type, native_wrapper, active);
        }
        has_deferred_work
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
        if self.fault.is_some() {
            return;
        }
        let root = self.hosts.root();
        if parent == root {
            // A direct root reconciliation consumes every portal invalidation
            // accumulated so far in this mutation batch.
            self.projection.take_root_dirty();
        }
        let parent_tag = self.hosts[parent].tag();
        if Self::merges_text_children(parent_tag) {
            self.sync_content_attribute(parent);
            return;
        }
        let Some(parent_native) = self.native_child_container(parent) else {
            return;
        };

        let mut children = self.hosts[parent]
            .children
            .iter()
            .copied()
            .filter(|child| !matches!(self.hosts[*child].kind, HostKind::Portal { .. }))
            .collect::<Vec<_>>();
        if parent == root {
            let mut portals = self
                .projection
                .active_portals
                .iter()
                .filter(|(host, _)| self.hosts.is_connected_to_root(**host))
                .map(|(host, activation_order)| (*host, *activation_order))
                .collect::<Vec<_>>();
            debug_assert!(portals
                .iter()
                .all(|(host, _)| matches!(self.hosts[*host].kind, HostKind::Portal { .. })));
            portals.sort_by_key(|(host, activation_order)| {
                (
                    self.hosts[*host].kind.portal_layer().unwrap_or_default(),
                    *activation_order,
                )
            });
            children.extend(portals.into_iter().map(|(host, _)| host));
        }
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
            .collect::<FxHashSet<_>>();

        let child_count = parent_native.borrow().children().len();
        for index in (0..child_count).rev() {
            let should_remove = parent_native
                .borrow()
                .children()
                .get(index)
                .map(|child| !desired_raws.contains(&Self::native_raw_id(child)))
                .unwrap_or(false);
            if should_remove {
                let result = parent_native.borrow_mut().remove_child(index);
                if self
                    .latch_structural("sync_native_children remove_child", result)
                    .is_none()
                {
                    return;
                }
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
                    let wrapper_changed = !Rc::ptr_eq(&child_native, &mounted);
                    let was_attached = self.hosts[child].native_attached;
                    self.hosts[child].native = Some(mounted);
                    self.hosts[child].native_attached = parent_attached;
                    let became_attached = !was_attached && parent_attached;
                    if wrapper_changed || was_attached != parent_attached {
                        self.rebind_mounted_projection(child, became_attached);
                    }
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
                    let result = parent_native.borrow_mut().remove_child(index);
                    let removed = self
                        .latch_structural("sync_native_children detach reordered child", result);
                    let Some(Some(removed)) = removed else {
                        return;
                    };
                    removed
                } else {
                    child_native.clone()
                };
                let result = {
                    let mut parent_mut = parent_native.borrow_mut();
                    parent_mut.insert_child(node_to_insert, native_index)
                };
                let inserted = self
                    .latch_structural("sync_native_children insert_child", result)
                    .is_some();
                if !inserted {
                    return;
                }
                let mounted = parent_native.borrow().children().get(native_index).cloned();
                if let Some(mounted) = mounted {
                    self.hosts[child].native = Some(mounted);
                    self.hosts[child].native_attached = parent_attached;
                    self.rebind_mounted_projection(child, parent_attached);
                }
            }
        }
    }

    /// Tear down every integration that still needs a live native node.
    ///
    /// ArkUI recursively destroys descendants when an ancestor is disposed.
    /// Walk children first so item adapters, surfaces, web views, callbacks,
    /// and exact-node leases all release their native resources while their
    /// corresponding nodes are still valid.
    fn prepare_subtree_native_dispose(&mut self, host: HostId) {
        let children = self.hosts[host].children.clone();
        for child in children {
            self.prepare_subtree_native_dispose(child);
        }
        self.hosts[host].registered_gesture_listeners.clear();
        self.hosts[host].registered_event_listeners.clear();
        self.hosts[host].routed_node_events.clear();
        self.unbind_native_ref(host);
        if let Some(source) = self.hosts[host].virtual_source.take() {
            if let Some(native) = self.hosts[host].native.as_ref() {
                if let Err(error) = source.detach(native) {
                    ohos_hilog_binding::warn(format!(
                        "arkit_arkui: virtual source detach failed: {error}"
                    ));
                }
            }
        }
    }

    /// Clear renderer-owned state for a subtree whose native roots are already
    /// retained by an isolated ancestor.
    fn clear_subtree_state(&mut self, host: HostId) {
        let children = self.hosts[host].children.clone();
        for c in children {
            self.clear_subtree_state(c);
        }
        debug_assert!(
            self.hosts[host]
                .native_ref
                .as_ref()
                .is_none_or(|reference| self.hosts[host]
                    .native
                    .as_ref()
                    .is_none_or(|native| !reference.is_bound_to(native))),
            "native ref remained bound after pre-dispose teardown"
        );
        if let Some(source) = self.hosts[host].virtual_source.take() {
            // This is an exceptional fallback: the normal pre-dispose pass
            // detaches every source while its host is live.
            if let Some(native) = self.hosts[host].native.as_ref() {
                source.abandon_attachment(native);
            }
        }
        self.hosts[host].native = None;
        self.hosts[host].native_attached = false;
        self.hosts[host].content_native = None;
        self.hosts[host].event_listeners.clear();
        self.hosts[host].registered_event_listeners.clear();
        self.hosts[host].routed_node_events.clear();
        self.hosts[host].registered_gesture_listeners.clear();
        self.hosts[host].pending_scroll_offset = None;
        self.hosts[host].children.clear();
        self.hosts[host].parent = None;
        self.clear_host_image_source(host);
        self.release_host(host);
    }

    /// Retire a host subtree and clear renderer state.
    fn dispose_subtree(&mut self, host: HostId) {
        let removed_active_portal = self.deactivate_portals_in_subtree(host);
        let portal_branches = self.detach_portal_branches(host);
        if removed_active_portal {
            // Portal native nodes are root children rather than descendants of
            // their logical owner. Detach them before disposing either branch.
            self.flush_root_projection();
        }
        for portal in portal_branches {
            self.dispose_subtree(portal);
        }
        self.prepare_subtree_native_dispose(host);
        self.dispose_prepared_subtree(host);
    }

    /// Queue a subtree that has already been detached from the logical and
    /// native parent trees for disposal at the mutation-batch boundary.
    ///
    /// ArkUI may keep transient references to removed FrameNodes while a
    /// child-list patch is in progress. Releasing those nodes synchronously
    /// from `replace_node_with` or `remove_node` can therefore invalidate
    /// native reconciliation state that is still on the stack.
    fn retire_subtree(&mut self, host: HostId) {
        debug_assert!(
            self.hosts[host].parent.is_none(),
            "only detached subtrees can be retired"
        );
        debug_assert!(
            !self.pending_subtree_disposals.contains(&host),
            "a subtree can only be retired once"
        );
        self.pending_subtree_disposals.push(host);
    }

    /// Finalize subtrees retired by a completed mutation batch.
    ///
    /// The runtime calls this from a later OpenHarmony event-loop turn. ArkUI
    /// can retain transient FrameNode references until the native callback
    /// that triggered reconciliation has returned, so disposal must not run
    /// from [`Self::finish_mutation_batch`].
    pub fn dispose_retired_subtrees(&mut self) {
        let pending = std::mem::take(&mut self.pending_subtree_disposals);
        for host in pending {
            self.dispose_subtree(host);
        }
    }

    /// Whether a later event-loop turn must finish native subtree disposal.
    pub fn has_retired_subtrees(&self) -> bool {
        !self.pending_subtree_disposals.is_empty()
    }

    /// Isolate a subtree after [`Self::prepare_subtree_native_dispose`] has
    /// released every native-dependent integration.
    fn dispose_prepared_subtree(&mut self, host: HostId) {
        if let Some(native) = self.hosts[host].native.take() {
            self.retired_native_subtrees.push(native);
            let children = self.hosts[host].children.clone();
            for child in children {
                self.clear_subtree_state(child);
            }
        } else {
            let children = self.hosts[host].children.clone();
            for child in children {
                self.dispose_prepared_subtree(child);
            }
        }
        self.hosts[host].content_native = None;
        self.hosts[host].native_attached = false;
        self.hosts[host].event_listeners.clear();
        self.hosts[host].registered_event_listeners.clear();
        self.hosts[host].routed_node_events.clear();
        self.hosts[host].registered_gesture_listeners.clear();
        self.hosts[host].pending_scroll_offset = None;
        self.hosts[host].children.clear();
        self.hosts[host].parent = None;
        self.clear_host_image_source(host);
        self.release_host(host);
    }

    fn detach_portal_branches(&mut self, host: HostId) -> Vec<HostId> {
        let children = self.hosts[host].children.clone();
        let mut portals = Vec::new();
        for child in children {
            if matches!(self.hosts[child].kind, HostKind::Portal { .. }) {
                self.hosts[host]
                    .children
                    .retain(|candidate| *candidate != child);
                self.hosts[child].parent = None;
                portals.push(child);
            } else {
                portals.extend(self.detach_portal_branches(child));
            }
        }
        portals
    }

    fn discard_detached_hosts(&mut self, hosts: impl IntoIterator<Item = HostId>) {
        for host in hosts {
            debug_assert_ne!(
                host,
                self.hosts.root(),
                "the synthetic root cannot be a detached mutation"
            );
            self.dispose_subtree(host);
        }
    }

    fn bind_native_ref(&self, host: HostId) {
        if !self.hosts[host].native_attached {
            return;
        }
        let Some(reference) = self.hosts[host].native_ref.clone() else {
            return;
        };
        let Some(native) = self.hosts[host].native.as_ref() else {
            return;
        };
        let Some(event) = reference.bind(native) else {
            return;
        };
        if let Some(sink) = &self.sink {
            sink.dispatch_native_ref(NativeElementDelivery::new(reference, event));
        }
    }

    fn unbind_native_ref(&self, host: HostId) {
        let Some(reference) = self.hosts[host].native_ref.clone() else {
            return;
        };
        let Some(native) = self.hosts[host].native.as_ref() else {
            return;
        };
        let Some(event) = reference.unbind(native) else {
            return;
        };
        if let Some(sink) = &self.sink {
            sink.dispatch_native_ref(NativeElementDelivery::new(reference, event));
        }
    }

    fn set_native_ref(&mut self, host: HostId, reference: Option<NativeElementRef>) {
        let unchanged = self.hosts[host].native_ref == reference;
        if unchanged {
            self.bind_native_ref(host);
            self.replay_event_listeners(host);
            return;
        }
        self.unbind_native_ref(host);
        self.hosts[host].native_ref = reference;
        self.bind_native_ref(host);
        self.replay_event_listeners(host);
    }

    fn attach_virtual_source(&self, host: HostId) {
        if !self.hosts[host].native_attached {
            return;
        }
        let Some(source) = self.hosts[host].virtual_source.clone() else {
            return;
        };
        let Some(native) = self.hosts[host].native.as_ref() else {
            return;
        };
        if let Err(error) = source.attach(native) {
            ohos_hilog_binding::error(format!(
                "arkit_arkui: virtual source attach failed: {error}"
            ));
        }
    }

    fn set_virtual_source(&mut self, host: HostId, source: Option<VirtualSource>) {
        if self.hosts[host].virtual_source == source {
            self.attach_virtual_source(host);
            return;
        }
        if let Some(previous) = self.hosts[host].virtual_source.take() {
            if let Some(native) = self.hosts[host].native.as_ref() {
                if let Err(error) = previous.detach(native) {
                    ohos_hilog_binding::warn(format!(
                        "arkit_arkui: virtual source replacement detach failed: {error}"
                    ));
                }
            }
        }
        self.hosts[host].virtual_source = source;
        self.attach_virtual_source(host);
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
                let kind = if *tag == "portal" {
                    HostKind::Portal {
                        layer: PortalLayer::Modal,
                    }
                } else {
                    HostKind::Element { tag }
                };
                let host = self.alloc_host(kind);
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
            .map(|_| self.hosts.pop().expect("stack underflow"))
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
        let top = self
            .hosts
            .stack_last()
            .expect("arkit_arkui: assign_node_id with empty stack");
        let target = self.walk_host_path(top, path);
        self.bind_element(id, target);
    }

    fn create_placeholder(&mut self, id: ElementId) {
        let host = self.alloc_host(HostKind::Placeholder);
        self.bind_element(id, host);
        self.hosts.push(host);
    }

    fn create_text_node(&mut self, value: &str, id: ElementId) {
        let host = self.alloc_host(HostKind::Text {
            value: value.to_string(),
        });
        self.bind_element(id, host);
        self.hosts.push(host);
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
        self.hosts.push(host);
    }

    fn replace_node_with(&mut self, id: ElementId, m: usize) {
        let target = self.host_of(id);
        if target == self.hosts.root() {
            ohos_hilog_binding::warn("arkit_arkui: replace_node_with on root not supported");
            // Consume and dispose pending nodes to keep both the mutation stack
            // and host arena consistent after rejecting the operation.
            let discarded = (0..m).filter_map(|_| self.hosts.pop()).collect::<Vec<_>>();
            self.discard_detached_hosts(discarded);
            return;
        }
        let new_hosts: Vec<HostId> = (0..m)
            .map(|_| self.hosts.pop().expect("stack underflow"))
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
        self.hosts[target].parent = None;

        // Insert the new hosts at the same logical position.
        for (offset, &child) in new_hosts.iter().enumerate() {
            self.hosts[child].parent = Some(parent);
            self.hosts[parent]
                .children
                .insert(logical_index + offset, child);
            self.activate_portals_in_subtree(child);
        }
        self.sync_native_children(parent);
        self.retire_subtree(target);
    }

    fn replace_placeholder_with_nodes(&mut self, path: &'static [u8], m: usize) {
        let new_hosts: Vec<HostId> = (0..m)
            .map(|_| self.hosts.pop().expect("stack underflow"))
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        let top = self
            .hosts
            .stack_last()
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
            self.activate_portals_in_subtree(child);
        }
        self.sync_native_children(parent);
    }

    fn insert_nodes_after(&mut self, id: ElementId, m: usize) {
        let new_hosts: Vec<HostId> = (0..m)
            .map(|_| self.hosts.pop().expect("stack underflow"))
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
            .map(|_| self.hosts.pop().expect("stack underflow"))
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

        if name == "native_ref" {
            let reference = match value {
                dioxus_core::AttributeValue::Any(value) => {
                    value.as_any().downcast_ref::<NativeElementRef>().cloned()
                }
                dioxus_core::AttributeValue::None => None,
                _ => {
                    ohos_hilog_binding::warn(
                        "arkit_arkui: native_ref requires a NativeElementRef value",
                    );
                    None
                }
            };
            self.set_native_ref(host, reference);
            return;
        }

        if name == "virtual_source" {
            let source = match value {
                dioxus_core::AttributeValue::Any(value) => {
                    value.as_any().downcast_ref::<VirtualSource>().cloned()
                }
                dioxus_core::AttributeValue::None => None,
                _ => {
                    ohos_hilog_binding::warn(
                        "arkit_arkui: virtual_source requires a VirtualSource value",
                    );
                    None
                }
            };
            self.set_virtual_source(host, source);
            return;
        }

        if name == "portal_layer" {
            let layer = match value {
                dioxus_core::AttributeValue::Text(value) => match value.as_str() {
                    "floating" => PortalLayer::Floating,
                    "transient" => PortalLayer::Transient,
                    _ => PortalLayer::Modal,
                },
                dioxus_core::AttributeValue::Int(value) => match *value {
                    1 => PortalLayer::Floating,
                    2 => PortalLayer::Transient,
                    _ => PortalLayer::Modal,
                },
                _ => PortalLayer::Modal,
            };
            if matches!(self.hosts[host].kind, HostKind::Portal { .. }) {
                let previous = self.hosts[host].kind.portal_layer();
                self.hosts[host].kind = HostKind::Portal { layer };
                if previous != Some(layer) {
                    self.projection.mark_portal_order_dirty(host);
                }
            }
            return;
        }

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

        if name == "scroll_offset" && tag == "scroll" {
            self.hosts[host].pending_scroll_offset = ScrollOffsetCommand::from_attribute(value);
            self.apply_pending_scroll_offset(host);
            return;
        }

        // Store in desired_attrs (the source of truth for replay).
        let mutation = self.hosts[host]
            .desired_attrs
            .borrow_mut()
            .set(tag, name, value);

        // Attributes currently driven by an animation stay declarative-only:
        // writing them here would fight the animation's per-frame writes (and
        // be fought back, causing visible jitter). The animation clears its
        // declaration when it finishes, after which the declarative value
        // becomes the steady state again.
        let animated = self.hosts[host]
            .native_ref
            .as_ref()
            .is_some_and(|reference| reference.animates(name));

        if let Some(native) = self.hosts[host].native.clone() {
            let desired_attrs = self.hosts[host].desired_attrs.borrow();
            if !matches!(mutation, AttrMutation::Unchanged) && !animated {
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
        let host = self.host_of(id);
        if !self.hosts[host]
            .event_listeners
            .iter()
            .any(|&(existing, existing_id)| existing == name && existing_id == id)
        {
            self.hosts[host].event_listeners.push((name, id));
        }
        self.replay_event_listeners(host);
    }

    fn remove_event_listener(&mut self, name: &'static str, id: ElementId) {
        let host = self.host_of(id);
        self.hosts[host]
            .event_listeners
            .retain(|&(existing, existing_id)| existing != name || existing_id != id);
        self.hosts[host]
            .registered_gesture_listeners
            .retain(|listener| listener.name != name || listener.id != id);
        self.replay_event_listeners(host);
    }

    fn remove_node(&mut self, id: ElementId) {
        let host = self.host_of(id);
        if host == self.hosts.root() {
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
        self.hosts[host].parent = None;
        self.sync_native_children(parent);
        self.retire_subtree(host);
    }

    fn push_root(&mut self, id: ElementId) {
        let host = self.host_of(id);
        self.hosts.push(host);
    }
}

impl ArkUIRenderer {
    /// Install the runtime-side dispatcher used by one-shot `EventOnAppear`
    /// declarative replays.
    ///
    /// The closure receives the dioxus element id of the host whose native
    /// node just appeared; the runtime routes it back into
    /// [`Self::replay_element_attrs`] on the UI thread.
    pub fn set_appear_replay_handler(&mut self, handler: Rc<dyn Fn(ElementId)>) {
        self.appear_replay_handler = Some(handler);
    }

    /// Arm a one-shot `EventOnAppear` replay for a freshly attached host.
    ///
    /// ArkUI controls (notably `Button`) may write their own skin attributes
    /// after insertion into the mounted tree, so the synchronous replay done
    /// during the mutation batch can be clobbered before first paint.
    /// `EventOnAppear` fires after layout and before the node is drawn, so a
    /// replay from this callback converges on the declarative style in a
    /// single frame.
    fn arm_appear_replay(&mut self, host: HostId) {
        if self.appear_replay_handler.is_none() || self.hosts[host].appear_replay_armed {
            return;
        }
        let Some(native) = self.hosts[host].native.clone() else {
            return;
        };
        if !self.hosts[host].native_attached {
            return;
        }
        let Some(element) = self.hosts.element_for_host(host) else {
            return;
        };
        let handler = self.appear_replay_handler.clone().expect("checked above");
        let active = Rc::new(std::cell::Cell::new(true));
        let callback_active = active.clone();
        {
            let mut borrowed = native.borrow_mut();
            let mut event_node = EventNode(&mut borrowed);
            event_node.on_event(NodeEventType::EventOnAppear, move |_| {
                if !callback_active.get() {
                    return;
                }
                callback_active.set(false);
                handler(ElementId(element.index()));
            });
        }
        self.hosts[host].appear_replay_armed = true;
        self.hosts[host]
            .registered_event_listeners
            .push(RegisteredEventListener {
                event_type: NodeEventType::EventOnAppear,
                native_wrapper: native.borrow().raw_handle() as usize,
                active,
            });
    }

    /// Replay declarative attrs for one element after its native node
    /// appeared. No-op when the element is gone or already replayed.
    ///
    /// Attributes currently driven by an animation are skipped (the animation
    /// owns the live value until it finishes).
    pub fn replay_element_attrs(&mut self, element: ElementId) {
        let Some(host) = self.hosts.host_for_element(ElementKey::new(element.0)) else {
            return;
        };
        if !self.hosts[host].appear_replay_armed {
            return;
        }
        self.hosts[host].appear_replay_armed = false;
        let Some(native) = self.hosts[host].native.clone() else {
            return;
        };
        let tag = self.hosts[host].tag();
        let animated = self.hosts[host]
            .native_ref
            .as_ref()
            .map(NativeElementRef::animated_attrs)
            .unwrap_or_default();
        let attrs = self.hosts[host].desired_attrs.borrow();
        if animated.is_empty() {
            attrs.apply_to(&mut native.borrow_mut(), tag);
        } else {
            attrs.apply_to_skipping(&mut native.borrow_mut(), tag, &animated);
        }
        attrs.after_patch(&mut native.borrow_mut(), tag);
        drop(attrs);
        self.apply_host_image_source(host);
        self.replay_composite_content(host);
    }

    /// Commit renderer work that must run after a complete Dioxus mutation
    /// batch.
    ///
    /// Portal membership/order invalidates the root projection, while deferred
    /// native observers invalidate only their owning hosts. Both queues are
    /// deduplicated during mutation writing and consumed exactly once here.
    pub fn finish_mutation_batch(&mut self) {
        self.flush_root_projection();
        for host in self.projection.deferred_event_hosts.drain() {
            if self.hosts[host].native_attached {
                let _ = self.replay_event_listeners_inner(host, true);
            }
        }
    }

    /// Unmount the root from the NodeContent slot.
    pub fn unmount(&mut self) -> ArkUIResult<()> {
        if self.inert {
            // The native subtree is already gone. Release every Rust-side
            // handle without touching native APIs: wrapper drops are inert
            // (ArkUINode has no Drop), virtual sources dispose their
            // independently-owned adapters, and retained teardown closures are
            // dropped unexecuted — their resources release through their own
            // Drop impls (WebView unrefs its N-API reference, animators dispose
            // independent native objects).
            self.hosts = HostTree::new(NativeHostState::default());
            self.templates.clear();
            self.projection = ProjectionState::default();
            self.pending_subtree_disposals.clear();
            self.retired_native_subtrees.clear();
            self.fault = None;
            return Ok(());
        }
        self.dispose_retired_subtrees();
        // Every native-dependent integration must stop before RootNode
        // destroys the native subtree (or before an embedded root's caller
        // disposes it). This pass is idempotent, so explicit unmount followed
        // by Drop is safe.
        self.prepare_subtree_native_dispose(self.hosts.root());
        match &mut self.root_mount {
            RendererRootMount::NodeContent(root_node) => root_node.unmount(),
            RendererRootMount::Embedded => Ok(()),
        }
    }

    /// Walk the host tree from `root` following `path` (logical child indices),
    /// returning the target host id.
    fn walk_host_path(&self, root: HostId, path: &[u8]) -> HostId {
        self.hosts
            .walk_path(root, path)
            .expect("arkit_arkui: host path index out of range")
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

/// Register exactly one native callback for a node event type. The callback
/// multiplexes declarative Dioxus listeners and exact-element observers.
fn register_routed_node_event(
    node: &NodeRef,
    event_type: NodeEventType,
    route: Rc<RefCell<NodeEventRoute>>,
) -> Rc<std::cell::Cell<bool>> {
    let mut borrowed = node.borrow_mut();
    let mut event_node = EventNode(&mut borrowed);
    let active = Rc::new(std::cell::Cell::new(true));
    let callback_active = active.clone();
    event_node.on_event(event_type, move |event: &ArkNativeEvent| {
        if !callback_active.get() {
            return;
        }
        let (node, sink, listeners, native_ref) = {
            let route = route.borrow();
            (
                route.node.upgrade(),
                route.sink.clone(),
                route.listeners.clone(),
                route.native_ref.clone(),
            )
        };
        let payload = extract_payload(event_type, event, node.as_ref());
        for (name, id) in listeners {
            sink.dispatch(name, id, payload.clone());
        }
        let Some((reference, epoch)) = native_ref else {
            return;
        };
        let native_event = match event_type {
            NodeEventType::EventOnAreaChange => match payload {
                ArkEventPayload::Layout(layout) => Some(NativeElementEvent::Layout {
                    epoch,
                    frame: LayoutFramePx {
                        x: layout.x,
                        y: layout.y,
                        width: layout.width,
                        height: layout.height,
                    },
                }),
                _ => None,
            },
            NodeEventType::EventOnAppear => Some(NativeElementEvent::Visibility {
                epoch,
                visibility: NativeVisibility {
                    visible: true,
                    fraction: 1.0,
                },
            }),
            NodeEventType::EventOnDisappear => Some(NativeElementEvent::Visibility {
                epoch,
                visibility: NativeVisibility::default(),
            }),
            NodeEventType::EventOnVisibleAreaChange => {
                // data[0] is the direction of the ratio change, not current
                // visibility. A decreasing node can still be partially
                // visible, so derive presentation state from the ratio.
                let fraction = event.f32_value(1).unwrap_or_default();
                Some(NativeElementEvent::Visibility {
                    epoch,
                    visibility: NativeVisibility {
                        visible: fraction.is_finite() && fraction > f32::EPSILON,
                        fraction,
                    },
                })
            }
            _ => None,
        };
        if let Some(event) = native_event {
            sink.dispatch_native_ref(NativeElementDelivery::new(reference, event));
        }
    });
    active
}

fn is_deferred_node_event(event_type: NodeEventType) -> bool {
    matches!(
        event_type,
        NodeEventType::EventOnAreaChange
            | NodeEventType::EventOnAppear
            | NodeEventType::EventOnDisappear
            | NodeEventType::EventOnVisibleAreaChange
    )
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
/// The value accessor + index differ per event type: ArkUI stores each event's
/// values at fixed numeric indices exposed by
/// `i32_value`/`f32_value`/`string_value`/`async_string`.
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
        ScrollEventOnDidScroll => ArkEventPayload::ScrollOffset(ScrollOffsetPayload {
            x: component_event_f32(event, 0).unwrap_or_default(),
            y: component_event_f32(event, 1).unwrap_or_default(),
        }),
        EventOnAreaChange => node
            .and_then(extract_layout_payload)
            .map(ArkEventPayload::Layout)
            .unwrap_or_default(),
        // Swiper change: new index i32(0).
        SwiperEventOnChange | SwiperEventOnAnimationEnd => {
            ArkEventPayload::Int(event.i32_value(0).unwrap_or(0))
        }
        // Legacy hover stores an integer; API 17+ hover stores a UIInputEvent.
        OnHover => ArkEventPayload::Bool(event.i32_value(0).unwrap_or(0) != 0),
        // `OnClickEvent` is a valid UIInputEvent, but it does not necessarily
        // carry a `UI_TOUCH_EVENT_ACTION`. The current upstream input binding
        // parses that field eagerly and panics on the platform's sentinel
        // value, so clicks remain payload-free rather than crashing dispatch.
        OnClickEvent => ArkEventPayload::None,
        TouchEvent => extract_pointer_payload(event)
            .map(ArkEventPayload::Pointer)
            .unwrap_or_default(),
        // These are non-touch UIInputEvents. Keep delivery intact without
        // asking the upstream wrapper to interpret their action as touch.
        OnHoverEvent | OnHoverMove | OnMouse => ArkEventPayload::None,
        // Drag callbacks carry ArkUI_DragEvent, not ArkUI_UIInputEvent. The
        // binding intentionally exposes that object as an opaque pointer, so
        // dispatch the lifecycle event without inventing pointer coordinates.
        OnDragStart | OnDragMove | OnDragEnd | OnDragEnter | OnDragLeave | OnDrop => {
            ArkEventPayload::None
        }
        _ => ArkEventPayload::None,
    }
}

/// Read the component-event union used by Scroll's `OnDidScroll`.
///
/// `OH_ArkUI_NodeEvent_GetNumberValue` returns zero for Scroll component data
/// on current API-20 devices even though `ArkUI_NodeComponentEvent::data`
/// contains the documented per-frame offsets. The pointer is owned by ArkUI
/// and remains valid only for the duration of the callback.
fn component_event_f32(event: &ArkNativeEvent, index: usize) -> Option<f32> {
    if index >= 12 {
        return None;
    }
    let component = event
        .node_component_event()?
        .cast::<ArkUI_NodeComponentEvent>();
    // SAFETY: ArkUI returned this pointer for the active synchronous callback.
    // `ArkUI_NodeComponentEvent` has a fixed 12-element `data` array, and the
    // index is checked above.
    Some(unsafe { component.as_ref().data[index].f32_ })
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
    // Prefer layout position (window, no graphic translate). Translate-inclusive
    // coords accumulate ancestor matrix offsets and mis-anchor floating panels
    // (Select same-width start align was ~48vp too far right on device).
    let position = n
        .layout_position_in_window()
        .or_else(|_| n.position_with_translate_in_window())
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
        (ArkEventKind::Click, _) => OnClickEvent,

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
        (ArkEventKind::Scroll, "scroll") => ScrollEventOnDidScroll,
        (ArkEventKind::ReachEnd, "scroll") => ScrollEventOnReachEnd,

        // Swiper change. Animation-end is used as the stable selection
        // boundary: unlike the early change callback, it fires after the
        // native viewport has committed its new index.
        (ArkEventKind::SwiperChange, "swiper") => SwiperEventOnAnimationEnd,

        // Refresh trigger.
        (ArkEventKind::Refresh, "refresh") => RefreshOnRefresh,

        // Keep the numeric hover variant until the upstream input binding can
        // parse non-touch input actions without panicking.
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
    use super::{
        event_type_for_name, latch_renderer_fault, DirtyHostQueue, HostId, NodeEventType,
        ProjectionState,
    };

    #[test]
    fn component_events_use_their_typed_native_event() {
        assert_eq!(
            event_type_for_name("click", "row"),
            Some(NodeEventType::OnClickEvent)
        );
        assert_eq!(
            event_type_for_name("hover", "row"),
            Some(NodeEventType::OnHover)
        );
        assert_eq!(
            event_type_for_name("change", "toggle"),
            Some(NodeEventType::ToggleOnChange)
        );
        assert_eq!(
            event_type_for_name("_hover_move", "column"),
            Some(NodeEventType::OnHoverMove)
        );
        assert_eq!(
            event_type_for_name("scroll", "scroll"),
            Some(NodeEventType::ScrollEventOnDidScroll)
        );
        assert_eq!(event_type_for_name("scroll", "grid"), None);
        assert_eq!(
            event_type_for_name("_reach_end", "scroll"),
            Some(NodeEventType::ScrollEventOnReachEnd)
        );
        assert_eq!(event_type_for_name("reach_end", "list"), None);
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

    #[test]
    fn dirty_host_queue_deduplicates_and_preserves_first_mark_order() {
        let first = HostId::new(7);
        let second = HostId::new(3);
        let mut queue = DirtyHostQueue::default();

        queue.mark(first);
        queue.mark(second);
        queue.mark(first);

        assert_eq!(queue.drain(), vec![first, second]);
        assert!(queue.drain().is_empty());
    }

    #[test]
    fn discarded_dirty_host_can_be_reused_without_a_stale_entry() {
        let host = HostId::new(4);
        let mut queue = DirtyHostQueue::default();

        queue.mark(host);
        queue.discard(host);
        queue.mark(host);

        assert_eq!(queue.drain(), vec![host]);
    }

    #[test]
    fn portal_registry_invalidates_root_only_on_semantic_changes() {
        let portal = HostId::new(9);
        let mut projection = ProjectionState::default();

        assert!(projection.activate_portal(portal));
        assert!(!projection.activate_portal(portal));
        assert!(projection.take_root_dirty());
        assert!(!projection.take_root_dirty());

        projection.mark_portal_order_dirty(portal);
        assert!(projection.take_root_dirty());

        assert!(projection.deactivate_portal(portal));
        assert!(!projection.deactivate_portal(portal));
        assert!(projection.take_root_dirty());
        assert!(!projection.take_root_dirty());
    }

    #[test]
    fn portal_activation_order_survives_host_slot_reuse() {
        let reused = HostId::new(9);
        let older = HostId::new(10);
        let mut projection = ProjectionState::default();

        projection.activate_portal(reused);
        projection.activate_portal(older);
        projection.deactivate_portal(reused);
        projection.activate_portal(reused);

        assert!(projection.active_portals[&older] < projection.active_portals[&reused]);
    }

    #[test]
    fn renderer_fault_latch_keeps_the_first_structural_failure() {
        let mut fault = None;
        latch_renderer_fault(&mut fault, "insert child", "first".to_string());
        latch_renderer_fault(&mut fault, "detach child", "second".to_string());

        assert_eq!(
            fault.as_ref().map(ToString::to_string).as_deref(),
            Some("insert child failed: first")
        );
    }
}
