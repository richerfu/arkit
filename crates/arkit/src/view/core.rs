use std::any::{type_name, TypeId};
use std::mem::{align_of, size_of, ManuallyDrop};

use crate::component::{run_cleanups, Cleanup, MountedElement};
use crate::signal::Signal;
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

use super::element::{Element, ViewNode};

type Mutator<T> = Box<dyn FnOnce(&mut T) -> ArkUIResult<Option<Cleanup>>>;

pub struct ComponentElement<T> {
    constructor: fn() -> ArkUIResult<T>,
    key: Option<String>,
    mutators: Vec<Mutator<T>>,
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

fn wrap_mutator<T>(mutator: impl FnOnce(&mut T) -> ArkUIResult<()> + 'static) -> Mutator<T> {
    Box::new(move |node| {
        mutator(node)?;
        Ok(None)
    })
}

fn apply_mutators<T>(node: &mut T, mutators: Vec<Mutator<T>>) -> ArkUIResult<Vec<Cleanup>> {
    let mut cleanups = Vec::new();
    for mutator in mutators {
        match mutator(node) {
            Ok(Some(cleanup)) => cleanups.push(cleanup),
            Ok(None) => {}
            Err(error) => {
                run_cleanups(cleanups);
                return Err(error);
            }
        }
    }
    Ok(cleanups)
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
            mutators: Vec::new(),
            children: Vec::new(),
        }
    }

    pub fn key(mut self, key: impl Into<String>) -> Self {
        self.key = Some(key.into());
        self
    }

    pub fn with(mut self, mutator: impl FnOnce(&mut T) -> ArkUIResult<()> + 'static) -> Self {
        self.mutators.push(wrap_mutator(mutator));
        self
    }

    pub fn with_cleanup<C>(
        mut self,
        mutator: impl FnOnce(&mut T) -> ArkUIResult<C> + 'static,
    ) -> Self
    where
        C: FnOnce() + 'static,
    {
        self.mutators.push(Box::new(move |node| {
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
    pub fn watch_signal<S>(
        self,
        signal: Signal<S>,
        apply: impl Fn(&mut ArkUINode, S) -> ArkUIResult<()> + 'static,
    ) -> Self
    where
        S: Clone + 'static,
    {
        let apply = std::rc::Rc::new(apply);
        self.native_with_cleanup(move |node| {
            let mut runtime_node = node.borrow_mut().clone();
            apply(&mut runtime_node, signal.get())?;

            let subscription_node = std::rc::Rc::new(std::cell::RefCell::new(runtime_node));
            let subscription_signal = signal.clone();
            let subscription_apply = apply.clone();
            let subscription_node_clone = subscription_node.clone();
            let subscription_id = signal.subscribe(move || {
                let value = subscription_signal.get();
                let mut node = subscription_node_clone.borrow_mut();
                let _ = subscription_apply(&mut node, value);
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

    pub fn attr(
        mut self,
        attr: ArkUINodeAttributeType,
        value: impl Into<ArkUINodeAttributeItem>,
    ) -> Self {
        let value = value.into();
        self.mutators.push(wrap_mutator::<T>(move |node| {
            node.set_attribute(attr, value)
        }));
        self
    }

    pub fn style(
        self,
        attr: ArkUINodeAttributeType,
        value: impl Into<ArkUINodeAttributeItem>,
    ) -> Self {
        self.attr(attr, value)
    }

    pub fn constraint_size(
        mut self,
        min_width: f32,
        max_width: f32,
        min_height: f32,
        max_height: f32,
    ) -> Self {
        self.mutators.push(wrap_mutator::<T>(move |node| {
            node.set_attribute(
                ArkUINodeAttributeType::ConstraintSize,
                vec![min_width, max_width, min_height, max_height].into(),
            )
        }));
        self
    }

    pub fn max_width_constraint(self, value: f32) -> Self {
        self.constraint_size(0.0, value, 0.0, 100000.0)
    }

    pub fn width(mut self, value: f32) -> Self {
        self.mutators
            .push(wrap_mutator::<T>(move |node| node.width(value)));
        self
    }

    pub fn height(mut self, value: f32) -> Self {
        self.mutators
            .push(wrap_mutator::<T>(move |node| node.height(value)));
        self
    }

    pub fn percent_width(mut self, value: f32) -> Self {
        self.mutators
            .push(wrap_mutator::<T>(move |node| node.percent_width(value)));
        self
    }

    pub fn percent_height(mut self, value: f32) -> Self {
        self.mutators
            .push(wrap_mutator::<T>(move |node| node.percent_height(value)));
        self
    }

    pub fn background_color(mut self, value: u32) -> Self {
        self.mutators
            .push(wrap_mutator::<T>(move |node| node.background_color(value)));
        self
    }

    pub fn gesture(mut self, mutator: impl FnOnce(&mut T) -> ArkUIResult<()> + 'static) -> Self
    where
        T: ArkUIGesture,
    {
        self.mutators.push(wrap_mutator(mutator));
        self
    }
}

impl<T> ComponentElement<T>
where
    T: ArkUICommonFontAttribute + 'static,
{
    pub fn font_size(mut self, value: f32) -> Self {
        self.mutators
            .push(wrap_mutator::<T>(move |node| node.font_size(value)));
        self
    }
}

impl<T> ComponentElement<T>
where
    T: ArkUIEvent + 'static,
{
    pub fn on_click(mut self, callback: impl Fn() + 'static) -> Self {
        self.mutators.push(wrap_mutator::<T>(move |node| {
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
        self.mutators.push(wrap_mutator::<T>(move |node| {
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
        self.mutators.push(wrap_mutator::<T>(move |node| {
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
        self.mutators.push(wrap_mutator::<T>(move |node| {
            node.on_custom_event(event_type, move |event| callback(event));
            Ok(())
        }));
        self
    }

    pub fn on_custom_measure(mut self, callback: impl Fn(&NodeCustomEvent) + 'static) -> Self {
        self.mutators.push(wrap_mutator::<T>(move |node| {
            node.on_custom_measure(move |event| callback(event));
            Ok(())
        }));
        self
    }

    pub fn on_custom_layout(mut self, callback: impl Fn(&NodeCustomEvent) + 'static) -> Self {
        self.mutators.push(wrap_mutator::<T>(move |node| {
            node.on_custom_layout(move |event| callback(event));
            Ok(())
        }));
        self
    }

    pub fn on_custom_draw(mut self, callback: impl Fn(&NodeCustomEvent) + 'static) -> Self {
        self.mutators.push(wrap_mutator::<T>(move |node| {
            node.on_custom_draw(move |event| callback(event));
            Ok(())
        }));
        self
    }

    pub fn on_custom_foreground_draw(
        mut self,
        callback: impl Fn(&NodeCustomEvent) + 'static,
    ) -> Self {
        self.mutators.push(wrap_mutator::<T>(move |node| {
            node.on_custom_foreground_draw(move |event| callback(event));
            Ok(())
        }));
        self
    }

    pub fn on_custom_overlay_draw(mut self, callback: impl Fn(&NodeCustomEvent) + 'static) -> Self {
        self.mutators.push(wrap_mutator::<T>(move |node| {
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
            let _ = child_cleanup_node.dispose();
            child_meta.cleanup_recursive();
            return Err(error);
        }
        mounted_children.push(child_meta);
    }

    Ok(())
}

pub(crate) fn reconcile_children<T>(
    parent: &mut T,
    mounted_children: &mut Vec<MountedElement>,
    next_children: Vec<Element>,
) -> ArkUIResult<()>
where
    T: ArkUICommonAttribute,
{
    let mut next_iter = next_children.into_iter();
    let mut index = 0;

    while index < mounted_children.len() {
        let Some(next_child) = next_iter.next() else {
            rebuild_children_tail(parent, mounted_children, index, std::iter::empty())?;
            return Ok(());
        };

        if next_child.kind() != mounted_children[index].kind
            || next_child.key() != mounted_children[index].key.as_deref()
        {
            rebuild_children_tail(
                parent,
                mounted_children,
                index,
                std::iter::once(next_child).chain(next_iter),
            )?;
            return Ok(());
        }

        let child_handle = parent.borrow_mut().children()[index].clone();
        let mut child_node = child_handle.borrow_mut();
        next_child.patch(&mut child_node, &mut mounted_children[index])?;
        index += 1;
    }

    for child in next_iter {
        let (child_node, child_meta) = child.mount()?;
        let mut child_cleanup_node = child_node.clone();
        if let Err(error) = parent.add_child(child_node) {
            let _ = child_cleanup_node.dispose();
            child_meta.cleanup_recursive();
            return Err(error);
        }
        mounted_children.push(child_meta);
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
            mutators,
            children,
        } = *self;

        let mut node = constructor()?;
        let self_cleanups = match apply_mutators(&mut node, mutators) {
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
            mutators,
            children,
            ..
        } = *self;

        let mut typed = T::from_existing(node.clone());
        typed.borrow_mut().reset_events()?;
        let new_cleanups = apply_mutators(&mut typed, mutators)?;
        if let Err(error) = reconcile_children(&mut typed, &mut mounted.children, children) {
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
