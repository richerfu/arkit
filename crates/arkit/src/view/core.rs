use std::any::{type_name, TypeId};
use std::mem::{align_of, size_of, ManuallyDrop};

use crate::component::{run_cleanups, Cleanup, MountedElement};
use crate::logging;
use crate::ohos_arkui_binding::api::node_custom_event::NodeCustomEvent;
use crate::ohos_arkui_binding::common::attribute::ArkUINodeAttributeItem;
use crate::ohos_arkui_binding::common::error::ArkUIResult;
use crate::ohos_arkui_binding::common::node::ArkUINode;
use crate::ohos_arkui_binding::component::attribute::{
    ArkUICommonAttribute, ArkUICommonFontAttribute, ArkUIEvent, ArkUIGesture,
};
#[cfg(feature = "api-22")]
use crate::ohos_arkui_binding::component::built_in_component::CheckboxGroup;
#[cfg(feature = "api-22")]
use crate::ohos_arkui_binding::component::built_in_component::XComponentTexture;
use crate::ohos_arkui_binding::component::built_in_component::{
    Button, CalendarPicker, Checkbox, Column, Custom, CustomSpan, DatePicker, Flex, FlowItem, Grid,
    GridItem, Image, ImageAnimator, ImageSpan, List, ListItem, ListItemGroup, LoadingProgress,
    Progress, Radio, Refresh, RelativeContainer, Row, Scroll, Slider, Span, Stack, Swiper, Text,
    TextArea, TextInput, TextPicker, TimePicker, Toggle, WaterFlow, XComponent,
};
#[cfg(feature = "api-22")]
use crate::ohos_arkui_binding::component::built_in_component::{EmbeddedComponent, Undefined};
use crate::ohos_arkui_binding::event::inner_event::Event as ArkEvent;
use crate::ohos_arkui_binding::types::advanced::NodeCustomEventType;
use crate::ohos_arkui_binding::types::attribute::ArkUINodeAttributeType;
use crate::ohos_arkui_binding::types::event::NodeEventType;
use crate::signal::Signal;

use super::element::{Element, ViewNode};

type Effect<T> = Box<dyn FnOnce(&mut T) -> ArkUIResult<Option<Cleanup>>>;

pub struct ComponentElement<T> {
    constructor: fn() -> ArkUIResult<T>,
    key: Option<String>,
    /// Attributes applied once at mount only (Solid-style static setup).
    init_attrs: Vec<(ArkUINodeAttributeType, ArkUINodeAttributeItem)>,
    /// Attributes reapplied on every patch when the parent re-renders with new values.
    patch_attrs: Vec<(ArkUINodeAttributeType, ArkUINodeAttributeItem)>,
    /// Side effects: events, subscriptions, `native` blocks — run on mount and on each patch.
    effects: Vec<Effect<T>>,
    children: Vec<Element>,
}

trait HostComponent: ArkUICommonAttribute + Into<ArkUINode> + 'static {
    fn from_existing(node: ArkUINode) -> Self;

    fn kind() -> TypeId {
        TypeId::of::<Self>()
    }

    fn name() -> &'static str {
        type_name::<Self>()
    }
}

fn wrap_component<T>(node: ArkUINode) -> T {
    assert_eq!(size_of::<T>(), size_of::<ArkUINode>());
    assert_eq!(align_of::<T>(), align_of::<ArkUINode>());
    let node = ManuallyDrop::new(node);
    unsafe { std::ptr::read((&*node as *const ArkUINode).cast::<T>()) }
}

fn wrap_effect<T>(mutator: impl FnOnce(&mut T) -> ArkUIResult<()> + 'static) -> Effect<T> {
    Box::new(move |node| {
        mutator(node)?;
        Ok(None)
    })
}

fn apply_effects<T>(node: &mut T, effects: Vec<Effect<T>>) -> ArkUIResult<Vec<Cleanup>> {
    let mut cleanups = Vec::new();
    for (index, effect) in effects.into_iter().enumerate() {
        match effect(node) {
            Ok(Some(cleanup)) => cleanups.push(cleanup),
            Ok(None) => {}
            Err(error) => {
                logging::error(format!(
                    "component error: failed to apply effect #{index} on {}: {error}",
                    type_name::<T>()
                ));
                run_cleanups(cleanups);
                return Err(error);
            }
        }
    }
    Ok(cleanups)
}

