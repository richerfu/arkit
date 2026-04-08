use std::any::{type_name, Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::mem::{align_of, size_of, ManuallyDrop};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::rc::Rc;

use arkit_core::{advanced, Horizontal, Length, Padding, Size, Vertical};
use ohos_arkui_binding::api::node_custom_event::NodeCustomEvent;
use ohos_arkui_binding::common::attribute::ArkUINodeAttributeItem;
use ohos_arkui_binding::common::error::ArkUIResult;
use ohos_arkui_binding::common::node::ArkUINode;
use ohos_arkui_binding::component::attribute::{
    ArkUIAttributeBasic, ArkUICommonAttribute, ArkUICommonFontAttribute, ArkUIEvent, ArkUIGesture,
};
use ohos_arkui_binding::component::built_in_component::{
    Button, CalendarPicker, Checkbox, Column, DatePicker, Image, Progress, Radio, Row, Scroll,
    Slider, Stack, Swiper, Text, TextArea, TextInput, Toggle,
};
use ohos_arkui_binding::event::inner_event::Event as ArkEvent;
use ohos_arkui_binding::gesture::gesture_data::GestureEventData;
use ohos_arkui_binding::gesture::inner_gesture::Gesture;
use ohos_arkui_binding::types::advanced::{
    HorizontalAlignment, NodeCustomEventType, VerticalAlignment,
};
use ohos_arkui_binding::types::attribute::ArkUINodeAttributeType;
use ohos_arkui_binding::types::event::NodeEventType;
use ohos_arkui_binding::types::gesture_event::GestureEventAction;

pub use ohos_arkui_binding::common::attribute::ArkUINodeAttributeItem as AttributeValue;
pub use ohos_arkui_binding::types::attribute::ArkUINodeAttributeType as Attribute;

pub type Element<Message = (), AppTheme = arkit_core::Theme> =
    arkit_core::Element<'static, Message, AppTheme, Renderer>;

#[derive(Debug, Clone, Copy, Default)]
pub struct Renderer;

type Cleanup = Box<dyn FnOnce()>;
type MountEffect = Box<dyn FnOnce(&mut ArkUINode) -> ArkUIResult<Option<Cleanup>> + 'static>;
type PatchEffect = Box<dyn FnOnce(&mut ArkUINode) -> ArkUIResult<()> + 'static>;

const DEFAULT_LONG_PRESS_DURATION_MS: i32 = 500;
const FLEX_ALIGN_START: i32 = 1;
const FLEX_ALIGN_CENTER: i32 = 2;
const FLEX_ALIGN_END: i32 = 3;

struct RuntimeNode<'a>(&'a mut ArkUINode);

impl ArkUIAttributeBasic for RuntimeNode<'_> {
    fn raw(&self) -> &ArkUINode {
        self.0
    }

    fn borrow_mut(&mut self) -> &mut ArkUINode {
        self.0
    }
}

impl ArkUICommonAttribute for RuntimeNode<'_> {}
impl ArkUIEvent for RuntimeNode<'_> {}
impl ArkUIGesture for RuntimeNode<'_> {}

struct LongPressCallbackContext {
    callback: Rc<RefCell<Rc<dyn Fn()>>>,
}

fn panic_payload_message(payload: &(dyn Any + Send)) -> String {
    if let Some(message) = payload.downcast_ref::<&'static str>() {
        (*message).to_string()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        String::from("non-string panic payload")
    }
}

fn run_guarded_ui_callback(error_label: &'static str, callback: impl FnOnce()) {
    if let Err(payload) = catch_unwind(AssertUnwindSafe(callback)) {
        ohos_hilog_binding::error(format!(
            "{error_label}: {}",
            panic_payload_message(payload.as_ref())
        ));
    }
}

