use std::any::{type_name, Any, TypeId};
use std::cell::{Cell, RefCell};
use std::mem::{align_of, size_of, ManuallyDrop};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::rc::Rc;

use crate::{Alignment, LayoutFrame, LayoutSize};
use arkit_core::{advanced, Horizontal, Length, Padding, Size, Vertical};
use ohos_arkui_binding::api::attribute_option::ProgressLinearStyleOption;
use ohos_arkui_binding::api::node_custom_event::NodeCustomEvent;
use ohos_arkui_binding::common::attribute::{ArkUINodeAttributeItem, ArkUINodeAttributeNumber};
use ohos_arkui_binding::common::error::ArkUIResult;
use ohos_arkui_binding::common::node::ArkUINode;
use ohos_arkui_binding::component::attribute::{
    ArkUIAttributeBasic, ArkUICommonAttribute, ArkUIEvent, ArkUIGesture,
};
use ohos_arkui_binding::component::built_in_component::{
    Button, CalendarPicker, Checkbox, Column, DatePicker, Image, Progress, Radio, Row, Scroll,
    Slider, Stack, Swiper, Text, TextArea, TextInput, Toggle,
};
use ohos_arkui_binding::event::inner_event::Event as ArkEvent;
use ohos_arkui_binding::gesture::gesture_data::GestureEventData;
use ohos_arkui_binding::gesture::inner_gesture::Gesture;
use ohos_arkui_binding::types::advanced::{
    FontWeight, HorizontalAlignment, NodeCustomEventType, ShadowStyle, VerticalAlignment,
};
use ohos_arkui_binding::types::attribute::ArkUINodeAttributeType;
use ohos_arkui_binding::types::event::NodeEventType;
use ohos_arkui_binding::types::gesture_event::GestureEventAction;
use ohos_arkui_binding::types::text_alignment::TextAlignment;

pub use ohos_arkui_binding::common::attribute::ArkUINodeAttributeItem as AttributeValue;
pub use ohos_arkui_binding::types::attribute::ArkUINodeAttributeType as Attribute;

pub type Element<Message = (), AppTheme = arkit_core::Theme> =
    arkit_core::Element<'static, Message, AppTheme, Renderer>;

#[derive(Debug, Clone, Copy, Default)]
pub struct Renderer;

type Cleanup = Box<dyn FnOnce()>;
type MountEffect = Box<dyn FnOnce(&mut ArkUINode) -> ArkUIResult<Option<Cleanup>> + 'static>;
type AttachEffect = Box<dyn FnOnce(&mut ArkUINode) -> ArkUIResult<Option<Cleanup>> + 'static>;
type PatchEffect = Box<dyn FnOnce(&mut ArkUINode) -> ArkUIResult<()> + 'static>;
type ExitEffect =
    Box<dyn FnOnce(&mut ArkUINode, Cleanup) -> ArkUIResult<Option<Cleanup>> + 'static>;
type EventCallback = Rc<dyn Fn(&ArkEvent)>;
type UiStateCallback = Rc<dyn Fn(&mut ArkUINode, UiState)>;

const DEFAULT_LONG_PRESS_DURATION_MS: i32 = 500;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontStyle {
    Normal,
    Italic,
}