fn apply_attr_list<T: ArkUICommonAttribute>(
    node: &mut T,
    attrs: Vec<(ArkUINodeAttributeType, ArkUINodeAttributeItem)>,
    phase: &'static str,
    component_name: &'static str,
) {
    for (attr, value) in attrs {
        if let Err(error) = node.set_attribute(attr, value) {
            logging::error(format!(
                "{phase} error: failed to set attribute {attr:?} on {component_name}: {error}"
            ));
        }
    }
}

macro_rules! impl_host_component {
    ($($ty:ty),* $(,)?) => {
        $(
            impl HostComponent for $ty {
                fn from_existing(node: ArkUINode) -> Self {
                    // All generated ArkUI component wrappers are single-field tuple structs over
                    // `ArkUINode`. We reuse that representation here to patch existing nodes.
                    wrap_component(node)
                }
            }
        )*
    };
}

impl_host_component!(
    Button,
    CalendarPicker,
    Checkbox,
    Column,
    Custom,
    CustomSpan,
    DatePicker,
    Flex,
    FlowItem,
    Grid,
    GridItem,
    Image,
    ImageAnimator,
    ImageSpan,
    List,
    ListItem,
    ListItemGroup,
    LoadingProgress,
    Progress,
    Radio,
    Refresh,
    RelativeContainer,
    Row,
    Scroll,
    Slider,
    Span,
    Stack,
    Swiper,
    Text,
    TextArea,
    TextInput,
    TextPicker,
    TimePicker,
    Toggle,
    WaterFlow,
    XComponent,
);

#[cfg(feature = "api-22")]
impl_host_component!(CheckboxGroup);

#[cfg(feature = "api-22")]
impl_host_component!(XComponentTexture);

#[cfg(feature = "api-22")]
impl_host_component!(EmbeddedComponent, Undefined);

impl<T> ComponentElement<T> {
    pub fn new(constructor: fn() -> ArkUIResult<T>) -> Self {
        Self {
            constructor,
            key: None,
            init_attrs: Vec::new(),
            patch_attrs: Vec::new(),
            effects: Vec::new(),
            children: Vec::new(),
        }
    }

    pub fn key(mut self, key: impl Into<String>) -> Self {
        self.key = Some(key.into());
        self
    }

    pub fn with(mut self, mutator: impl FnOnce(&mut T) -> ArkUIResult<()> + 'static) -> Self {
        self.effects.push(wrap_effect(mutator));
        self
    }

    pub fn with_cleanup<C>(
        mut self,
        mutator: impl FnOnce(&mut T) -> ArkUIResult<C> + 'static,
    ) -> Self
    where
        C: FnOnce() + 'static,
    {
        self.effects.push(Box::new(move |node| {
            mutator(node).map(|cleanup| Some(Box::new(cleanup) as Cleanup))
        }));
        self
    }

    pub fn native(self, mutator: impl FnOnce(&mut T) -> ArkUIResult<()> + 'static) -> Self {
        self.with(mutator)
    }

    pub fn native_with_cleanup<C>(
        self,
        mutator: impl FnOnce(&mut T) -> ArkUIResult<C> + 'static,
    ) -> Self
    where
        C: FnOnce() + 'static,
    {
        self.with_cleanup(mutator)
    }
}