fn long_press_gesture_callback(event: GestureEventData) {
    let Some(data) = event.data else {
        return;
    };
    let context = unsafe { &*(data as *const LongPressCallbackContext) };
    let callback = context.callback.borrow().clone();
    run_guarded_ui_callback(
        "gesture error: on_long_press callback panicked",
        move || (callback.as_ref())(),
    );
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NodeKind {
    Button,
    CalendarPicker,
    Checkbox,
    Column,
    DatePicker,
    Image,
    Progress,
    Radio,
    Row,
    Scroll,
    Slider,
    Stack,
    Swiper,
    Text,
    TextArea,
    TextInput,
    Toggle,
}

pub struct MountedNode {
    tag: TypeId,
    key: Option<String>,
    cleanups: Vec<Cleanup>,
    children: Vec<MountedNode>,
}

impl MountedNode {
    fn new(tag: TypeId, key: Option<String>, cleanups: Vec<Cleanup>, children: Vec<Self>) -> Self {
        Self {
            tag,
            key,
            cleanups,
            children,
        }
    }

    pub fn cleanup_recursive(self) {
        for child in self.children {
            child.cleanup_recursive();
        }
        run_cleanups(self.cleanups);
    }
}

pub struct Node<Message, AppTheme = arkit_core::Theme> {
    kind: NodeKind,
    key: Option<String>,
    init_attrs: Vec<(ArkUINodeAttributeType, ArkUINodeAttributeItem)>,
    patch_attrs: Vec<(ArkUINodeAttributeType, ArkUINodeAttributeItem)>,
    mount_effects: Vec<MountEffect>,
    patch_effects: Vec<PatchEffect>,
    children: Vec<Element<Message, AppTheme>>,
}

impl<Message, AppTheme> Node<Message, AppTheme> {
    fn new(kind: NodeKind) -> Self {
        Self {
            kind,
            key: None,
            init_attrs: Vec::new(),
            patch_attrs: Vec::new(),
            mount_effects: Vec::new(),
            patch_effects: Vec::new(),
            children: Vec::new(),
        }
    }

    pub fn key(mut self, key: impl Into<String>) -> Self {
        self.key = Some(key.into());
        self
    }

    pub fn child(mut self, child: impl Into<Element<Message, AppTheme>>) -> Self {
        self.children.push(child.into());
        self
    }

    pub fn children(mut self, children: Vec<Element<Message, AppTheme>>) -> Self {
        self.children.extend(children);
        self
    }

    pub fn attr(
        mut self,
        attr: ArkUINodeAttributeType,
        value: impl Into<ArkUINodeAttributeItem>,
    ) -> Self {
        self.init_attrs.push((attr, value.into()));
        self
    }

    pub fn style(
        self,
        attr: ArkUINodeAttributeType,
        value: impl Into<ArkUINodeAttributeItem>,
    ) -> Self {
        self.attr(attr, value)
    }

    pub fn patch_attr(
        mut self,
        attr: ArkUINodeAttributeType,
        value: impl Into<ArkUINodeAttributeItem>,
    ) -> Self {
        self.patch_attrs.push((attr, value.into()));
        self
    }

    pub fn width(mut self, value: impl Into<Length>) -> Self {
        match value.into() {
            Length::Shrink => {}
            Length::Fill => {
                self.init_attrs
                    .push((ArkUINodeAttributeType::WidthPercent, 1.0_f32.into()));
            }
            Length::FillPortion(portion) => {
                self.init_attrs.push((
                    ArkUINodeAttributeType::LayoutWeight,
                    f32::from(portion).into(),
                ));
            }
            Length::Fixed(value) => {
                self.init_attrs
                    .push((ArkUINodeAttributeType::Width, value.into()));
            }
        }
        self
    }

    pub fn height(mut self, value: impl Into<Length>) -> Self {
        match value.into() {
            Length::Shrink => {}
            Length::Fill => {
                self.init_attrs
                    .push((ArkUINodeAttributeType::HeightPercent, 1.0_f32.into()));
            }
            Length::FillPortion(portion) => {
                self.init_attrs.push((
                    ArkUINodeAttributeType::LayoutWeight,
                    f32::from(portion).into(),
                ));
            }
            Length::Fixed(value) => {
                self.init_attrs
                    .push((ArkUINodeAttributeType::Height, value.into()));
            }
        }
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

    pub fn max_width_constraint(mut self, value: f32) -> Self {
        self.init_attrs.push((
            ArkUINodeAttributeType::ConstraintSize,
            vec![0.0_f32, value, 0.0_f32, 100_000.0_f32].into(),
        ));
        self
    }

    pub fn constraint_size(
        mut self,
        min_width: f32,
        max_width: f32,
        min_height: f32,
        max_height: f32,
    ) -> Self {
        let value = vec![min_width, max_width, min_height, max_height];
        self.init_attrs
            .push((ArkUINodeAttributeType::ConstraintSize, value.clone().into()));
        self.patch_attrs
            .push((ArkUINodeAttributeType::ConstraintSize, value.into()));
        self
    }

    pub fn background_color(mut self, value: u32) -> Self {
        self.init_attrs
            .push((ArkUINodeAttributeType::BackgroundColor, value.into()));
        self
    }

    pub fn patch_background_color(mut self, value: u32) -> Self {
        self.patch_attrs
            .push((ArkUINodeAttributeType::BackgroundColor, value.into()));
        self
    }

    pub fn padding(mut self, value: impl Into<Padding>) -> Self {
        let padding = value.into();
        self.init_attrs.push((
            ArkUINodeAttributeType::Padding,
            vec![padding.top, padding.right, padding.bottom, padding.left].into(),
        ));
        self
    }

    pub fn padding_x(self, value: f32) -> Self {
        self.padding(Padding::symmetric(value, 0.0))
    }

    pub fn padding_y(self, value: f32) -> Self {
        self.padding(Padding::symmetric(0.0, value))
    }

    pub fn font_size(mut self, value: f32) -> Self {
        self.init_attrs
            .push((ArkUINodeAttributeType::FontSize, value.into()));
        self
    }

    pub fn line_height(mut self, value: f32) -> Self {
        self.init_attrs
            .push((ArkUINodeAttributeType::TextLineHeight, value.into()));
        self.patch_attrs
            .push((ArkUINodeAttributeType::TextLineHeight, value.into()));
        self
    }

    pub fn patch_font_size(mut self, value: f32) -> Self {
        self.patch_attrs
            .push((ArkUINodeAttributeType::FontSize, value.into()));
        self
    }

    pub fn enabled(mut self, value: bool) -> Self {
        self.init_attrs
            .push((ArkUINodeAttributeType::Enabled, value.into()));
        self
    }

    pub fn opacity(mut self, value: f32) -> Self {
        self.init_attrs
            .push((ArkUINodeAttributeType::Opacity, value.into()));
        self
    }

    pub fn clip(mut self, value: bool) -> Self {
        self.init_attrs
            .push((ArkUINodeAttributeType::Clip, value.into()));
        self
    }

    pub fn align_x(self, alignment: Horizontal) -> Self {
        match self.kind {
            NodeKind::Column => match alignment {
                Horizontal::Left => self.align_items_start(),
                Horizontal::Center => self.align_items_center(),
                Horizontal::Right => self.align_items_end(),
            },
            _ => self,
        }
    }

    pub fn align_y(self, alignment: Vertical) -> Self {
        match self.kind {
            NodeKind::Row => match alignment {
                Vertical::Top => self.align_items_top(),
                Vertical::Center => self.align_items_center(),
                Vertical::Bottom => self.align_items_bottom(),
            },
            NodeKind::Column => {
                let justify = match alignment {
                    Vertical::Top => FLEX_ALIGN_START,
                    Vertical::Center => FLEX_ALIGN_CENTER,
                    Vertical::Bottom => FLEX_ALIGN_END,
                };
                self.style(ArkUINodeAttributeType::ColumnJustifyContent, justify)
            }
            _ => self,
        }
    }

    pub fn align_items_start(self) -> Self {
        match self.kind {
            NodeKind::Column => self
                .style(
                    ArkUINodeAttributeType::ColumnAlignItems,
                    HorizontalAlignment::Start as i32,
                )
                .patch_attr(
                    ArkUINodeAttributeType::ColumnAlignItems,
                    HorizontalAlignment::Start as i32,
                ),
            _ => self,
        }
    }

    pub fn align_items_center(self) -> Self {
        match self.kind {
            NodeKind::Column => self
                .style(
                    ArkUINodeAttributeType::ColumnAlignItems,
                    HorizontalAlignment::Center as i32,
                )
                .patch_attr(
                    ArkUINodeAttributeType::ColumnAlignItems,
                    HorizontalAlignment::Center as i32,
                ),
            NodeKind::Row => self
                .style(
                    ArkUINodeAttributeType::RowAlignItems,
                    VerticalAlignment::Center as i32,
                )
                .patch_attr(
                    ArkUINodeAttributeType::RowAlignItems,
                    VerticalAlignment::Center as i32,
                ),
            _ => self,
        }
    }

    pub fn align_items_end(self) -> Self {
        match self.kind {
            NodeKind::Column => self
                .style(
                    ArkUINodeAttributeType::ColumnAlignItems,
                    HorizontalAlignment::End as i32,
                )
                .patch_attr(
                    ArkUINodeAttributeType::ColumnAlignItems,
                    HorizontalAlignment::End as i32,
                ),
            _ => self,
        }
    }

    pub fn align_items_top(self) -> Self {
        match self.kind {
            NodeKind::Row => self
                .style(
                    ArkUINodeAttributeType::RowAlignItems,
                    VerticalAlignment::Top as i32,
                )
                .patch_attr(
                    ArkUINodeAttributeType::RowAlignItems,
                    VerticalAlignment::Top as i32,
                ),
            _ => self,
        }
    }

    pub fn align_items_bottom(self) -> Self {
        match self.kind {
            NodeKind::Row => self
                .style(
                    ArkUINodeAttributeType::RowAlignItems,
                    VerticalAlignment::Bottom as i32,
                )
                .patch_attr(
                    ArkUINodeAttributeType::RowAlignItems,
                    VerticalAlignment::Bottom as i32,
                ),
            _ => self,
        }
    }

    pub fn label(self, label: impl Into<String>) -> Self {
        let label = label.into();
        self.style(ArkUINodeAttributeType::ButtonLabel, label.clone())
            .patch_attr(ArkUINodeAttributeType::ButtonLabel, label)
    }

    pub fn content(self, content: impl Into<String>) -> Self {
        let content = content.into();
        self.style(ArkUINodeAttributeType::TextContent, content.clone())
            .patch_attr(ArkUINodeAttributeType::TextContent, content)
    }

    pub fn value(self, value: impl Into<String>) -> Self {
        let value = value.into();
        match self.kind {
            NodeKind::TextInput => self
                .style(ArkUINodeAttributeType::TextInputText, value.clone())
                .patch_attr(ArkUINodeAttributeType::TextInputText, value),
            NodeKind::TextArea => self
                .style(ArkUINodeAttributeType::TextAreaText, value.clone())
                .patch_attr(ArkUINodeAttributeType::TextAreaText, value),
            _ => self,
        }
    }

    pub fn placeholder(self, value: impl Into<String>) -> Self {
        let value = value.into();
        match self.kind {
            NodeKind::TextInput => self
                .style(ArkUINodeAttributeType::TextInputPlaceholder, value.clone())
                .patch_attr(ArkUINodeAttributeType::TextInputPlaceholder, value),
            NodeKind::TextArea => self
                .style(ArkUINodeAttributeType::TextAreaPlaceholder, value.clone())
                .patch_attr(ArkUINodeAttributeType::TextAreaPlaceholder, value),
            _ => self,
        }
    }

    pub fn placeholder_color(self, value: u32) -> Self {
        match self.kind {
            NodeKind::TextInput => self
                .style(ArkUINodeAttributeType::TextInputPlaceholderColor, value)
                .patch_attr(ArkUINodeAttributeType::TextInputPlaceholderColor, value),
            NodeKind::TextArea => self
                .style(ArkUINodeAttributeType::TextAreaPlaceholderColor, value)
                .patch_attr(ArkUINodeAttributeType::TextAreaPlaceholderColor, value),
            _ => self,
        }
    }

    pub fn checked(self, value: bool) -> Self {
        match self.kind {
            NodeKind::Checkbox => self
                .style(ArkUINodeAttributeType::CheckboxSelect, value)
                .patch_attr(ArkUINodeAttributeType::CheckboxSelect, value),
            NodeKind::Toggle => self
                .style(ArkUINodeAttributeType::ToggleValue, value)
                .patch_attr(ArkUINodeAttributeType::ToggleValue, value),
            NodeKind::Radio => self
                .style(ArkUINodeAttributeType::RadioChecked, value)
                .patch_attr(ArkUINodeAttributeType::RadioChecked, value),
            _ => self,
        }
    }

    pub fn range(mut self, min: f32, max: f32) -> Self {
        match self.kind {
            NodeKind::Slider => {
                self.init_attrs
                    .push((ArkUINodeAttributeType::SliderMinValue, min.into()));
                self.init_attrs
                    .push((ArkUINodeAttributeType::SliderMaxValue, max.into()));
                self.patch_attrs
                    .push((ArkUINodeAttributeType::SliderMinValue, min.into()));
                self.patch_attrs
                    .push((ArkUINodeAttributeType::SliderMaxValue, max.into()));
            }
            NodeKind::Progress => {
                self.init_attrs
                    .push((ArkUINodeAttributeType::ProgressTotal, max.into()));
                self.patch_attrs
                    .push((ArkUINodeAttributeType::ProgressTotal, max.into()));
            }
            _ => {}
        }
        self
    }

    pub fn with(
        mut self,
        effect: impl FnOnce(&mut ArkUINode) -> ArkUIResult<()> + 'static,
    ) -> Self {
        self.mount_effects.push(Box::new(move |node| {
            effect(node)?;
            Ok(None)
        }));
        self
    }

    pub fn with_cleanup<C>(
        mut self,
        effect: impl FnOnce(&mut ArkUINode) -> ArkUIResult<C> + 'static,
    ) -> Self
    where
        C: FnOnce() + 'static,
    {
        self.mount_effects.push(Box::new(move |node| {
            effect(node).map(|cleanup| Some(Box::new(cleanup) as Cleanup))
        }));
        self
    }

    pub fn native(self, effect: impl FnOnce(&mut ArkUINode) -> ArkUIResult<()> + 'static) -> Self {
        self.with(effect)
    }

    pub fn native_with_cleanup<C>(
        self,
        effect: impl FnOnce(&mut ArkUINode) -> ArkUIResult<C> + 'static,
    ) -> Self
    where
        C: FnOnce() + 'static,
    {
        self.with_cleanup(effect)
    }

    pub fn on_press(self, message: Message) -> Self
    where
        Message: Clone + Send + 'static,
    {
        self.on_click(move || arkit_runtime::dispatch(message.clone()))
    }

    pub fn on_click(mut self, callback: impl Fn() + 'static) -> Self {
        let callback = Rc::new(callback) as Rc<dyn Fn()>;
        let callback_state = Rc::new(RefCell::new(callback.clone()));
        let mount_callback_state = callback_state.clone();
        self.mount_effects.push(Box::new(move |node| {
            let callback_state = mount_callback_state.clone();
            let mut node = RuntimeNode(node);
            node.on_click(move || {
                let callback = callback_state.borrow().clone();
                callback();
            });
            Ok(None)
        }));
        self.patch_effects.push(Box::new(move |_node| {
            callback_state.replace(callback);
            Ok(())
        }));
        self
    }

    pub fn on_event(
        mut self,
        event_type: NodeEventType,
        callback: impl Fn(&ArkEvent) + 'static,
    ) -> Self {
        let callback = Rc::new(callback) as Rc<dyn Fn(&ArkEvent)>;
        let callback_state = Rc::new(RefCell::new(callback.clone()));
        let mount_callback_state = callback_state.clone();
        self.mount_effects.push(Box::new(move |node| {
            let callback_state = mount_callback_state.clone();
            let mut node = RuntimeNode(node);
            node.on_event(event_type, move |event| {
                let callback = callback_state.borrow().clone();
                callback(event);
            });
            Ok(None)
        }));
        self.patch_effects.push(Box::new(move |_node| {
            callback_state.replace(callback);
            Ok(())
        }));
        self
    }

    pub fn on_event_no_param(
        mut self,
        event_type: NodeEventType,
        callback: impl Fn() + 'static,
    ) -> Self {
        let callback = Rc::new(callback) as Rc<dyn Fn()>;
        let callback_state = Rc::new(RefCell::new(callback.clone()));
        let mount_callback_state = callback_state.clone();
        self.mount_effects.push(Box::new(move |node| {
            let callback_state = mount_callback_state.clone();
            let mut node = RuntimeNode(node);
            node.on_event_no_param(event_type, move || {
                let callback = callback_state.borrow().clone();
                callback();
            });
            Ok(None)
        }));
        self.patch_effects.push(Box::new(move |_node| {
            callback_state.replace(callback);
            Ok(())
        }));
        self
    }

    pub fn on_custom_event(
        mut self,
        event_type: NodeCustomEventType,
        callback: impl Fn(&NodeCustomEvent) + 'static,
    ) -> Self {
        let callback = Rc::new(callback) as Rc<dyn Fn(&NodeCustomEvent)>;
        let callback_state = Rc::new(RefCell::new(callback.clone()));
        let mount_callback_state = callback_state.clone();
        self.mount_effects.push(Box::new(move |node| {
            let callback_state = mount_callback_state.clone();
            let mut node = RuntimeNode(node);
            node.on_custom_event(event_type, move |event| {
                let callback = callback_state.borrow().clone();
                callback(event);
            });
            Ok(None)
        }));
        self.patch_effects.push(Box::new(move |_node| {
            callback_state.replace(callback);
            Ok(())
        }));
        self
    }

    pub fn on_long_press_message(self, message: Message) -> Self
    where
        Message: Clone + Send + 'static,
    {
        self.on_long_press(move || arkit_runtime::dispatch(message.clone()))
    }

    pub fn on_long_press(mut self, callback: impl Fn() + 'static) -> Self {
        let callback = Rc::new(callback) as Rc<dyn Fn()>;
        let callback_state = Rc::new(RefCell::new(callback.clone()));
        let mount_callback_state = callback_state.clone();
        self.mount_effects.push(Box::new(move |node| {
            let gesture = Gesture::create_long_gesture(1, false, DEFAULT_LONG_PRESS_DURATION_MS)?;
            let callback_data = Box::into_raw(Box::new(LongPressCallbackContext {
                callback: mount_callback_state.clone(),
            }));

            if let Err(error) = gesture.on_gesture_with_data(
                GestureEventAction::Accept,
                callback_data.cast(),
                long_press_gesture_callback,
            ) {
                unsafe {
                    drop(Box::from_raw(callback_data));
                }
                let _ = gesture.dispose();
                return Err(error);
            }

            let node = RuntimeNode(node);
            if let Err(error) = node.add_gesture_ref(&gesture, None, None) {
                unsafe {
                    drop(Box::from_raw(callback_data));
                }
                let _ = gesture.dispose();
                return Err(error);
            }

            Ok(Some(Box::new(move || {
                let _ = gesture.dispose();
                unsafe {
                    drop(Box::from_raw(callback_data));
                }
            }) as Cleanup))
        }));
        self.patch_effects.push(Box::new(move |_node| {
            callback_state.replace(callback);
            Ok(())
        }));
        self
    }

    pub fn on_input(self, handler: impl Fn(String) -> Message + 'static) -> Self
    where
        Message: Send + 'static,
    {
        let kind = self.kind;
        self.with(move |node| {
            match kind {
                NodeKind::TextInput => {
                    let mut input: TextInput = wrap_component(node.clone());
                    input.on_text_input_change(move |value| {
                        arkit_runtime::dispatch(handler(value));
                    });
                }
                NodeKind::TextArea => {
                    let mut area: TextArea = wrap_component(node.clone());
                    area.on_text_area_change(move |value| {
                        arkit_runtime::dispatch(handler(value));
                    });
                }
                _ => {}
            }
            Ok(())
        })
    }

    pub fn on_submit(self, message: Message) -> Self
    where
        Message: Clone + Send + 'static,
    {
        let kind = self.kind;
        self.with(move |node| {
            match kind {
                NodeKind::TextInput => {
                    let mut input: TextInput = wrap_component(node.clone());
                    input.on_text_input_submit(move |_| {
                        arkit_runtime::dispatch(message.clone());
                    });
                }
                NodeKind::TextArea => {
                    let mut area: TextArea = wrap_component(node.clone());
                    area.on_text_area_submit(move |_| {
                        arkit_runtime::dispatch(message.clone());
                    });
                }
                _ => {}
            }
            Ok(())
        })
    }

    pub fn on_toggle(self, handler: impl Fn(bool) -> Message + 'static) -> Self
    where
        Message: Send + 'static,
    {
        let kind = self.kind;
        self.with(move |node| {
            match kind {
                NodeKind::Checkbox => {
                    let mut checkbox: Checkbox = wrap_component(node.clone());
                    checkbox.on_checkbox_change(move |value| {
                        arkit_runtime::dispatch(handler(value));
                    });
                }
                NodeKind::Toggle => {
                    let mut toggle: Toggle = wrap_component(node.clone());
                    toggle.on_toggle_change(move |value| {
                        arkit_runtime::dispatch(handler(value));
                    });
                }
                NodeKind::Radio => {
                    let mut radio: Radio = wrap_component(node.clone());
                    radio.on_radio_change(move |value| {
                        arkit_runtime::dispatch(handler(value));
                    });
                }
                _ => {}
            }
            Ok(())
        })
    }

    pub fn on_toggle_local(self, handler: impl Fn(bool) + 'static) -> Self {
        let kind = self.kind;
        let handler = Rc::new(handler) as Rc<dyn Fn(bool)>;
        self.with(move |node| {
            match kind {
                NodeKind::Checkbox => {
                    let mut checkbox: Checkbox = wrap_component(node.clone());
                    let handler = handler.clone();
                    checkbox.on_checkbox_change(move |value| handler(value));
                }
                NodeKind::Toggle => {
                    let mut toggle: Toggle = wrap_component(node.clone());
                    let handler = handler.clone();
                    toggle.on_toggle_change(move |value| handler(value));
                }
                NodeKind::Radio => {
                    let mut radio: Radio = wrap_component(node.clone());
                    let handler = handler.clone();
                    radio.on_radio_change(move |value| handler(value));
                }
                _ => {}
            }
            Ok(())
        })
    }

    pub fn on_change(self, handler: impl Fn(f32) -> Message + 'static) -> Self
    where
        Message: Send + 'static,
    {
        let kind = self.kind;
        self.with(move |node| {
            if matches!(kind, NodeKind::Slider) {
                let mut slider: Slider = wrap_component(node.clone());
                slider.on_slider_change(move |value| {
                    arkit_runtime::dispatch(handler(value.value));
                });
            }
            Ok(())
        })
    }
}

impl<Message: 'static, AppTheme: 'static> advanced::Widget<Message, AppTheme, Renderer>
    for Node<Message, AppTheme>
{
    fn tag(&self) -> advanced::widget::Tag {
        advanced::widget::Tag::of::<(NodeKind, AppTheme, Message)>()
    }

    fn size_hint(&self) -> Size<Length> {
        Size::new(Length::Shrink, Length::Shrink)
    }

    fn layout(&self) -> arkit_core::layout::Node {
        arkit_core::layout::Node::new(Size::new(0.0, 0.0))
    }

    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

pub fn button_component<Message, AppTheme>() -> Node<Message, AppTheme> {
    Node::new(NodeKind::Button)
}

pub fn button<Message, AppTheme>(label: impl Into<String>) -> Node<Message, AppTheme> {
    button_component().label(label)
}

pub fn text_component<Message, AppTheme>() -> Node<Message, AppTheme> {
    Node::new(NodeKind::Text)
}

pub fn text<Message, AppTheme>(content: impl Into<String>) -> Node<Message, AppTheme> {
    text_component().content(content)
}

pub fn text_input_component<Message, AppTheme>() -> Node<Message, AppTheme> {
    Node::new(NodeKind::TextInput)
}

pub fn text_input<Message, AppTheme>(
    placeholder: impl Into<String>,
    value: impl Into<String>,
) -> Node<Message, AppTheme> {
    text_input_component().placeholder(placeholder).value(value)
}

pub fn text_area_component<Message, AppTheme>() -> Node<Message, AppTheme> {
    Node::new(NodeKind::TextArea)
}

pub fn text_area<Message, AppTheme>(
    placeholder: impl Into<String>,
    value: impl Into<String>,
) -> Node<Message, AppTheme> {
    text_area_component().placeholder(placeholder).value(value)
}

pub fn checkbox_component<Message, AppTheme>() -> Node<Message, AppTheme> {
    Node::new(NodeKind::Checkbox)
}

pub fn checkbox<Message, AppTheme>(checked: bool) -> Node<Message, AppTheme> {
    checkbox_component().checked(checked)
}

pub fn toggle_component<Message, AppTheme>() -> Node<Message, AppTheme> {
    Node::new(NodeKind::Toggle)
}

pub fn toggle<Message, AppTheme>(checked: bool) -> Node<Message, AppTheme> {
    toggle_component().checked(checked)
}

pub fn radio_component<Message, AppTheme>() -> Node<Message, AppTheme> {
    Node::new(NodeKind::Radio)
}

pub fn radio<Message, AppTheme>(checked: bool) -> Node<Message, AppTheme> {
    radio_component().checked(checked)
}

pub fn slider_component<Message, AppTheme>() -> Node<Message, AppTheme> {
    Node::new(NodeKind::Slider)
}

pub fn slider<Message, AppTheme>(value: f32, min: f32, max: f32) -> Node<Message, AppTheme> {
    slider_component()
        .style(ArkUINodeAttributeType::SliderValue, value)
        .patch_attr(ArkUINodeAttributeType::SliderValue, value)
        .range(min, max)
}

pub fn progress_component<Message, AppTheme>() -> Node<Message, AppTheme> {
    Node::new(NodeKind::Progress)
}

pub fn progress<Message, AppTheme>(value: f32, total: f32) -> Node<Message, AppTheme> {
    progress_component()
        .style(ArkUINodeAttributeType::ProgressValue, value)
        .patch_attr(ArkUINodeAttributeType::ProgressValue, value)
        .range(0.0, total)
}

pub fn image_component<Message, AppTheme>() -> Node<Message, AppTheme> {
    Node::new(NodeKind::Image)
}

pub fn image<Message, AppTheme>(src: impl Into<String>) -> Node<Message, AppTheme> {
    let src = src.into();
    image_component()
        .style(ArkUINodeAttributeType::ImageSrc, src.clone())
        .patch_attr(ArkUINodeAttributeType::ImageSrc, src)
}

pub fn column_component<Message, AppTheme>() -> Node<Message, AppTheme> {
    Node::new(NodeKind::Column)
}

pub fn column<Message: 'static, AppTheme: 'static>(
    children: Vec<Element<Message, AppTheme>>,
) -> Element<Message, AppTheme> {
    column_component()
        .percent_width(1.0)
        .percent_height(1.0)
        .children(children)
        .into()
}

pub fn row_component<Message, AppTheme>() -> Node<Message, AppTheme> {
    Node::new(NodeKind::Row)
}

pub fn row<Message: 'static, AppTheme: 'static>(
    children: Vec<Element<Message, AppTheme>>,
) -> Element<Message, AppTheme> {
    row_component().percent_width(1.0).children(children).into()
}

pub fn stack_component<Message, AppTheme>() -> Node<Message, AppTheme> {
    Node::new(NodeKind::Stack)
}

pub fn stack<Message: 'static, AppTheme: 'static>(
    children: Vec<Element<Message, AppTheme>>,
) -> Element<Message, AppTheme> {
    stack_component()
        .percent_width(1.0)
        .percent_height(1.0)
        .children(children)
        .into()
}

pub type Container<Message, AppTheme = arkit_core::Theme> = Node<Message, AppTheme>;

pub fn container<Message: 'static, AppTheme: 'static>(
    child: impl Into<Element<Message, AppTheme>>,
) -> Container<Message, AppTheme> {
    column_component().child(child.into())
}

pub fn scroll_component<Message, AppTheme>() -> Node<Message, AppTheme> {
    Node::new(NodeKind::Scroll)
}

pub fn scroll<Message: 'static, AppTheme: 'static>(
    child: impl Into<Element<Message, AppTheme>>,
) -> Element<Message, AppTheme> {
    scroll_component().child(child.into()).into()
}

pub fn swiper_component<Message, AppTheme>() -> Node<Message, AppTheme> {
    Node::new(NodeKind::Swiper)
}

pub fn swiper<Message, AppTheme>(
    children: Vec<Element<Message, AppTheme>>,
) -> Node<Message, AppTheme> {
    swiper_component().children(children)
}

pub fn calendar_picker_component<Message, AppTheme>() -> Node<Message, AppTheme> {
    Node::new(NodeKind::CalendarPicker)
}

pub fn calendar_picker<Message, AppTheme>() -> Node<Message, AppTheme> {
    calendar_picker_component()
}

pub fn date_picker_component<Message, AppTheme>() -> Node<Message, AppTheme> {
    Node::new(NodeKind::DatePicker)
}

pub fn date_picker<Message, AppTheme>() -> Node<Message, AppTheme> {
    date_picker_component()
}

fn apply_attr_list(
    node: &mut ArkUINode,
    attrs: Vec<(ArkUINodeAttributeType, ArkUINodeAttributeItem)>,
) {
    let runtime = RuntimeNode(node);
    for (attr, value) in attrs {
        if let Err(error) = runtime.set_attribute(attr, value) {
            ohos_hilog_binding::error(format!(
                "renderer error: failed to set attribute {attr:?}: {error}"
            ));
        }
    }
}

fn run_cleanups(mut cleanups: Vec<Cleanup>) {
    while let Some(cleanup) = cleanups.pop() {
        cleanup();
    }
}

fn wrap_component<T>(node: ArkUINode) -> T {
    assert_eq!(size_of::<T>(), size_of::<ArkUINode>());
    assert_eq!(align_of::<T>(), align_of::<ArkUINode>());
    let node = ManuallyDrop::new(node);
    unsafe { std::ptr::read((&*node as *const ArkUINode).cast::<T>()) }
}

fn create_node(kind: NodeKind) -> ArkUIResult<ArkUINode> {
    Ok(match kind {
        NodeKind::Button => Button::new()?.into(),
        NodeKind::CalendarPicker => CalendarPicker::new()?.into(),
        NodeKind::Checkbox => Checkbox::new()?.into(),
        NodeKind::Column => Column::new()?.into(),
        NodeKind::DatePicker => DatePicker::new()?.into(),
        NodeKind::Image => Image::new()?.into(),
        NodeKind::Progress => Progress::new()?.into(),
        NodeKind::Radio => Radio::new()?.into(),
        NodeKind::Row => Row::new()?.into(),
        NodeKind::Scroll => Scroll::new()?.into(),
        NodeKind::Slider => Slider::new()?.into(),
        NodeKind::Stack => Stack::new()?.into(),
        NodeKind::Swiper => Swiper::new()?.into(),
        NodeKind::Text => Text::new()?.into(),
        NodeKind::TextArea => TextArea::new()?.into(),
        NodeKind::TextInput => TextInput::new()?.into(),
        NodeKind::Toggle => Toggle::new()?.into(),
    })
}

fn node_type_id(kind: NodeKind) -> TypeId {
    match kind {
        NodeKind::Button => TypeId::of::<Button>(),
        NodeKind::CalendarPicker => TypeId::of::<CalendarPicker>(),
        NodeKind::Checkbox => TypeId::of::<Checkbox>(),
        NodeKind::Column => TypeId::of::<Column>(),
        NodeKind::DatePicker => TypeId::of::<DatePicker>(),
        NodeKind::Image => TypeId::of::<Image>(),
        NodeKind::Progress => TypeId::of::<Progress>(),
        NodeKind::Radio => TypeId::of::<Radio>(),
        NodeKind::Row => TypeId::of::<Row>(),
        NodeKind::Scroll => TypeId::of::<Scroll>(),
        NodeKind::Slider => TypeId::of::<Slider>(),
        NodeKind::Stack => TypeId::of::<Stack>(),
        NodeKind::Swiper => TypeId::of::<Swiper>(),
        NodeKind::Text => TypeId::of::<Text>(),
        NodeKind::TextArea => TypeId::of::<TextArea>(),
        NodeKind::TextInput => TypeId::of::<TextInput>(),
        NodeKind::Toggle => TypeId::of::<Toggle>(),
    }
}

fn into_node<Message, AppTheme>(element: Element<Message, AppTheme>) -> Node<Message, AppTheme>
where
    Message: Send + 'static,
    AppTheme: 'static,
{
    let widget = element.into_widget();
    let any = widget.into_any();
    *any.downcast::<Node<Message, AppTheme>>()
        .unwrap_or_else(|_| {
            panic!(
                "arkit renderer only supports renderer::Node widgets in this build; got {}",
                type_name::<Node<Message, AppTheme>>()
            )
        })
}

pub fn mount<Message, AppTheme>(
    element: Element<Message, AppTheme>,
) -> ArkUIResult<(ArkUINode, MountedNode)>
where
    Message: Send + 'static,
    AppTheme: 'static,
{
    let Node {
        kind,
        key,
        init_attrs,
        patch_attrs,
        mount_effects,
        patch_effects,
        children,
    } = into_node(element);

    let mut node = create_node(kind)?;
    apply_attr_list(&mut node, init_attrs);
    apply_attr_list(&mut node, patch_attrs);

    let mut cleanups = Vec::new();
    for effect in mount_effects {
        match effect(&mut node) {
            Ok(Some(cleanup)) => cleanups.push(cleanup),
            Ok(None) => {}
            Err(error) => {
                run_cleanups(cleanups);
                let _ = node.dispose();
                return Err(error);
            }
        }
    }

    for effect in patch_effects {
        effect(&mut node)?;
    }

    let mut mounted_children = Vec::with_capacity(children.len());
    let mut runtime = RuntimeNode(&mut node);
    for child in children {
        let (child_node, child_mounted) = mount(child)?;
        runtime.borrow_mut().add_child(child_node)?;
        mounted_children.push(child_mounted);
    }

    Ok((
        node,
        MountedNode::new(node_type_id(kind), key, cleanups, mounted_children),
    ))
}

pub fn patch<Message, AppTheme>(
    element: Element<Message, AppTheme>,
    node: &mut ArkUINode,
    mounted: &mut MountedNode,
) -> ArkUIResult<()>
where
    Message: Send + 'static,
    AppTheme: 'static,
{
    let Node {
        kind,
        key,
        init_attrs,
        patch_attrs,
        mount_effects: _,
        patch_effects,
        children,
    } = into_node(element);

    mounted.tag = node_type_id(kind);
    mounted.key = key;
    apply_attr_list(node, init_attrs);
    apply_attr_list(node, patch_attrs);
    for effect in patch_effects {
        effect(node)?;
    }

    reconcile_children(node, &mut mounted.children, children)
}

fn reconcile_children<Message, AppTheme>(
    parent: &mut ArkUINode,
    mounted_children: &mut Vec<MountedNode>,
    next_children: Vec<Element<Message, AppTheme>>,
) -> ArkUIResult<()>
where
    Message: Send + 'static,
    AppTheme: 'static,
{
    type ChildKey = (TypeId, Option<String>);

    fn child_key(mounted: &MountedNode) -> ChildKey {
        (mounted.tag, mounted.key.clone())
    }

    let mut next_nodes = Vec::with_capacity(next_children.len());
    for child in next_children {
        let node = into_node(child);
        next_nodes.push(node);
    }

    let next_len = next_nodes.len();
    let old_len = mounted_children.len();
    let mut prefix = 0;

    while prefix < next_len && prefix < old_len {
        let next_key = (
            node_type_id(next_nodes[prefix].kind),
            next_nodes[prefix].key.clone(),
        );
        if next_key != child_key(&mounted_children[prefix]) {
            break;
        }
        prefix += 1;
    }

    for i in 0..prefix {
        let child_handle = parent.children()[i].clone();
        let mut child_node = child_handle.borrow_mut();
        patch(
            next_nodes.remove(0).into(),
            &mut child_node,
            &mut mounted_children[i],
        )?;
    }

    let next_nodes = next_nodes;
    if prefix == old_len && prefix == next_len {
        return Ok(());
    }

    if prefix == old_len {
        let mut runtime = RuntimeNode(parent);
        for child in next_nodes {
            let (child_node, child_meta) = mount(child.into())?;
            runtime.borrow_mut().add_child(child_node)?;
            mounted_children.push(child_meta);
        }
        return Ok(());
    }

    if prefix == next_len {
        rebuild_children_tail(parent, mounted_children, prefix)?;
        return Ok(());
    }

    let mut old_map: HashMap<ChildKey, Vec<usize>> = HashMap::new();
    for index in prefix..old_len {
        old_map
            .entry(child_key(&mounted_children[index]))
            .or_default()
            .push(index);
    }

    struct PendingChild {
        node: ArkUINode,
        mounted: MountedNode,
    }

    let mut matched_old = vec![false; old_len];
    let mut new_children = Vec::new();
    for child in next_nodes {
        let key = (node_type_id(child.kind), child.key.clone());
        if let Some(index) = old_map.get_mut(&key).and_then(|indices| indices.pop()) {
            matched_old[index] = true;
            let handle = parent.children()[index].clone();
            let mut child_node = handle.borrow_mut();
            let mut mounted = MountedNode::new(
                mounted_children[index].tag,
                mounted_children[index].key.clone(),
                std::mem::take(&mut mounted_children[index].cleanups),
                std::mem::take(&mut mounted_children[index].children),
            );
            patch(child.into(), &mut child_node, &mut mounted)?;
            new_children.push(PendingChild {
                node: child_node.clone(),
                mounted,
            });
        } else {
            let (node, mounted) = mount(child.into())?;
            new_children.push(PendingChild { node, mounted });
        }
    }

    for index in (prefix..old_len).rev() {
        let removed = parent.remove_child(index)?;
        if matched_old[index] {
            drop(removed);
            mounted_children.remove(index);
        } else {
            if let Some(removed) = removed {
                let mut removed = removed.borrow().clone();
                let _ = removed.dispose();
            }
            let mounted = mounted_children.remove(index);
            mounted.cleanup_recursive();
        }
    }

    let mut runtime = RuntimeNode(parent);
    for (offset, pending) in new_children.into_iter().enumerate() {
        let index = prefix + offset;
        if index >= runtime.borrow_mut().children().len() {
            runtime.borrow_mut().add_child(pending.node)?;
        } else {
            runtime.borrow_mut().insert_child(pending.node, index)?;
        }
        mounted_children.insert(index, pending.mounted);
    }

    Ok(())
}

fn rebuild_children_tail(
    parent: &mut ArkUINode,
    mounted_children: &mut Vec<MountedNode>,
    start: usize,
) -> ArkUIResult<()> {
    while mounted_children.len() > start {
        let mounted = mounted_children.remove(start);
        if let Some(node) = parent.remove_child(start)? {
            let mut node = node.borrow().clone();
            let _ = node.dispose();
        }
        mounted.cleanup_recursive();
    }
    Ok(())
}

impl<Message, AppTheme> From<Node<Message, AppTheme>> for Element<Message, AppTheme>
where
    Message: 'static,
    AppTheme: 'static,
{
    fn from(value: Node<Message, AppTheme>) -> Self {
        arkit_core::Element::new(value)
    }
}