impl From<FontStyle> for i32 {
    fn from(value: FontStyle) -> Self {
        match value {
            FontStyle::Normal => 0,
            FontStyle::Italic => 1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BorderStyle {
    Solid,
    Dashed,
    Dotted,
}

impl From<BorderStyle> for i32 {
    fn from(value: BorderStyle) -> Self {
        match value {
            BorderStyle::Solid => 0,
            BorderStyle::Dashed => 1,
            BorderStyle::Dotted => 2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgressType {
    Linear,
    Ring,
    Eclipse,
    ScaleRing,
    Capsule,
}

impl From<ProgressType> for i32 {
    fn from(value: ProgressType) -> Self {
        match value {
            ProgressType::Linear => 0,
            ProgressType::Ring => 1,
            ProgressType::Eclipse => 2,
            ProgressType::ScaleRing => 3,
            ProgressType::Capsule => 4,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProgressLinearStyle {
    pub stroke_width: f32,
    pub stroke_radius: f32,
    pub scan_effect_enabled: bool,
    pub smooth_effect_enabled: bool,
}

impl ProgressLinearStyle {
    pub fn new(stroke_width: f32, stroke_radius: f32) -> Self {
        Self {
            stroke_width,
            stroke_radius,
            scan_effect_enabled: false,
            smooth_effect_enabled: true,
        }
    }

    pub fn scan_effect_enabled(mut self, value: bool) -> Self {
        self.scan_effect_enabled = value;
        self
    }

    pub fn smooth_effect_enabled(mut self, value: bool) -> Self {
        self.smooth_effect_enabled = value;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemAlignment {
    Auto,
    Start,
    Center,
    End,
    Stretch,
    Baseline,
}

impl From<ItemAlignment> for i32 {
    fn from(value: ItemAlignment) -> Self {
        match value {
            ItemAlignment::Auto => 0,
            ItemAlignment::Start => 1,
            ItemAlignment::Center => 2,
            ItemAlignment::End => 3,
            ItemAlignment::Stretch => 4,
            ItemAlignment::Baseline => 5,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    Visible,
    Hidden,
    None,
}

impl From<Visibility> for i32 {
    fn from(value: Visibility) -> Self {
        match value {
            Visibility::Visible => 0,
            Visibility::Hidden => 1,
            Visibility::None => 2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HitTestBehavior {
    Default,
    Block,
    Transparent,
    None,
    BlockHierarchy,
    BlockDescendants,
}

impl From<HitTestBehavior> for i32 {
    fn from(value: HitTestBehavior) -> Self {
        match value {
            HitTestBehavior::Default => 0,
            HitTestBehavior::Block => 1,
            HitTestBehavior::Transparent => 2,
            HitTestBehavior::None => 3,
            HitTestBehavior::BlockHierarchy => 4,
            HitTestBehavior::BlockDescendants => 5,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonType {
    Normal,
    Capsule,
    Circle,
    RoundedRectangle,
}

impl From<ButtonType> for i32 {
    fn from(value: ButtonType) -> Self {
        match value {
            ButtonType::Normal => 0,
            ButtonType::Capsule => 1,
            ButtonType::Circle => 2,
            ButtonType::RoundedRectangle => 8,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiState(i32);

impl UiState {
    pub const NORMAL: Self = Self(0);
    pub const PRESSED: Self = Self(1);
    pub const FOCUSED: Self = Self(2);
    pub const DISABLED: Self = Self(4);
    pub const SELECTED: Self = Self(8);

    pub const fn from_bits(bits: i32) -> Self {
        Self(bits)
    }

    pub const fn bits(self) -> i32 {
        self.0
    }

    pub const fn contains(self, state: Self) -> bool {
        self.0 & state.0 == state.0
    }
}

impl std::ops::BitOr for UiState {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl From<UiState> for i32 {
    fn from(value: UiState) -> Self {
        value.bits()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectFit {
    Contain,
    Cover,
    Auto,
    Fill,
    ScaleDown,
    None,
    NoneAndAlignTopStart,
    NoneAndAlignTop,
    NoneAndAlignTopEnd,
    NoneAndAlignStart,
    NoneAndAlignCenter,
    NoneAndAlignEnd,
    NoneAndAlignBottomStart,
    NoneAndAlignBottom,
    NoneAndAlignBottomEnd,
    NoneMatrix,
}

impl From<ObjectFit> for i32 {
    fn from(value: ObjectFit) -> Self {
        match value {
            ObjectFit::Contain => 0,
            ObjectFit::Cover => 1,
            ObjectFit::Auto => 2,
            ObjectFit::Fill => 3,
            ObjectFit::ScaleDown => 4,
            ObjectFit::None => 5,
            ObjectFit::NoneAndAlignTopStart => 6,
            ObjectFit::NoneAndAlignTop => 7,
            ObjectFit::NoneAndAlignTopEnd => 8,
            ObjectFit::NoneAndAlignStart => 9,
            ObjectFit::NoneAndAlignCenter => 10,
            ObjectFit::NoneAndAlignEnd => 11,
            ObjectFit::NoneAndAlignBottomStart => 12,
            ObjectFit::NoneAndAlignBottom => 13,
            ObjectFit::NoneAndAlignBottomEnd => 14,
            ObjectFit::NoneMatrix => 15,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JustifyContent {
    Start,
    Center,
    End,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

impl From<JustifyContent> for i32 {
    fn from(value: JustifyContent) -> Self {
        match value {
            JustifyContent::Start => 1,
            JustifyContent::Center => 2,
            JustifyContent::End => 3,
            JustifyContent::SpaceBetween => 6,
            JustifyContent::SpaceAround => 7,
            JustifyContent::SpaceEvenly => 8,
        }
    }
}

fn font_weight_value(value: FontWeight) -> i32 {
    match value {
        FontWeight::W100 => 0,
        FontWeight::W200 => 1,
        FontWeight::W300 => 2,
        FontWeight::W400 => 3,
        FontWeight::W500 => 4,
        FontWeight::W600 => 5,
        FontWeight::W700 => 6,
        FontWeight::W800 => 7,
        FontWeight::W900 => 8,
        FontWeight::Bold => 9,
        FontWeight::Normal => 10,
        FontWeight::Bolder => 11,
        FontWeight::Lighter => 12,
        FontWeight::Medium => 13,
        FontWeight::Regular => 14,
    }
}

fn shadow_style_value(value: ShadowStyle) -> i32 {
    match value {
        ShadowStyle::OuterDefaultXs => 0,
        ShadowStyle::OuterDefaultSm => 1,
        ShadowStyle::OuterDefaultMd => 2,
        ShadowStyle::OuterDefaultLg => 3,
        ShadowStyle::OuterFloatingSm => 4,
        ShadowStyle::OuterFloatingMd => 5,
    }
}

fn apply_progress_linear_style(
    node: &mut ArkUINode,
    style: ProgressLinearStyle,
) -> ArkUIResult<()> {
    let mut option = ProgressLinearStyleOption::new()?;
    option.set_stroke_width(style.stroke_width);
    option.set_stroke_radius(style.stroke_radius);
    option.set_scan_effect_enabled(style.scan_effect_enabled);
    option.set_smooth_effect_enabled(style.smooth_effect_enabled);

    let result = RuntimeNode(node).set_attribute(
        ArkUINodeAttributeType::ProgressLinearStyle,
        (&option).into(),
    );
    option.destroy();
    result
}

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

#[derive(Clone)]
struct EventHandlerSpec {
    event_type: NodeEventType,
    callback: EventCallback,
}

#[derive(Clone)]
struct LongPressHandlerSpec {
    callback: Rc<dyn Fn()>,
}

struct LongPressCallbackContext {
    callback: Rc<dyn Fn()>,
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
    let callback = context.callback.clone();
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
    tree: advanced::widget::Tree,
    render: MountedRenderNode,
}

struct MountedRenderNode {
    tag: TypeId,
    key: Option<String>,
    attrs: Vec<ArkUINodeAttributeType>,
    events: Vec<NodeEventType>,
    mount_effect_count: usize,
    attach_effect_count: usize,
    patch_effect_count: usize,
    has_long_press: bool,
    long_press_cleanup: Option<Cleanup>,
    cleanups: Vec<Cleanup>,
    exit_effect: Option<ExitEffect>,
    exiting_children: Rc<RefCell<Vec<PendingExit>>>,
    pending_patch_attrs: Vec<(ArkUINodeAttributeType, ArkUINodeAttributeItem)>,
    pending_attach_effects: Vec<AttachEffect>,
    pending_patch_effects: Vec<PatchEffect>,
    children: Vec<MountedRenderNode>,
}

struct PendingExit {
    raw_handle: usize,
    alive: Rc<Cell<bool>>,
    mounted: Rc<RefCell<Option<MountedRenderNode>>>,
    effect_cleanup: Rc<RefCell<Option<Cleanup>>>,
}

impl MountedNode {
    fn new(tree: advanced::widget::Tree, render: MountedRenderNode) -> Self {
        Self { tree, render }
    }

    fn tree_mut(&mut self) -> &mut advanced::widget::Tree {
        &mut self.tree
    }

    fn render_mut(&mut self) -> &mut MountedRenderNode {
        &mut self.render
    }

    pub fn cleanup_recursive(self) {
        self.render.cleanup_recursive();
    }
}

impl MountedRenderNode {
    fn new(
        tag: TypeId,
        key: Option<String>,
        attrs: Vec<ArkUINodeAttributeType>,
        events: Vec<NodeEventType>,
        mount_effect_count: usize,
        attach_effect_count: usize,
        patch_effect_count: usize,
        has_long_press: bool,
        long_press_cleanup: Option<Cleanup>,
        cleanups: Vec<Cleanup>,
        exit_effect: Option<ExitEffect>,
        pending_patch_attrs: Vec<(ArkUINodeAttributeType, ArkUINodeAttributeItem)>,
        pending_attach_effects: Vec<AttachEffect>,
        pending_patch_effects: Vec<PatchEffect>,
        children: Vec<MountedRenderNode>,
    ) -> Self {
        Self {
            tag,
            key,
            attrs,
            events,
            mount_effect_count,
            attach_effect_count,
            patch_effect_count,
            has_long_press,
            long_press_cleanup,
            cleanups,
            exit_effect,
            exiting_children: Rc::new(RefCell::new(Vec::new())),
            pending_patch_attrs,
            pending_attach_effects,
            pending_patch_effects,
            children,
        }
    }

    fn cleanup_recursive(self) {
        for child in self.children {
            child.cleanup_recursive();
        }
        let pending_exits = self
            .exiting_children
            .borrow_mut()
            .drain(..)
            .collect::<Vec<_>>();
        for exit in pending_exits {
            exit.alive.set(false);
            if let Some(cleanup) = exit.effect_cleanup.borrow_mut().take() {
                cleanup();
            }
            if let Some(mounted) = exit.mounted.borrow_mut().take() {
                mounted.cleanup_recursive();
            }
        }
        if let Some(cleanup) = self.long_press_cleanup {
            cleanup();
        }
        run_cleanups(self.cleanups);
    }
}

fn padding_edges(value: Padding) -> Vec<f32> {
    vec![value.top, value.right, value.bottom, value.left]
}

fn clone_attr_value(value: &ArkUINodeAttributeItem) -> ArkUINodeAttributeItem {
    match value {
        ArkUINodeAttributeItem::NumberValue(values) => {
            ArkUINodeAttributeItem::NumberValue(values.clone())
        }
        ArkUINodeAttributeItem::String(value) => ArkUINodeAttributeItem::String(value.clone()),
        ArkUINodeAttributeItem::Object(value) => ArkUINodeAttributeItem::Object(*value),
        ArkUINodeAttributeItem::Composite(value) => {
            ArkUINodeAttributeItem::Composite(value.clone())
        }
    }
}

pub struct Node<Message, AppTheme = arkit_core::Theme> {
    kind: NodeKind,
    key: Option<String>,
    init_attrs: Vec<(ArkUINodeAttributeType, ArkUINodeAttributeItem)>,
    patch_attrs: Vec<(ArkUINodeAttributeType, ArkUINodeAttributeItem)>,
    event_handlers: Vec<EventHandlerSpec>,
    long_press_handler: Option<LongPressHandlerSpec>,
    mount_effects: Vec<MountEffect>,
    attach_effects: Vec<AttachEffect>,
    patch_effects: Vec<PatchEffect>,
    exit_effect: Option<ExitEffect>,
    children: Vec<Element<Message, AppTheme>>,
}

impl<Message, AppTheme> Node<Message, AppTheme> {
    fn new(kind: NodeKind) -> Self {
        Self {
            kind,
            key: None,
            init_attrs: Vec::new(),
            patch_attrs: Vec::new(),
            event_handlers: Vec::new(),
            long_press_handler: None,
            mount_effects: Vec::new(),
            attach_effects: Vec::new(),
            patch_effects: Vec::new(),
            exit_effect: None,
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

    pub fn map_descendants(self, mut map: impl FnMut(Self) -> Self) -> Self
    where
        Message: 'static,
        AppTheme: 'static,
    {
        self.map_descendants_with(&mut map)
    }

    fn map_descendants_with(self, map: &mut impl FnMut(Self) -> Self) -> Self
    where
        Message: 'static,
        AppTheme: 'static,
    {
        let Self {
            kind,
            key,
            init_attrs,
            patch_attrs,
            event_handlers,
            long_press_handler,
            mount_effects,
            attach_effects,
            patch_effects,
            exit_effect,
            children,
        } = self;

        let children = children
            .into_iter()
            .map(|child| into_node(child).map_descendants_with(map).into())
            .collect();

        map(Self {
            kind,
            key,
            init_attrs,
            patch_attrs,
            event_handlers,
            long_press_handler,
            mount_effects,
            attach_effects,
            patch_effects,
            exit_effect,
            children,
        })
    }

    pub fn attr(
        mut self,
        attr: ArkUINodeAttributeType,
        value: impl Into<ArkUINodeAttributeItem>,
    ) -> Self {
        self.init_attrs.push((attr, value.into()));
        self
    }

    pub fn patch_attr(
        mut self,
        attr: ArkUINodeAttributeType,
        value: impl Into<ArkUINodeAttributeItem>,
    ) -> Self {
        self.patch_attrs.push((attr, value.into()));
        self
    }

    fn builder_attr(
        mut self,
        attr: ArkUINodeAttributeType,
        value: impl Into<ArkUINodeAttributeItem>,
    ) -> Self {
        let value = value.into();
        self.init_attrs.push((attr, clone_attr_value(&value)));
        self.patch_attrs.push((attr, value));
        self
    }

    pub fn attr_string(&self, attr: ArkUINodeAttributeType) -> Option<&str> {
        self.attr_value(attr).and_then(|value| match value {
            ArkUINodeAttributeItem::String(value) => Some(value.as_str()),
            ArkUINodeAttributeItem::NumberValue(_)
            | ArkUINodeAttributeItem::Object(_)
            | ArkUINodeAttributeItem::Composite(_) => None,
        })
    }

    pub fn attr_f32(&self, attr: ArkUINodeAttributeType) -> Option<f32> {
        self.attr_value(attr).and_then(|value| match value {
            ArkUINodeAttributeItem::NumberValue(values) => {
                values.first().map(|value| match value {
                    ArkUINodeAttributeNumber::Float(value) => *value,
                    ArkUINodeAttributeNumber::Int(value) => *value as f32,
                    ArkUINodeAttributeNumber::Uint(value) => *value as f32,
                })
            }
            ArkUINodeAttributeItem::String(_)
            | ArkUINodeAttributeItem::Object(_)
            | ArkUINodeAttributeItem::Composite(_) => None,
        })
    }

    fn attr_value(&self, attr: ArkUINodeAttributeType) -> Option<&ArkUINodeAttributeItem> {
        self.patch_attrs
            .iter()
            .rev()
            .chain(self.init_attrs.iter().rev())
            .find_map(|(current_attr, value)| (*current_attr == attr).then_some(value))
    }

    pub fn width(mut self, value: impl Into<Length>) -> Self {
        match value.into() {
            Length::Shrink => {}
            Length::Fill => {
                self = self.builder_attr(ArkUINodeAttributeType::WidthPercent, 1.0_f32);
            }
            Length::FillPortion(portion) => {
                self = self.builder_attr(ArkUINodeAttributeType::LayoutWeight, f32::from(portion));
            }
            Length::Fixed(value) => {
                self = self.builder_attr(ArkUINodeAttributeType::Width, value);
            }
        }
        self
    }

    pub fn height(mut self, value: impl Into<Length>) -> Self {
        match value.into() {
            Length::Shrink => {}
            Length::Fill => {
                self = self.builder_attr(ArkUINodeAttributeType::HeightPercent, 1.0_f32);
            }
            Length::FillPortion(portion) => {
                self = self.builder_attr(ArkUINodeAttributeType::LayoutWeight, f32::from(portion));
            }
            Length::Fixed(value) => {
                self = self.builder_attr(ArkUINodeAttributeType::Height, value);
            }
        }
        self
    }

    pub fn percent_width(self, value: f32) -> Self {
        self.builder_attr(ArkUINodeAttributeType::WidthPercent, value)
    }

    pub fn percent_height(self, value: f32) -> Self {
        self.builder_attr(ArkUINodeAttributeType::HeightPercent, value)
    }

    pub fn max_width_constraint(self, value: f32) -> Self {
        self.builder_attr(
            ArkUINodeAttributeType::ConstraintSize,
            vec![0.0_f32, value, 0.0_f32, 100_000.0_f32],
        )
    }

    pub fn constraint_size(
        self,
        min_width: f32,
        max_width: f32,
        min_height: f32,
        max_height: f32,
    ) -> Self {
        let value = vec![min_width, max_width, min_height, max_height];
        self.builder_attr(ArkUINodeAttributeType::ConstraintSize, value)
    }

    pub fn background_color(self, value: u32) -> Self {
        self.builder_attr(ArkUINodeAttributeType::BackgroundColor, value)
    }

    pub fn padding(self, value: impl Into<Padding>) -> Self {
        self.builder_attr(ArkUINodeAttributeType::Padding, padding_edges(value.into()))
    }

    pub fn padding_x(self, value: f32) -> Self {
        self.padding(Padding::symmetric(value, 0.0))
    }

    pub fn padding_y(self, value: f32) -> Self {
        self.padding(Padding::symmetric(0.0, value))
    }

    pub fn margin(self, value: impl Into<Padding>) -> Self {
        self.builder_attr(ArkUINodeAttributeType::Margin, padding_edges(value.into()))
    }

    pub fn margin_x(self, value: f32) -> Self {
        self.margin(Padding::symmetric(value, 0.0))
    }

    pub fn margin_y(self, value: f32) -> Self {
        self.margin(Padding::symmetric(0.0, value))
    }

    pub fn margin_top(self, value: f32) -> Self {
        self.margin(Padding {
            top: value,
            ..Padding::ZERO
        })
    }

    pub fn margin_right(self, value: f32) -> Self {
        self.margin(Padding {
            right: value,
            ..Padding::ZERO
        })
    }

    pub fn margin_bottom(self, value: f32) -> Self {
        self.margin(Padding {
            bottom: value,
            ..Padding::ZERO
        })
    }

    pub fn margin_left(self, value: f32) -> Self {
        self.margin(Padding {
            left: value,
            ..Padding::ZERO
        })
    }

    pub fn foreground_color(self, value: u32) -> Self {
        self.builder_attr(ArkUINodeAttributeType::ForegroundColor, value)
    }

    pub fn font_color(self, value: u32) -> Self {
        self.builder_attr(ArkUINodeAttributeType::FontColor, value)
    }

    pub fn font_weight(self, value: FontWeight) -> Self {
        self.builder_attr(ArkUINodeAttributeType::FontWeight, font_weight_value(value))
    }

    pub fn font_family(self, value: impl Into<String>) -> Self {
        let value = value.into();
        self.builder_attr(ArkUINodeAttributeType::FontFamily, value)
    }

    pub fn font_style(self, value: FontStyle) -> Self {
        self.builder_attr(ArkUINodeAttributeType::FontStyle, i32::from(value))
    }

    pub fn font_size(self, value: f32) -> Self {
        self.builder_attr(ArkUINodeAttributeType::FontSize, value)
    }

    pub fn line_height(self, value: f32) -> Self {
        self.builder_attr(ArkUINodeAttributeType::TextLineHeight, value)
    }

    pub fn text_align(self, value: TextAlignment) -> Self {
        self.builder_attr(ArkUINodeAttributeType::TextAlign, i32::from(value))
    }

    pub fn text_letter_spacing(self, value: f32) -> Self {
        self.builder_attr(ArkUINodeAttributeType::TextLetterSpacing, value)
    }

    pub fn text_decoration(self, value: impl Into<ArkUINodeAttributeItem>) -> Self {
        self.builder_attr(ArkUINodeAttributeType::TextDecoration, value)
    }

    pub fn enabled(self, value: bool) -> Self {
        self.builder_attr(ArkUINodeAttributeType::Enabled, value)
    }

    pub fn opacity(self, value: f32) -> Self {
        self.builder_attr(ArkUINodeAttributeType::Opacity, value)
    }

    pub fn clip(self, value: bool) -> Self {
        self.builder_attr(ArkUINodeAttributeType::Clip, value)
    }

    pub fn focusable(self, value: bool) -> Self {
        self.builder_attr(ArkUINodeAttributeType::Focusable, value)
    }

    pub fn focus_on_touch(self, value: bool) -> Self {
        self.builder_attr(ArkUINodeAttributeType::FocusOnTouch, value)
    }

    pub fn border_radius(self, value: impl Into<Padding>) -> Self {
        self.builder_attr(
            ArkUINodeAttributeType::BorderRadius,
            padding_edges(value.into()),
        )
    }

    pub fn border_width(self, value: impl Into<Padding>) -> Self {
        self.builder_attr(
            ArkUINodeAttributeType::BorderWidth,
            padding_edges(value.into()),
        )
    }

    pub fn border_color(self, value: u32) -> Self {
        self.builder_attr(ArkUINodeAttributeType::BorderColor, vec![value])
    }

    pub fn border_color_all(self, value: u32) -> Self {
        self.builder_attr(
            ArkUINodeAttributeType::BorderColor,
            vec![value, value, value, value],
        )
    }

    pub fn border_style(self, value: BorderStyle) -> Self {
        self.builder_attr(ArkUINodeAttributeType::BorderStyle, i32::from(value))
    }

    pub fn shadow(self, value: ShadowStyle) -> Self {
        self.builder_attr(
            ArkUINodeAttributeType::Shadow,
            vec![shadow_style_value(value)],
        )
    }

    pub fn clear_shadow(self) -> Self {
        self.builder_attr(ArkUINodeAttributeType::Shadow, vec![0_i32])
    }

    pub fn alignment(self, value: Alignment) -> Self {
        self.builder_attr(ArkUINodeAttributeType::Alignment, i32::from(value))
    }

    pub fn align_self(self, value: ItemAlignment) -> Self {
        self.builder_attr(ArkUINodeAttributeType::AlignSelf, i32::from(value))
    }

    pub fn layout_weight(self, value: f32) -> Self {
        self.builder_attr(ArkUINodeAttributeType::LayoutWeight, value)
    }

    pub fn visibility(self, value: Visibility) -> Self {
        self.builder_attr(ArkUINodeAttributeType::Visibility, i32::from(value))
    }

    pub fn hit_test_behavior(self, value: HitTestBehavior) -> Self {
        self.builder_attr(ArkUINodeAttributeType::HitTestBehavior, i32::from(value))
    }

    pub fn button_type(self, value: ButtonType) -> Self {
        self.builder_attr(ArkUINodeAttributeType::ButtonType, i32::from(value))
    }

    pub fn color_blend(self, value: u32) -> Self {
        self.builder_attr(ArkUINodeAttributeType::ColorBlend, value)
    }

    pub fn position(self, x: f32, y: f32) -> Self {
        self.builder_attr(ArkUINodeAttributeType::Position, vec![x, y])
    }

    pub fn z_index(self, value: i32) -> Self {
        self.builder_attr(ArkUINodeAttributeType::ZIndex, value)
    }

    pub fn aspect_ratio(self, value: f32) -> Self {
        self.builder_attr(ArkUINodeAttributeType::AspectRatio, value)
    }

    pub fn image_object_fit(self, value: ObjectFit) -> Self {
        self.builder_attr(ArkUINodeAttributeType::ImageObjectFit, i32::from(value))
    }

    pub fn progress_color(self, value: u32) -> Self {
        self.builder_attr(ArkUINodeAttributeType::ProgressColor, value)
    }

    pub fn progress_type(self, value: ProgressType) -> Self {
        self.builder_attr(ArkUINodeAttributeType::ProgressType, i32::from(value))
    }

    pub fn progress_linear_style(mut self, value: ProgressLinearStyle) -> Self {
        self.patch_effects.push(Box::new(move |node| {
            apply_progress_linear_style(node, value)
        }));
        self.mount_effects.push(Box::new(move |node| {
            apply_progress_linear_style(node, value)?;
            Ok(None)
        }));
        self
    }

    pub fn toggle_selected_color(self, value: u32) -> Self {
        self.builder_attr(ArkUINodeAttributeType::ToggleSelectedColor, value)
    }

    pub fn toggle_unselected_color(self, value: u32) -> Self {
        self.builder_attr(ArkUINodeAttributeType::ToggleUnselectedColor, value)
    }

    pub fn toggle_switch_point_color(self, value: u32) -> Self {
        self.builder_attr(ArkUINodeAttributeType::ToggleSwitchPointColor, value)
    }

    pub fn justify_content(self, value: JustifyContent) -> Self {
        let value = i32::from(value);
        match self.kind {
            NodeKind::Column => self
                .attr(ArkUINodeAttributeType::ColumnJustifyContent, value)
                .patch_attr(ArkUINodeAttributeType::ColumnJustifyContent, value),
            NodeKind::Row => self
                .attr(ArkUINodeAttributeType::RowJustifyContent, value)
                .patch_attr(ArkUINodeAttributeType::RowJustifyContent, value),
            _ => self,
        }
    }

    pub fn justify_content_start(self) -> Self {
        self.justify_content(JustifyContent::Start)
    }

    pub fn justify_content_center(self) -> Self {
        self.justify_content(JustifyContent::Center)
    }

    pub fn justify_content_end(self) -> Self {
        self.justify_content(JustifyContent::End)
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
                    Vertical::Top => JustifyContent::Start,
                    Vertical::Center => JustifyContent::Center,
                    Vertical::Bottom => JustifyContent::End,
                };
                self.justify_content(justify)
            }
            _ => self,
        }
    }

    pub fn align_items_start(self) -> Self {
        match self.kind {
            NodeKind::Column => self
                .attr(
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
                .attr(
                    ArkUINodeAttributeType::ColumnAlignItems,
                    HorizontalAlignment::Center as i32,
                )
                .patch_attr(
                    ArkUINodeAttributeType::ColumnAlignItems,
                    HorizontalAlignment::Center as i32,
                ),
            NodeKind::Row => self
                .attr(
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
                .attr(
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
                .attr(
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
                .attr(
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
        self.attr(ArkUINodeAttributeType::ButtonLabel, label.clone())
            .patch_attr(ArkUINodeAttributeType::ButtonLabel, label)
    }

    pub fn content(self, content: impl Into<String>) -> Self {
        let content = content.into();
        self.attr(ArkUINodeAttributeType::TextContent, content.clone())
            .patch_attr(ArkUINodeAttributeType::TextContent, content)
    }

    pub fn value(self, value: impl Into<String>) -> Self {
        let value = value.into();
        match self.kind {
            NodeKind::TextInput => self
                .attr(ArkUINodeAttributeType::TextInputText, value.clone())
                .patch_attr(ArkUINodeAttributeType::TextInputText, value),
            NodeKind::TextArea => self
                .attr(ArkUINodeAttributeType::TextAreaText, value.clone())
                .patch_attr(ArkUINodeAttributeType::TextAreaText, value),
            _ => self,
        }
    }

    pub fn placeholder(self, value: impl Into<String>) -> Self {
        let value = value.into();
        match self.kind {
            NodeKind::TextInput => self
                .attr(ArkUINodeAttributeType::TextInputPlaceholder, value.clone())
                .patch_attr(ArkUINodeAttributeType::TextInputPlaceholder, value),
            NodeKind::TextArea => self
                .attr(ArkUINodeAttributeType::TextAreaPlaceholder, value.clone())
                .patch_attr(ArkUINodeAttributeType::TextAreaPlaceholder, value),
            _ => self,
        }
    }

    pub fn placeholder_color(self, value: u32) -> Self {
        match self.kind {
            NodeKind::TextInput => self
                .attr(ArkUINodeAttributeType::TextInputPlaceholderColor, value)
                .patch_attr(ArkUINodeAttributeType::TextInputPlaceholderColor, value),
            NodeKind::TextArea => self
                .attr(ArkUINodeAttributeType::TextAreaPlaceholderColor, value)
                .patch_attr(ArkUINodeAttributeType::TextAreaPlaceholderColor, value),
            _ => self,
        }
    }

    pub fn checked(self, value: bool) -> Self {
        match self.kind {
            NodeKind::Checkbox => self
                .attr(ArkUINodeAttributeType::CheckboxSelect, value)
                .patch_attr(ArkUINodeAttributeType::CheckboxSelect, value),
            NodeKind::Toggle => self
                .attr(ArkUINodeAttributeType::ToggleValue, value)
                .patch_attr(ArkUINodeAttributeType::ToggleValue, value),
            NodeKind::Radio => self
                .attr(ArkUINodeAttributeType::RadioChecked, value)
                .patch_attr(ArkUINodeAttributeType::RadioChecked, value),
            _ => self,
        }
    }

    pub fn range(mut self, min: f32, max: f32) -> Self {
        match self.kind {
            NodeKind::Slider => {
                self = self
                    .builder_attr(ArkUINodeAttributeType::SliderMinValue, min)
                    .builder_attr(ArkUINodeAttributeType::SliderMaxValue, max);
            }
            NodeKind::Progress => {
                self = self.builder_attr(ArkUINodeAttributeType::ProgressTotal, max);
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

    pub fn with_exit(
        mut self,
        effect: impl FnOnce(&mut ArkUINode, Cleanup) -> ArkUIResult<()> + 'static,
    ) -> Self {
        self.exit_effect = Some(Box::new(move |node, finish| {
            effect(node, finish)?;
            Ok(None)
        }));
        self
    }

    pub fn with_exit_cleanup<C>(
        mut self,
        effect: impl FnOnce(&mut ArkUINode, Cleanup) -> ArkUIResult<C> + 'static,
    ) -> Self
    where
        C: FnOnce() + 'static,
    {
        self.exit_effect = Some(Box::new(move |node, finish| {
            effect(node, finish).map(|cleanup| Some(Box::new(cleanup) as Cleanup))
        }));
        self
    }

    pub fn native(self, effect: impl FnOnce(&mut ArkUINode) -> ArkUIResult<()> + 'static) -> Self {
        self.with(effect)
    }

    /// Run a callback after the node is attached to the render tree and on
    /// every subsequent patch. Use this to capture a live reference to the
    /// underlying native node that stays valid across re-renders.
    pub fn with_patch(
        mut self,
        effect: impl Fn(&mut ArkUINode) -> ArkUIResult<()> + 'static,
    ) -> Self {
        let shared = Rc::new(effect);
        self.patch_effects.push(Box::new(move |node| shared(node)));
        self
    }

    pub fn with_next_frame(
        mut self,
        effect: impl Fn(&mut ArkUINode) -> ArkUIResult<()> + 'static,
    ) -> Self {
        let shared = Rc::new(effect);
        self.attach_effects.push(Box::new(move |node| {
            let alive = Rc::new(Cell::new(true));
            let callback_alive = alive.clone();
            let callback_effect = shared.clone();
            let callback_node = node.clone();
            node.post_frame_callback(move |_timestamp, _frame| {
                if !callback_alive.get() {
                    return;
                }
                let mut node = callback_node.clone();
                let effect = callback_effect.clone();
                run_guarded_ui_callback(
                    "renderer error: next-frame callback panicked",
                    move || {
                        if let Err(error) = effect(&mut node) {
                            ohos_hilog_binding::error(format!(
                                "renderer error: next-frame callback failed: {error}"
                            ));
                        }
                    },
                );
            })?;

            Ok(Some(Box::new(move || alive.set(false)) as Cleanup))
        }));
        self
    }

    pub fn with_next_idle(
        mut self,
        effect: impl Fn(&mut ArkUINode) -> ArkUIResult<()> + 'static,
    ) -> Self {
        let shared = Rc::new(effect);
        self.attach_effects.push(Box::new(move |node| {
            let alive = Rc::new(Cell::new(true));
            let callback_alive = alive.clone();
            let callback_effect = shared.clone();
            let callback_node = node.clone();
            node.post_idle_callback(move |_time_left, _frame| {
                if !callback_alive.get() {
                    return;
                }
                let mut node = callback_node.clone();
                let effect = callback_effect.clone();
                run_guarded_ui_callback("renderer error: next-idle callback panicked", move || {
                    if let Err(error) = effect(&mut node) {
                        ohos_hilog_binding::error(format!(
                            "renderer error: next-idle callback failed: {error}"
                        ));
                    }
                });
            })?;

            Ok(Some(Box::new(move || alive.set(false)) as Cleanup))
        }));
        self
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
        self.event_handlers.push(EventHandlerSpec {
            event_type: NodeEventType::OnClick,
            callback: Rc::new(move |_| callback()),
        });
        self
    }

    pub fn on_event(
        mut self,
        event_type: NodeEventType,
        callback: impl Fn(&ArkEvent) + 'static,
    ) -> Self {
        self.event_handlers.push(EventHandlerSpec {
            event_type,
            callback: Rc::new(callback),
        });
        self
    }

    pub fn on_event_no_param(
        mut self,
        event_type: NodeEventType,
        callback: impl Fn() + 'static,
    ) -> Self {
        self.event_handlers.push(EventHandlerSpec {
            event_type,
            callback: Rc::new(move |_| callback()),
        });
        self
    }

    pub fn on_supported_ui_states(
        mut self,
        ui_states: UiState,
        exclude_inner: bool,
        callback: impl Fn(&mut ArkUINode, UiState) + 'static,
    ) -> Self {
        let callback = Rc::new(callback) as UiStateCallback;
        let callback_state = Rc::new(RefCell::new(callback.clone()));
        let mount_callback_state = callback_state.clone();
        let ui_state_bits = ui_states.bits();

        self.attach_effects.push(Box::new(move |node| {
            let callback_state = mount_callback_state.clone();
            let callback_node = node.clone();
            node.add_supported_ui_states(
                ui_state_bits,
                move |current_states| {
                    let current = UiState::from_bits(current_states);
                    let handler = callback_state.borrow().clone();
                    let mut node = callback_node.clone();
                    handler(&mut node, current);
                },
                exclude_inner,
            )?;

            let node_handle = node.clone();
            Ok(Some(Box::new(move || {
                let _ = node_handle.remove_supported_ui_states(ui_state_bits);
            }) as Cleanup))
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
        self.long_press_handler = Some(LongPressHandlerSpec {
            callback: Rc::new(callback),
        });
        self
    }

    pub fn on_input(self, handler: impl Fn(String) -> Message + 'static) -> Self
    where
        Message: Send + 'static,
    {
        match self.kind {
            NodeKind::TextInput => self.on_event(NodeEventType::TextInputOnChange, move |event| {
                arkit_runtime::dispatch(handler(event.async_string().unwrap_or_default()));
            }),
            NodeKind::TextArea => self.on_event(NodeEventType::TextAreaOnChange, move |event| {
                arkit_runtime::dispatch(handler(event.async_string().unwrap_or_default()));
            }),
            _ => self,
        }
    }

    pub fn on_submit(self, message: Message) -> Self
    where
        Message: Clone + Send + 'static,
    {
        match self.kind {
            NodeKind::TextInput => self.on_event(NodeEventType::TextInputOnSubmit, move |_| {
                arkit_runtime::dispatch(message.clone());
            }),
            NodeKind::TextArea => self.on_event(NodeEventType::TextAreaOnSubmit, move |_| {
                arkit_runtime::dispatch(message.clone());
            }),
            _ => self,
        }
    }

    pub fn on_toggle(self, handler: impl Fn(bool) -> Message + 'static) -> Self
    where
        Message: Send + 'static,
    {
        match self.kind {
            NodeKind::Checkbox => {
                self.on_event(NodeEventType::CheckboxEventOnChange, move |event| {
                    arkit_runtime::dispatch(handler(event.i32_value(0).unwrap_or_default() != 0));
                })
            }
            NodeKind::Toggle => self.on_event(NodeEventType::ToggleOnChange, move |event| {
                arkit_runtime::dispatch(handler(event.i32_value(0).unwrap_or_default() != 0));
            }),
            NodeKind::Radio => self.on_event(NodeEventType::RadioEventOnChange, move |event| {
                arkit_runtime::dispatch(handler(event.i32_value(0).unwrap_or_default() != 0));
            }),
            _ => self,
        }
    }

    pub fn on_toggle_local(self, handler: impl Fn(bool) + 'static) -> Self {
        let handler = Rc::new(handler) as Rc<dyn Fn(bool)>;
        match self.kind {
            NodeKind::Checkbox => {
                self.on_event(NodeEventType::CheckboxEventOnChange, move |event| {
                    handler(event.i32_value(0).unwrap_or_default() != 0);
                })
            }
            NodeKind::Toggle => self.on_event(NodeEventType::ToggleOnChange, move |event| {
                handler(event.i32_value(0).unwrap_or_default() != 0);
            }),
            NodeKind::Radio => self.on_event(NodeEventType::RadioEventOnChange, move |event| {
                handler(event.i32_value(0).unwrap_or_default() != 0);
            }),
            _ => self,
        }
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

    fn children(&self) -> Vec<advanced::widget::Tree> {
        self.children
            .iter()
            .map(arkit_core::advanced::tree_of)
            .collect()
    }

    fn diff(&self, tree: &mut advanced::widget::Tree)
    where
        Self: 'static,
    {
        tree.set_tag(self.tag());
        sync_child_trees(&self.children, tree);
    }

    fn size_hint(&self) -> Size<Length> {
        Size::new(Length::Shrink, Length::Shrink)
    }

    fn layout(&self) -> arkit_core::layout::Node {
        arkit_core::layout::Node::new(Size::new(0.0, 0.0))
    }

    fn as_any(&self) -> &dyn Any {
        self
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
        .attr(ArkUINodeAttributeType::SliderValue, value)
        .patch_attr(ArkUINodeAttributeType::SliderValue, value)
        .range(min, max)
}

pub fn progress_component<Message, AppTheme>() -> Node<Message, AppTheme> {
    Node::new(NodeKind::Progress)
}

pub fn progress<Message, AppTheme>(value: f32, total: f32) -> Node<Message, AppTheme> {
    progress_component()
        .attr(ArkUINodeAttributeType::ProgressValue, value)
        .patch_attr(ArkUINodeAttributeType::ProgressValue, value)
        .range(0.0, total)
}

pub fn image_component<Message, AppTheme>() -> Node<Message, AppTheme> {
    Node::new(NodeKind::Image)
}

pub fn image<Message, AppTheme>(src: impl Into<String>) -> Node<Message, AppTheme> {
    let src = src.into();
    image_component()
        .attr(ArkUINodeAttributeType::ImageSrc, src.clone())
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

pub(crate) fn read_layout_size(node: &ArkUINode) -> Option<LayoutSize> {
    let size = node.layout_size().ok()?;
    Some(LayoutSize {
        width: size.width as f32,
        height: size.height as f32,
    })
}

pub(crate) fn read_layout_frame(node: &ArkUINode) -> Option<LayoutFrame> {
    let size = read_layout_size(node)?;
    let position = node
        .position_with_translate_in_window()
        .or_else(|_| node.layout_position_in_window())
        .ok()?;
    Some(LayoutFrame {
        x: position.x as f32,
        y: position.y as f32,
        width: size.width,
        height: size.height,
    })
}

pub fn observe_layout_size<Message, AppTheme>(
    element: Element<Message, AppTheme>,
    on_change: impl Fn(LayoutSize) + 'static,
) -> Element<Message, AppTheme>
where
    Message: 'static,
    AppTheme: 'static,
{
    let node_ref = Rc::new(RefCell::new(None::<ArkUINode>));
    let on_change = Rc::new(on_change) as Rc<dyn Fn(LayoutSize)>;

    into_node(element)
        .with_patch({
            let node_ref = node_ref.clone();
            let on_change = on_change.clone();
            move |node| {
                let runtime = node.borrow_mut().clone();
                if let Some(size) = read_layout_size(&runtime) {
                    on_change(size);
                }
                node_ref.replace(Some(runtime));
                Ok(())
            }
        })
        .on_event_no_param(NodeEventType::EventOnAreaChange, move || {
            if let Some(node) = node_ref.borrow().as_ref() {
                if let Some(size) = read_layout_size(node) {
                    on_change(size);
                }
            }
        })
        .into()
}

pub fn observe_layout_frame<Message, AppTheme>(
    element: Element<Message, AppTheme>,
    enabled: bool,
    on_change: impl Fn(LayoutFrame) + 'static,
) -> Element<Message, AppTheme>
where
    Message: 'static,
    AppTheme: 'static,
{
    if !enabled {
        return element;
    }

    let node_ref = Rc::new(RefCell::new(None::<ArkUINode>));
    let on_change = Rc::new(on_change) as Rc<dyn Fn(LayoutFrame)>;

    into_node(element)
        .with_patch({
            let node_ref = node_ref.clone();
            let on_change = on_change.clone();
            move |node| {
                let runtime = node.borrow_mut().clone();
                if let Some(frame) = read_layout_frame(&runtime) {
                    on_change(frame);
                }
                node_ref.replace(Some(runtime));
                Ok(())
            }
        })
        .on_event_no_param(NodeEventType::EventOnAreaChange, move || {
            if let Some(node) = node_ref.borrow().as_ref() {
                if let Some(frame) = read_layout_frame(node) {
                    on_change(frame);
                }
            }
        })
        .into()
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

fn sync_element_tree<Message, AppTheme>(
    element: &Element<Message, AppTheme>,
    tree: &mut advanced::widget::Tree,
) where
    Message: 'static,
    AppTheme: 'static,
{
    let widget = element.as_widget();
    if tree.tag() != widget.tag() {
        *tree = arkit_core::advanced::tree_of(element);
    } else {
        widget.diff(tree);
    }
}

fn sync_child_trees<Message, AppTheme>(
    children: &[Element<Message, AppTheme>],
    tree: &mut advanced::widget::Tree,
) where
    Message: 'static,
    AppTheme: 'static,
{
    let mut next_trees = Vec::with_capacity(children.len());
    let mut existing = std::mem::take(tree.children_mut());

    for child in children {
        let mut child_tree = if existing.is_empty() {
            arkit_core::advanced::tree_of(child)
        } else {
            existing.remove(0)
        };
        sync_element_tree(child, &mut child_tree);
        next_trees.push(child_tree);
    }

    tree.replace_children(next_trees);
}

struct CompiledElement<Message, AppTheme = arkit_core::Theme> {
    body: Element<Message, AppTheme>,
    overlays: Vec<Element<Message, AppTheme>>,
}

fn compile_node<Message, AppTheme>(
    node: Node<Message, AppTheme>,
    tree: &mut advanced::widget::Tree,
    renderer: &Renderer,
) -> CompiledElement<Message, AppTheme>
where
    Message: 'static,
    AppTheme: 'static,
{
    let Node {
        kind,
        key,
        init_attrs,
        patch_attrs,
        event_handlers,
        long_press_handler,
        mount_effects,
        attach_effects,
        patch_effects,
        exit_effect,
        children,
    } = node;

    sync_child_trees(&children, tree);

    let mut compiled_children = Vec::with_capacity(children.len());
    let mut overlays = Vec::new();

    for (child, child_tree) in children.into_iter().zip(tree.children_mut().iter_mut()) {
        let compiled = compile_element(child, child_tree, renderer);
        compiled_children.push(compiled.body);
        overlays.extend(compiled.overlays);
    }

    CompiledElement {
        body: Node {
            kind,
            key,
            init_attrs,
            patch_attrs,
            event_handlers,
            long_press_handler,
            mount_effects,
            attach_effects,
            patch_effects,
            exit_effect,
            children: compiled_children,
        }
        .into(),
        overlays,
    }
}

fn compile_element<Message, AppTheme>(
    element: Element<Message, AppTheme>,
    tree: &mut advanced::widget::Tree,
    renderer: &Renderer,
) -> CompiledElement<Message, AppTheme>
where
    Message: 'static,
    AppTheme: 'static,
{
    sync_element_tree(&element, tree);

    let widget = element.into_widget();
    if widget.as_any().is::<Node<Message, AppTheme>>() {
        let any = widget.into_any();
        let node = any
            .downcast::<Node<Message, AppTheme>>()
            .unwrap_or_else(|_| {
                panic!(
                    "renderer node downcast failed for {}",
                    type_name::<Node<Message, AppTheme>>()
                )
            });
        return compile_node(*node, tree, renderer);
    }

    let body = widget
        .body(tree, renderer)
        .unwrap_or_else(|| panic!("composite widget did not provide a body element"));
    let compiled_body = {
        let body_tree = tree
            .child_mut(0)
            .unwrap_or_else(|| panic!("composite widget body child was not initialized"));
        sync_element_tree(&body, body_tree);
        compile_element(body, body_tree, renderer)
    };

    let overlay = widget.overlay(tree, renderer);
    let mut overlays = compiled_body.overlays;
    if let Some(overlay) = overlay {
        let overlay_tree = tree
            .child_mut(1)
            .unwrap_or_else(|| panic!("composite widget overlay child was not initialized"));
        sync_element_tree(&overlay, overlay_tree);
        let compiled_overlay = compile_element(overlay, overlay_tree, renderer);
        overlays.push(compiled_overlay.body);
        overlays.extend(compiled_overlay.overlays);
    }

    CompiledElement {
        body: compiled_body.body,
        overlays,
    }
}

fn compose_compiled_overlays<Message, AppTheme>(
    compiled: CompiledElement<Message, AppTheme>,
) -> Element<Message, AppTheme>
where
    Message: 'static,
    AppTheme: 'static,
{
    let mut children = vec![compiled.body];

    if !compiled.overlays.is_empty() {
        children.push(
            stack_component::<Message, AppTheme>()
                .percent_width(1.0)
                .percent_height(1.0)
                .attr(ArkUINodeAttributeType::Clip, false)
                .hit_test_behavior(HitTestBehavior::Transparent)
                .attr(
                    ArkUINodeAttributeType::Alignment,
                    i32::from(Alignment::TopStart),
                )
                .attr(ArkUINodeAttributeType::ZIndex, 10_000_i32)
                .children(compiled.overlays)
                .into(),
        );
    }

    stack_component::<Message, AppTheme>()
        .percent_width(1.0)
        .percent_height(1.0)
        .attr(ArkUINodeAttributeType::Clip, false)
        .attr(
            ArkUINodeAttributeType::Alignment,
            i32::from(Alignment::TopStart),
        )
        .children(children)
        .into()
}

fn into_node<Message, AppTheme>(element: Element<Message, AppTheme>) -> Node<Message, AppTheme>
where
    Message: 'static,
    AppTheme: 'static,
{
    let mut tree = arkit_core::advanced::tree_of(&element);
    let compiled = compile_element(element, &mut tree, &Renderer::default());
    let widget = compiled.body.into_widget();
    let any = widget.into_any();
    *any.downcast::<Node<Message, AppTheme>>()
        .unwrap_or_else(|_| {
            panic!(
                "arkit renderer only supports renderer::Node widget bodies in this build; got {}",
                type_name::<Node<Message, AppTheme>>()
            )
        })
}

fn desired_attrs(
    init_attrs: Vec<(ArkUINodeAttributeType, ArkUINodeAttributeItem)>,
    patch_attrs: Vec<(ArkUINodeAttributeType, ArkUINodeAttributeItem)>,
) -> Vec<(ArkUINodeAttributeType, ArkUINodeAttributeItem)> {
    let mut attrs = Vec::new();
    for (attr, value) in init_attrs.into_iter().chain(patch_attrs) {
        if let Some((_, current)) = attrs
            .iter_mut()
            .find(|(current_attr, _)| *current_attr == attr)
        {
            *current = value;
        } else {
            attrs.push((attr, value));
        }
    }
    attrs
}

fn attr_types(
    attrs: &[(ArkUINodeAttributeType, ArkUINodeAttributeItem)],
) -> Vec<ArkUINodeAttributeType> {
    attrs.iter().map(|(attr, _)| *attr).collect()
}

fn desired_attr_types(
    init_attrs: &[(ArkUINodeAttributeType, ArkUINodeAttributeItem)],
    patch_attrs: &[(ArkUINodeAttributeType, ArkUINodeAttributeItem)],
) -> Vec<ArkUINodeAttributeType> {
    let mut attrs = Vec::new();
    for (attr, _) in init_attrs.iter().chain(patch_attrs.iter()) {
        if !attrs.contains(attr) {
            attrs.push(*attr);
        }
    }
    attrs
}

fn reset_stale_attrs(
    node: &mut ArkUINode,
    previous: &[ArkUINodeAttributeType],
    next: &[ArkUINodeAttributeType],
) {
    let runtime = RuntimeNode(node);
    for attr in previous {
        if !next.contains(attr) {
            let _ = runtime.reset_attribute(*attr);
        }
    }
}

fn apply_event_handlers(node: &mut ArkUINode, handlers: &[EventHandlerSpec]) {
    let mut runtime = RuntimeNode(node);
    for (event_type, callbacks) in grouped_event_handlers(handlers) {
        runtime.on_event(event_type, move |event| {
            for callback in &callbacks {
                callback(event);
            }
        });
    }
}

fn grouped_event_handlers(
    handlers: &[EventHandlerSpec],
) -> Vec<(NodeEventType, Vec<EventCallback>)> {
    let mut groups = Vec::<(NodeEventType, Vec<EventCallback>)>::new();
    for handler in handlers {
        if let Some((_, callbacks)) = groups
            .iter_mut()
            .find(|(event_type, _)| *event_type == handler.event_type)
        {
            callbacks.push(handler.callback.clone());
        } else {
            groups.push((handler.event_type, vec![handler.callback.clone()]));
        }
    }
    groups
}

fn event_types(handlers: &[EventHandlerSpec]) -> Vec<NodeEventType> {
    let mut events = Vec::new();
    for handler in handlers {
        if !events.contains(&handler.event_type) {
            events.push(handler.event_type);
        }
    }
    events
}

#[derive(Clone, PartialEq, Eq)]
struct NodeSignature {
    events: Vec<NodeEventType>,
    mount_effect_count: usize,
    attach_effect_count: usize,
    patch_effect_count: usize,
    has_long_press: bool,
}

fn node_signature<Message, AppTheme>(node: &Node<Message, AppTheme>) -> NodeSignature {
    NodeSignature {
        events: event_types(&node.event_handlers),
        mount_effect_count: node.mount_effects.len(),
        attach_effect_count: node.attach_effects.len(),
        patch_effect_count: node.patch_effects.len(),
        has_long_press: node.long_press_handler.is_some(),
    }
}

fn mounted_signature(mounted: &MountedRenderNode) -> NodeSignature {
    NodeSignature {
        events: mounted.events.clone(),
        mount_effect_count: mounted.mount_effect_count,
        attach_effect_count: mounted.attach_effect_count,
        patch_effect_count: mounted.patch_effect_count,
        has_long_press: mounted.has_long_press,
    }
}

fn clear_removed_events(node: &mut ArkUINode, previous: &[NodeEventType], next: &[NodeEventType]) {
    let mut runtime = RuntimeNode(node);
    for event_type in previous {
        if !next.contains(event_type) {
            runtime.on_event(*event_type, |_| {});
        }
    }
}

fn mount_long_press(
    node: &mut ArkUINode,
    handler: &LongPressHandlerSpec,
) -> ArkUIResult<Option<Cleanup>> {
    let gesture = Gesture::create_long_gesture(1, true, DEFAULT_LONG_PRESS_DURATION_MS)?;
    let callback_data = Box::into_raw(Box::new(LongPressCallbackContext {
        callback: handler.callback.clone(),
    }));

    if let Err(error) = gesture.on_gesture_with_data(
        GestureEventAction::Accept | GestureEventAction::Update | GestureEventAction::End,
        callback_data.cast(),
        long_press_gesture_callback,
    ) {
        unsafe {
            drop(Box::from_raw(callback_data));
        }
        let _ = gesture.dispose();
        return Err(error);
    }

    let runtime = RuntimeNode(node);
    if let Err(error) = runtime.add_gesture_ref(&gesture, None, None) {
        unsafe {
            drop(Box::from_raw(callback_data));
        }
        let _ = gesture.dispose();
        return Err(error);
    }

    let mut cleanup_node = node.clone();
    Ok(Some(Box::new(move || {
        let runtime = RuntimeNode(&mut cleanup_node);
        let _ = runtime.remove_gesture(&gesture);
        let _ = gesture.dispose();
        unsafe {
            drop(Box::from_raw(callback_data));
        }
    }) as Cleanup))
}

fn attach_child(parent: &mut ArkUINode, child: ArkUINode) -> ArkUIResult<()> {
    let mut runtime = RuntimeNode(parent);
    runtime.add_child(child)
}

fn insert_child(parent: &mut ArkUINode, child: ArkUINode, index: usize) -> ArkUIResult<()> {
    let mut runtime = RuntimeNode(parent);
    runtime.insert_child(child, index)
}

fn attach_child_at(parent: &mut ArkUINode, child: ArkUINode, index: usize) -> ArkUIResult<()> {
    if index == parent.children().len() {
        attach_child(parent, child)
    } else {
        insert_child(parent, child, index)
    }
}

fn remove_child_by_raw(
    parent: &mut ArkUINode,
    raw_handle: usize,
) -> ArkUIResult<Option<Rc<RefCell<ArkUINode>>>> {
    let index = parent
        .children()
        .iter()
        .position(|child| child.borrow().raw_handle() as usize == raw_handle);

    match index {
        Some(index) => parent.remove_child(index),
        None => Ok(None),
    }
}

fn complete_exiting_child(
    mut parent: ArkUINode,
    raw_handle: usize,
    mounted: Rc<RefCell<Option<MountedRenderNode>>>,
    pending_exits: Rc<RefCell<Vec<PendingExit>>>,
    alive: Rc<Cell<bool>>,
) {
    if !alive.replace(false) {
        return;
    }

    pending_exits
        .borrow_mut()
        .retain(|exit| exit.raw_handle != raw_handle);

    match remove_child_by_raw(&mut parent, raw_handle) {
        Ok(Some(removed)) => {
            let mut removed = removed.borrow().clone();
            let _ = removed.dispose();
        }
        Ok(None) => {}
        Err(error) => {
            ohos_hilog_binding::error(format!(
                "renderer error: failed to remove exiting child: {error}"
            ));
        }
    }

    if let Some(mounted) = mounted.borrow_mut().take() {
        mounted.cleanup_recursive();
    }
}

fn remove_or_exit_child(
    parent: &mut ArkUINode,
    index: usize,
    mut mounted: MountedRenderNode,
    pending_exits: Rc<RefCell<Vec<PendingExit>>>,
) -> ArkUIResult<()> {
    let Some(exit_effect) = mounted.exit_effect.take() else {
        let removed = parent.remove_child(index)?;
        if let Some(removed) = removed {
            let mut removed = removed.borrow().clone();
            let _ = removed.dispose();
        }
        mounted.cleanup_recursive();
        return Ok(());
    };

    let Some(child_handle) = parent.children().get(index).cloned() else {
        mounted.cleanup_recursive();
        return Ok(());
    };

    let raw_handle = child_handle.borrow().raw_handle() as usize;
    let alive = Rc::new(Cell::new(true));
    let mounted_slot = Rc::new(RefCell::new(Some(mounted)));
    let effect_cleanup = Rc::new(RefCell::new(None::<Cleanup>));

    pending_exits.borrow_mut().push(PendingExit {
        raw_handle,
        alive: alive.clone(),
        mounted: mounted_slot.clone(),
        effect_cleanup: effect_cleanup.clone(),
    });

    let finish_parent = parent.clone();
    let finish_mounted = mounted_slot.clone();
    let finish_pending = pending_exits.clone();
    let finish_alive = alive.clone();
    let finish = Box::new(move || {
        complete_exiting_child(
            finish_parent,
            raw_handle,
            finish_mounted,
            finish_pending,
            finish_alive,
        );
    }) as Cleanup;

    let mut child_node = child_handle.borrow_mut();
    if let Err(error) = child_node.set_attribute(
        ArkUINodeAttributeType::HitTestBehavior,
        i32::from(HitTestBehavior::None).into(),
    ) {
        ohos_hilog_binding::error(format!(
            "renderer error: failed to disable exiting child hit test: {error}"
        ));
    }

    match exit_effect(&mut child_node, finish) {
        Ok(cleanup) => {
            effect_cleanup.replace(cleanup);
        }
        Err(error) => {
            ohos_hilog_binding::error(format!("renderer error: exit effect failed: {error}"));
            drop(child_node);
            complete_exiting_child(
                parent.clone(),
                raw_handle,
                mounted_slot,
                pending_exits,
                alive,
            );
        }
    }

    Ok(())
}

fn realize_attached_node(node: &mut ArkUINode, mounted: &mut MountedRenderNode) -> ArkUIResult<()> {
    if !mounted.pending_patch_attrs.is_empty() {
        apply_attr_list(node, std::mem::take(&mut mounted.pending_patch_attrs));
    }

    for effect in std::mem::take(&mut mounted.pending_attach_effects) {
        match effect(node)? {
            Some(cleanup) => mounted.cleanups.push(cleanup),
            None => {}
        }
    }

    for effect in std::mem::take(&mut mounted.pending_patch_effects) {
        effect(node)?;
    }

    for (child_handle, child_mounted) in node.children().iter().zip(mounted.children.iter_mut()) {
        let mut child_node = child_handle.borrow_mut();
        realize_attached_node(&mut child_node, child_mounted)?;
    }

    Ok(())
}

fn mount_node<Message, AppTheme>(
    element: Element<Message, AppTheme>,
) -> ArkUIResult<(ArkUINode, MountedRenderNode)>
where
    Message: Send + 'static,
    AppTheme: 'static,
{
    let Node {
        kind,
        key,
        init_attrs,
        patch_attrs,
        event_handlers,
        long_press_handler,
        mount_effects,
        attach_effects,
        patch_effects,
        exit_effect,
        children,
    } = into_node(element);

    let mut node = create_node(kind)?;
    let init_attr_keys = attr_types(&init_attrs);
    let pending_patch_attrs = desired_attrs(Vec::new(), patch_attrs);
    let final_attr_keys = desired_attr_types(&init_attrs, &pending_patch_attrs);
    let mount_effect_count = mount_effects.len();
    let attach_effect_count = attach_effects.len();
    let patch_effect_count = patch_effects.len();
    let has_long_press = long_press_handler.is_some();
    apply_attr_list(&mut node, init_attrs);

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

    apply_event_handlers(&mut node, &event_handlers);
    let events = event_types(&event_handlers);
    let long_press_cleanup = match long_press_handler.as_ref() {
        Some(handler) => mount_long_press(&mut node, handler)?,
        None => None,
    };

    let mut mounted_children = Vec::with_capacity(children.len());
    for child in children {
        let (child_node, child_mounted) = mount_node(child)?;
        attach_child(&mut node, child_node)?;
        mounted_children.push(child_mounted);
    }

    Ok((
        node,
        MountedRenderNode::new(
            node_type_id(kind),
            key,
            if pending_patch_attrs.is_empty() && patch_effect_count == 0 {
                init_attr_keys
            } else {
                final_attr_keys
            },
            events,
            mount_effect_count,
            attach_effect_count,
            patch_effect_count,
            has_long_press,
            long_press_cleanup,
            cleanups,
            exit_effect,
            pending_patch_attrs,
            attach_effects,
            patch_effects,
            mounted_children,
        ),
    ))
}

fn patch_node<Message, AppTheme>(
    element: Element<Message, AppTheme>,
    node: &mut ArkUINode,
    mounted: &mut MountedRenderNode,
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
        event_handlers,
        long_press_handler,
        mount_effects: _,
        attach_effects: _,
        patch_effects,
        exit_effect,
        children,
    } = into_node(element);

    mounted.tag = node_type_id(kind);
    mounted.key = key;
    mounted.exit_effect = exit_effect;
    let attrs = desired_attrs(init_attrs, patch_attrs);
    let next_attr_types = attr_types(&attrs);
    reset_stale_attrs(node, &mounted.attrs, &next_attr_types);
    apply_attr_list(node, attrs);
    mounted.attrs = next_attr_types;
    for effect in patch_effects {
        effect(node)?;
    }

    let next_events = event_types(&event_handlers);
    clear_removed_events(node, &mounted.events, &next_events);
    apply_event_handlers(node, &event_handlers);
    mounted.events = next_events;

    if let Some(cleanup) = mounted.long_press_cleanup.take() {
        cleanup();
    }
    mounted.long_press_cleanup = match long_press_handler.as_ref() {
        Some(handler) => mount_long_press(node, handler)?,
        None => None,
    };

    reconcile_children(node, mounted, children)
}

fn reconcile_children<Message, AppTheme>(
    parent: &mut ArkUINode,
    mounted: &mut MountedRenderNode,
    next_children: Vec<Element<Message, AppTheme>>,
) -> ArkUIResult<()>
where
    Message: Send + 'static,
    AppTheme: 'static,
{
    type ChildKey = (TypeId, String);

    fn child_key(mounted: &MountedRenderNode) -> Option<ChildKey> {
        mounted.key.clone().map(|key| (mounted.tag, key))
    }

    fn can_reuse<Message, AppTheme>(
        next: &Node<Message, AppTheme>,
        mounted: &MountedRenderNode,
    ) -> bool {
        node_type_id(next.kind) == mounted.tag && node_signature(next) == mounted_signature(mounted)
    }

    let mut next_nodes = Vec::with_capacity(next_children.len());
    for child in next_children {
        let node = into_node(child);
        next_nodes.push(node);
    }

    let next_len = next_nodes.len();
    let pending_exits = mounted.exiting_children.clone();
    let mounted_children = &mut mounted.children;
    let old_len = mounted_children.len();
    let mut prefix = 0;

    while prefix < next_len && prefix < old_len {
        let next_key = match next_nodes[prefix].key.clone() {
            Some(key) => Some((node_type_id(next_nodes[prefix].kind), key)),
            None => None,
        };
        let current_key = child_key(&mounted_children[prefix]);
        let matches = if next_key.is_none() && current_key.is_none() {
            can_reuse(&next_nodes[prefix], &mounted_children[prefix])
        } else {
            next_key == current_key && can_reuse(&next_nodes[prefix], &mounted_children[prefix])
        };
        if !matches {
            break;
        }
        prefix += 1;
    }

    for i in 0..prefix {
        let child_handle = parent.children()[i].clone();
        let mut child_node = child_handle.borrow_mut();
        patch_node(
            next_nodes.remove(0).into(),
            &mut child_node,
            &mut mounted_children[i],
        )?;
    }

    if prefix == old_len && prefix == next_len {
        return Ok(());
    }

    for (offset, child) in next_nodes.into_iter().enumerate() {
        let index = prefix + offset;
        let (child_node, mut child_meta) = mount_node(child.into())?;
        attach_child_at(parent, child_node, index)?;
        if let Some(child_handle) = parent.children().get(index) {
            let mut child_node = child_handle.borrow_mut();
            realize_attached_node(&mut child_node, &mut child_meta)?;
        }
        mounted_children.insert(index, child_meta);
    }

    while mounted_children.len() > next_len {
        let index = mounted_children.len() - 1;
        let mounted = mounted_children.remove(index);
        remove_or_exit_child(parent, index, mounted, pending_exits.clone())?;
    }

    Ok(())
}

pub fn mount<Message, AppTheme>(
    element: Element<Message, AppTheme>,
) -> ArkUIResult<(ArkUINode, MountedNode)>
where
    Message: Send + 'static,
    AppTheme: 'static,
{
    let mut tree = arkit_core::advanced::tree_of(&element);
    let compiled = compile_element(element, &mut tree, &Renderer::default());
    let root = compose_compiled_overlays(compiled);
    let (node, render) = mount_node(root)?;
    Ok((node, MountedNode::new(tree, render)))
}

pub fn realize_attached_mount(node: &mut ArkUINode, mounted: &mut MountedNode) -> ArkUIResult<()> {
    realize_attached_node(node, mounted.render_mut())
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
    sync_element_tree(&element, mounted.tree_mut());
    let compiled = compile_element(element, mounted.tree_mut(), &Renderer::default());
    let root = compose_compiled_overlays(compiled);
    patch_node(root, node, mounted.render_mut())
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