impl<T> ComponentElement<T>
where
    T: ArkUICommonAttribute + 'static,
{
    #[allow(private_bounds)]
    pub fn watch_signal<S>(
        self,
        signal: Signal<S>,
        apply: impl Fn(&mut T, S) -> ArkUIResult<()> + 'static,
    ) -> Self
    where
        S: Clone + 'static,
        T: HostComponent,
    {
        let apply = std::rc::Rc::new(apply);
        self.native_with_cleanup(move |node| {
            if let Err(error) = apply(node, signal.get()) {
                logging::error(format!(
                    "signal error: initial watch apply failed on {}: {error}",
                    T::name()
                ));
                return Err(error);
            }

            let subscription_node =
                std::rc::Rc::new(std::cell::RefCell::new(node.borrow_mut().clone()));
            let subscription_signal = signal.clone();
            let subscription_apply = apply.clone();
            let subscription_node_clone = subscription_node.clone();
            let subscription_id = signal.subscribe(move || {
                let value = subscription_signal.get();
                let raw_node = { subscription_node_clone.borrow().clone() };
                let mut typed = T::from_existing(raw_node);
                match subscription_apply(&mut typed, value) {
                    Ok(()) => {
                        *subscription_node_clone.borrow_mut() = typed.into();
                    }
                    Err(error) => {
                        logging::error(format!(
                            "signal error: subscription apply failed on {}: {error}",
                            T::name()
                        ));
                    }
                }
            });

            Ok(move || {
                signal.unsubscribe(subscription_id);
            })
        })
    }

    pub fn child(mut self, child: impl Into<Element>) -> Self {
        self.children.push(child.into());
        self
    }

    pub fn children(mut self, children: Vec<Element>) -> Self {
        self.children.extend(children);
        self
    }

    /// Attribute applied only at mount (not on patch). Use for layout and structure.
    pub fn attr(
        mut self,
        attr: ArkUINodeAttributeType,
        value: impl Into<ArkUINodeAttributeItem>,
    ) -> Self {
        self.init_attrs.push((attr, value.into()));
        self
    }

    /// Same as [`Self::attr`] — mount-only styling.
    pub fn style(
        self,
        attr: ArkUINodeAttributeType,
        value: impl Into<ArkUINodeAttributeItem>,
    ) -> Self {
        self.attr(attr, value)
    }

    /// Attribute reapplied on every patch when props may change (parent re-render).
    pub fn patch_attr(
        mut self,
        attr: ArkUINodeAttributeType,
        value: impl Into<ArkUINodeAttributeItem>,
    ) -> Self {
        self.patch_attrs.push((attr, value.into()));
        self
    }

    pub fn constraint_size(
        mut self,
        min_width: f32,
        max_width: f32,
        min_height: f32,
        max_height: f32,
    ) -> Self {
        self.init_attrs.push((
            ArkUINodeAttributeType::ConstraintSize,
            vec![min_width, max_width, min_height, max_height].into(),
        ));
        self
    }

    pub fn max_width_constraint(self, value: f32) -> Self {
        self.constraint_size(0.0, value, 0.0, 100000.0)
    }

    pub fn width(mut self, value: f32) -> Self {
        self.init_attrs
            .push((ArkUINodeAttributeType::Width, value.into()));
        self
    }

    pub fn height(mut self, value: f32) -> Self {
        self.init_attrs
            .push((ArkUINodeAttributeType::Height, value.into()));
        self
    }

    pub fn percent_width(mut self, value: f32) -> Self {
        self.init_attrs
            .push((ArkUINodeAttributeType::WidthPercent, value.into()));
        self
    }

    pub fn percent_height(mut self, value: f32) -> Self {
        self.init_attrs
            .push((ArkUINodeAttributeType::HeightPercent, value.into()));
        self
    }

    pub fn background_color(mut self, value: u32) -> Self {
        self.init_attrs.push((
            ArkUINodeAttributeType::BackgroundColor,
            value.into(),
        ));
        self
    }

    /// Like [`Self::background_color`] but updates on patch when the value can change.
    pub fn patch_background_color(mut self, value: u32) -> Self {
        self.patch_attrs.push((
            ArkUINodeAttributeType::BackgroundColor,
            value.into(),
        ));
        self
    }

    pub fn gesture(mut self, mutator: impl FnOnce(&mut T) -> ArkUIResult<()> + 'static) -> Self
    where
        T: ArkUIGesture,
    {
        self.effects.push(wrap_effect(mutator));
        self
    }
}

impl<T> ComponentElement<T>
where
    T: ArkUICommonFontAttribute + 'static,
{
    pub fn font_size(mut self, value: f32) -> Self {
        self.init_attrs
            .push((ArkUINodeAttributeType::FontSize, value.into()));
        self
    }

    pub fn patch_font_size(mut self, value: f32) -> Self {
        self.patch_attrs
            .push((ArkUINodeAttributeType::FontSize, value.into()));
        self
    }
}

impl<T> ComponentElement<T>
where
    T: ArkUIEvent + 'static,
{
    pub fn on_click(mut self, callback: impl Fn() + 'static) -> Self {
        self.effects.push(wrap_effect::<T>(move |node| {
            node.on_click(move || callback());
            Ok(())
        }));
        self
    }

    pub fn on_event(
        mut self,
        event_type: NodeEventType,
        callback: impl Fn(&ArkEvent) + 'static,
    ) -> Self {
        self.effects.push(wrap_effect::<T>(move |node| {
            node.on_event(event_type, move |event| callback(event));
            Ok(())
        }));
        self
    }

    pub fn on_event_no_param(
        mut self,
        event_type: NodeEventType,
        callback: impl Fn() + 'static,
    ) -> Self {
        self.effects.push(wrap_effect::<T>(move |node| {
            node.on_event_no_param(event_type, move || callback());
            Ok(())
        }));
        self
    }

    pub fn on_custom_event(
        mut self,
        event_type: NodeCustomEventType,
        callback: impl Fn(&NodeCustomEvent) + 'static,
    ) -> Self {
        self.effects.push(wrap_effect::<T>(move |node| {
            node.on_custom_event(event_type, move |event| callback(event));
            Ok(())
        }));
        self
    }

    pub fn on_custom_measure(mut self, callback: impl Fn(&NodeCustomEvent) + 'static) -> Self {
        self.effects.push(wrap_effect::<T>(move |node| {
            node.on_custom_measure(move |event| callback(event));
            Ok(())
        }));
        self
    }

    pub fn on_custom_layout(mut self, callback: impl Fn(&NodeCustomEvent) + 'static) -> Self {
        self.effects.push(wrap_effect::<T>(move |node| {
            node.on_custom_layout(move |event| callback(event));
            Ok(())
        }));
        self
    }

    pub fn on_custom_draw(mut self, callback: impl Fn(&NodeCustomEvent) + 'static) -> Self {
        self.effects.push(wrap_effect::<T>(move |node| {
            node.on_custom_draw(move |event| callback(event));
            Ok(())
        }));
        self
    }

    pub fn on_custom_foreground_draw(
        mut self,
        callback: impl Fn(&NodeCustomEvent) + 'static,
    ) -> Self {
        self.effects.push(wrap_effect::<T>(move |node| {
            node.on_custom_foreground_draw(move |event| callback(event));
            Ok(())
        }));
        self
    }

    pub fn on_custom_overlay_draw(mut self, callback: impl Fn(&NodeCustomEvent) + 'static) -> Self {
        self.effects.push(wrap_effect::<T>(move |node| {
            node.on_custom_overlay_draw(move |event| callback(event));
            Ok(())
        }));
        self
    }
}

pub(crate) fn rebuild_children_tail<T, I>(
    parent: &mut T,
    mounted_children: &mut Vec<MountedElement>,
    start: usize,
    next_children: I,
) -> ArkUIResult<()>
where
    T: ArkUICommonAttribute,
    I: IntoIterator<Item = Element>,
{
    while mounted_children.len() > start {
        let removed_meta = mounted_children.remove(start);
        if let Some(removed) = parent.remove_child(start)? {
            removed.borrow_mut().dispose()?;
        }
        removed_meta.cleanup_recursive();
    }

    for child in next_children {
        let (child_node, child_meta) = child.mount()?;
        let mut child_cleanup_node = child_node.clone();
        if let Err(error) = parent.add_child(child_node) {
            logging::error(format!(
                "tree error: failed to append rebuilt child {} to {}: {error}",
                child_meta.name,
                type_name::<T>()
            ));
            let _ = child_cleanup_node.dispose();
            child_meta.cleanup_recursive();
            return Err(error);
        }
        mounted_children.push(child_meta);
    }

    Ok(())
}

/// Key-based child reconciliation aligned with Solid.js / Vue 3 strategy.
///
/// Instead of falling back to full rebuild on the first mismatch, this
/// algorithm uses a (key, kind) map to find reusable old children at any
/// position, minimizing unnecessary unmount/remount cycles.
pub(crate) fn reconcile_children<T>(
    parent: &mut T,
    mounted_children: &mut Vec<MountedElement>,
    next_children: Vec<Element>,
) -> ArkUIResult<()>
where
    T: ArkUICommonAttribute,
{
    use std::collections::HashMap;

    type ChildKey = (TypeId, Option<String>);

    fn child_key(m: &MountedElement) -> ChildKey {
        (m.kind, m.key.clone())
    }

    fn element_key(e: &Element) -> ChildKey {
        (e.kind(), e.key().map(str::to_owned))
    }

    let next_len = next_children.len();
    let old_len = mounted_children.len();

    if next_len == 0 {
        rebuild_children_tail(parent, mounted_children, 0, std::iter::empty())?;
        return Ok(());
    }

    // Fast path: linear prefix match (identical to previous algorithm for
    // the common case where children don't reorder).
    let mut prefix = 0;
    let next_children: Vec<Element> = next_children.into_iter().collect();
    while prefix < old_len && prefix < next_len {
        if element_key(&next_children[prefix]) != child_key(&mounted_children[prefix]) {
            break;
        }
        prefix += 1;
    }

    // Patch the prefix in-place.
    let mut patched_next: Vec<Option<Element>> = next_children.into_iter().map(Some).collect();
    for i in 0..prefix {
        let next_child = patched_next[i].take().unwrap();
        let child_handle = parent.borrow_mut().children()[i].clone();
        let mut child_node = child_handle.borrow_mut();
        if let Err(error) = next_child.patch(&mut child_node, &mut mounted_children[i]) {
            logging::error(format!(
                "tree error: failed to patch child {} at index {} under {}: {error}",
                mounted_children[i].name,
                i,
                type_name::<T>()
            ));
            return Err(error);
        }
    }

    // Everything matched linearly — handle tail additions/removals.
    if prefix == old_len && prefix == next_len {
        return Ok(());
    }

    if prefix == old_len {
        for i in prefix..next_len {
            let next_child = patched_next[i].take().unwrap();
            let (child_node, child_meta) = next_child.mount()?;
            let mut child_cleanup_node = child_node.clone();
            if let Err(error) = parent.add_child(child_node) {
                logging::error(format!(
                    "tree error: failed to append child {} to {}: {error}",
                    child_meta.name,
                    type_name::<T>()
                ));
                let _ = child_cleanup_node.dispose();
                child_meta.cleanup_recursive();
                return Err(error);
            }
            mounted_children.push(child_meta);
        }
        return Ok(());
    }

    if prefix == next_len {
        rebuild_children_tail(parent, mounted_children, prefix, std::iter::empty())?;
        return Ok(());
    }

    // Build a lookup map for remaining old children by (kind, key).
    // Tracks available indices so duplicates are handled.
    let mut old_map: HashMap<ChildKey, Vec<usize>> = HashMap::new();
    for i in prefix..old_len {
        old_map
            .entry(child_key(&mounted_children[i]))
            .or_default()
            .push(i);
    }

    // matched_old[i] = true if old child i was reused.
    let mut matched_old = vec![false; old_len];

    // Result buffer: new mounted list from `prefix` onward.
    struct PendingChild {
        node: ArkUINode,
        meta: MountedElement,
    }

    let mut new_children_buf: Vec<PendingChild> = Vec::with_capacity(next_len - prefix);

    for i in prefix..next_len {
        let next_child = patched_next[i].take().unwrap();
        let key = element_key(&next_child);

        // Try to find a reusable old child.
        let reused_idx = old_map
            .get_mut(&key)
            .and_then(|indices| indices.pop());

        if let Some(old_idx) = reused_idx {
            matched_old[old_idx] = true;
            let child_handle = parent.borrow_mut().children()[old_idx].clone();
            let mut child_node = child_handle.borrow_mut();
            if let Err(error) =
                next_child.patch(&mut child_node, &mut mounted_children[old_idx])
            {
                logging::error(format!(
                    "tree error: failed to patch reused child {} from {} to {} under {}: {error}",
                    mounted_children[old_idx].name,
                    old_idx,
                    i,
                    type_name::<T>()
                ));
                return Err(error);
            }
            new_children_buf.push(PendingChild {
                node: child_node.clone(),
                meta: MountedElement::new(
                    mounted_children[old_idx].kind,
                    mounted_children[old_idx].name,
                    mounted_children[old_idx].key.clone(),
                    std::mem::take(&mut mounted_children[old_idx].cleanups),
                    std::mem::take(&mut mounted_children[old_idx].children),
                ),
            });
            if let Some(state) = mounted_children[old_idx].state.take() {
                new_children_buf.last_mut().unwrap().meta.set_state(state);
            }
        } else {
            let (child_node, child_meta) = match next_child.mount() {
                Ok(result) => result,
                Err(error) => {
                    logging::error(format!(
                        "tree error: failed to mount new child at {} under {}: {error}",
                        i,
                        type_name::<T>()
                    ));
                    return Err(error);
                }
            };
            new_children_buf.push(PendingChild {
                node: child_node,
                meta: child_meta,
            });
        }
    }

    // Remove all old children from `prefix` onward (in reverse to keep indices stable).
    for i in (prefix..old_len).rev() {
        if let Some(removed) = parent.remove_child(i)? {
            removed.borrow_mut().dispose()?;
        }
        if !matched_old[i] {
            let old_meta = mounted_children.remove(i);
            old_meta.cleanup_recursive();
        } else {
            mounted_children.remove(i);
        }
    }

    // Insert new children starting at `prefix`.
    for (offset, pending) in new_children_buf.into_iter().enumerate() {
        let insert_idx = prefix + offset;
        let mut child_cleanup_node = pending.node.clone();
        let attach_result = if insert_idx >= parent.borrow_mut().children().len() {
            parent.add_child(pending.node)
        } else {
            parent.insert_child(pending.node, insert_idx)
        };
        if let Err(error) = attach_result {
            logging::error(format!(
                "tree error: failed to insert child {} at {} under {}: {error}",
                pending.meta.name,
                insert_idx,
                type_name::<T>()
            ));
            let _ = child_cleanup_node.dispose();
            pending.meta.cleanup_recursive();
            return Err(error);
        }
        mounted_children.insert(insert_idx, pending.meta);
    }

    Ok(())
}

impl<T> ViewNode for ComponentElement<T>
where
    T: HostComponent,
{
    fn kind(&self) -> TypeId {
        T::kind()
    }

    fn key(&self) -> Option<&str> {
        self.key.as_deref()
    }

    fn mount(self: Box<Self>) -> ArkUIResult<(ArkUINode, MountedElement)> {
        let Self {
            constructor,
            key,
            init_attrs,
            patch_attrs,
            effects,
            children,
        } = *self;

        let mut node = constructor().map_err(|error| {
            logging::error(format!(
                "mount error: failed to construct {}: {error}",
                T::name()
            ));
            error
        })?;

        apply_attr_list(&mut node, init_attrs, "mount", T::name());
        apply_attr_list(&mut node, patch_attrs, "mount", T::name());

        let self_cleanups = match apply_effects(&mut node, effects) {
            Ok(cleanups) => cleanups,
            Err(error) => {
                let mut raw: ArkUINode = node.into();
                let _ = raw.dispose();
                return Err(error);
            }
        };

        let mut mounted_children: Vec<MountedElement> = Vec::with_capacity(children.len());
        for child in children {
            let (child_node, child_meta) = match child.mount() {
                Ok(result) => result,
                Err(error) => {
                    logging::error(format!(
                        "mount error: failed to mount child under {}: {error}",
                        T::name()
                    ));
                    let mut raw: ArkUINode = node.into();
                    let _ = raw.dispose();
                    for mounted_child in mounted_children {
                        mounted_child.cleanup_recursive();
                    }
                    run_cleanups(self_cleanups);
                    return Err(error);
                }
            };

            let mut child_cleanup_node = child_node.clone();
            if let Err(error) = node.add_child(child_node) {
                logging::error(format!(
                    "tree error: failed to add child {} to {} during mount: {error}",
                    child_meta.name,
                    T::name()
                ));
                let _ = child_cleanup_node.dispose();
                child_meta.cleanup_recursive();
                let mut raw: ArkUINode = node.into();
                let _ = raw.dispose();
                for mounted_child in mounted_children {
                    mounted_child.cleanup_recursive();
                }
                run_cleanups(self_cleanups);
                return Err(error);
            }
            mounted_children.push(child_meta);
        }

        Ok((
            node.into(),
            MountedElement::new(T::kind(), T::name(), key, self_cleanups, mounted_children),
        ))
    }

    fn patch(
        self: Box<Self>,
        node: &mut ArkUINode,
        mounted: &mut MountedElement,
    ) -> ArkUIResult<()> {
        let Self {
            key,
            patch_attrs,
            effects,
            children,
            ..
        } = *self;

        let mut typed = T::from_existing(node.clone());
        apply_attr_list(&mut typed, patch_attrs, "patch", T::name());

        let new_cleanups = apply_effects(&mut typed, effects).map_err(|error| {
            logging::error(format!(
                "patch error: failed to apply effects on {}: {error}",
                T::name()
            ));
            error
        })?;

        if let Err(error) = reconcile_children(&mut typed, &mut mounted.children, children) {
            logging::error(format!(
                "patch error: failed to reconcile children for {}: {error}",
                T::name()
            ));
            run_cleanups(new_cleanups);
            return Err(error);
        }

        *node = typed.into();
        mounted.kind = T::kind();
        mounted.name = T::name();
        mounted.key = key;
        mounted.replace_cleanups(new_cleanups);
        Ok(())
    }
}
