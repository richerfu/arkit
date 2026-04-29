use super::*;

pub type Element<Message = (), AppTheme = arkit_core::Theme> =
    arkit_core::Element<'static, Message, AppTheme, Renderer>;

#[derive(Debug, Clone, PartialEq)]
pub struct TextLayoutLine {
    pub index: usize,
    pub top: f32,
    pub bottom: f32,
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextLayoutSnapshot {
    pub width: f32,
    pub height: f32,
    pub line_count: usize,
    pub lines: Vec<TextLayoutLine>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Renderer;

pub(super) type Cleanup = Box<dyn FnOnce()>;
pub(super) type MountEffect =
    Box<dyn FnOnce(&mut ArkUINode) -> ArkUIResult<Option<Cleanup>> + 'static>;
pub(super) type AttachEffect =
    Box<dyn FnOnce(&mut ArkUINode) -> ArkUIResult<Option<Cleanup>> + 'static>;
pub(super) type PatchEffect = Box<dyn FnOnce(&mut ArkUINode) -> ArkUIResult<()> + 'static>;
pub(super) type ExitEffect =
    Box<dyn FnOnce(&mut ArkUINode, Cleanup) -> ArkUIResult<Option<Cleanup>> + 'static>;
pub(super) type EventCallback = Rc<dyn Fn(&ArkEvent)>;
pub(super) type UiStateCallback = Rc<dyn Fn(&mut ArkUINode, UiState)>;

pub(super) const DEFAULT_LONG_PRESS_DURATION_MS: i32 = 500;

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

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ScrollOffset {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ScrollViewport {
    pub offset: ScrollOffset,
    pub viewport_size: Size<f32>,
    pub content_size: Size<f32>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ListScrollIndexEvent {
    pub first_index: i32,
    pub last_index: i32,
    pub center_index: i32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ListVisibleContentChangeEvent {
    pub first_index: i32,
    pub start_area: i32,
    pub start_item_index: i32,
    pub last_index: i32,
    pub end_area: i32,
    pub end_item_index: i32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct GridScrollIndexEvent {
    pub first_index: i32,
    pub last_index: i32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct WaterFlowScrollIndexEvent {
    pub start_index: i32,
    pub end_index: i32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct VirtualVisibleRange {
    pub first_index: i32,
    pub last_index: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListStickyStyle {
    None,
    Header,
    Footer,
    Both,
}

impl From<ListStickyStyle> for i32 {
    fn from(value: ListStickyStyle) -> Self {
        match value {
            ListStickyStyle::None => 0,
            ListStickyStyle::Header => 1,
            ListStickyStyle::Footer => 2,
            ListStickyStyle::Both => 3,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VirtualListGroup {
    pub key: String,
    pub item_count: u32,
}

impl VirtualListGroup {
    pub fn new(key: impl Into<String>, item_count: u32) -> Self {
        Self {
            key: key.into(),
            item_count,
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
pub enum FlexDirection {
    Row,
    Column,
    RowReverse,
    ColumnReverse,
}

impl From<FlexDirection> for i32 {
    fn from(value: FlexDirection) -> Self {
        match value {
            FlexDirection::Row => 0,
            FlexDirection::Column => 1,
            FlexDirection::RowReverse => 2,
            FlexDirection::ColumnReverse => 3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlexWrap {
    NoWrap,
    Wrap,
    WrapReverse,
}

impl From<FlexWrap> for i32 {
    fn from(value: FlexWrap) -> Self {
        match value {
            FlexWrap::NoWrap => 0,
            FlexWrap::Wrap => 1,
            FlexWrap::WrapReverse => 2,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FlexOptions {
    pub direction: FlexDirection,
    pub wrap: FlexWrap,
    pub justify_content: JustifyContent,
    pub align_items: ItemAlignment,
    pub align_content: JustifyContent,
}

impl FlexOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn direction(mut self, value: FlexDirection) -> Self {
        self.direction = value;
        self
    }

    pub fn wrap(mut self, value: FlexWrap) -> Self {
        self.wrap = value;
        self
    }

    pub fn justify_content(mut self, value: JustifyContent) -> Self {
        self.justify_content = value;
        self
    }

    pub fn align_items(mut self, value: ItemAlignment) -> Self {
        self.align_items = value;
        self
    }

    pub fn align_content(mut self, value: JustifyContent) -> Self {
        self.align_content = value;
        self
    }

    pub(super) fn attribute_numbers(self) -> Vec<i32> {
        vec![
            i32::from(self.direction),
            i32::from(self.wrap),
            i32::from(self.justify_content),
            i32::from(self.align_items),
            i32::from(self.align_content),
        ]
    }

    pub(super) fn from_attribute_value(value: &ArkUINodeAttributeItem) -> Option<Self> {
        let ArkUINodeAttributeItem::NumberValue(values) = value else {
            return None;
        };

        let mut options = Self::default();
        if let Some(value) = values.first().and_then(attribute_number_as_i32) {
            options.direction = flex_direction_from_i32(value)?;
        }
        if let Some(value) = values.get(1).and_then(attribute_number_as_i32) {
            options.wrap = flex_wrap_from_i32(value)?;
        }
        if let Some(value) = values.get(2).and_then(attribute_number_as_i32) {
            options.justify_content = justify_content_from_i32(value)?;
        }
        if let Some(value) = values.get(3).and_then(attribute_number_as_i32) {
            options.align_items = item_alignment_from_i32(value)?;
        }
        if let Some(value) = values.get(4).and_then(attribute_number_as_i32) {
            options.align_content = justify_content_from_i32(value)?;
        }
        Some(options)
    }
}

impl Default for FlexOptions {
    fn default() -> Self {
        Self {
            direction: FlexDirection::Row,
            wrap: FlexWrap::NoWrap,
            justify_content: JustifyContent::Start,
            align_items: ItemAlignment::Start,
            align_content: JustifyContent::Start,
        }
    }
}

pub(super) fn padding_edges(value: Padding) -> Vec<f32> {
    vec![value.top, value.right, value.bottom, value.left]
}

pub trait EdgeAttributeValue {
    fn edge_values(self) -> Vec<f32>;
}

impl EdgeAttributeValue for f32 {
    fn edge_values(self) -> Vec<f32> {
        vec![self, self, self, self]
    }
}

impl EdgeAttributeValue for [f32; 2] {
    fn edge_values(self) -> Vec<f32> {
        padding_edges(Padding::from(self))
    }
}

impl EdgeAttributeValue for [f32; 4] {
    fn edge_values(self) -> Vec<f32> {
        self.to_vec()
    }
}

impl EdgeAttributeValue for Padding {
    fn edge_values(self) -> Vec<f32> {
        padding_edges(self)
    }
}

pub(super) fn clone_attr_value(value: &ArkUINodeAttributeItem) -> ArkUINodeAttributeItem {
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

fn attribute_number_as_i32(value: &ArkUINodeAttributeNumber) -> Option<i32> {
    match value {
        ArkUINodeAttributeNumber::Float(value) => Some(*value as i32),
        ArkUINodeAttributeNumber::Int(value) => Some(*value),
        ArkUINodeAttributeNumber::Uint(value) => i32::try_from(*value).ok(),
    }
}

fn flex_direction_from_i32(value: i32) -> Option<FlexDirection> {
    match value {
        0 => Some(FlexDirection::Row),
        1 => Some(FlexDirection::Column),
        2 => Some(FlexDirection::RowReverse),
        3 => Some(FlexDirection::ColumnReverse),
        _ => None,
    }
}

fn flex_wrap_from_i32(value: i32) -> Option<FlexWrap> {
    match value {
        0 => Some(FlexWrap::NoWrap),
        1 => Some(FlexWrap::Wrap),
        2 => Some(FlexWrap::WrapReverse),
        _ => None,
    }
}

fn item_alignment_from_i32(value: i32) -> Option<ItemAlignment> {
    match value {
        0 => Some(ItemAlignment::Auto),
        1 => Some(ItemAlignment::Start),
        2 => Some(ItemAlignment::Center),
        3 => Some(ItemAlignment::End),
        4 => Some(ItemAlignment::Stretch),
        5 => Some(ItemAlignment::Baseline),
        _ => None,
    }
}

fn justify_content_from_i32(value: i32) -> Option<JustifyContent> {
    match value {
        1 => Some(JustifyContent::Start),
        2 => Some(JustifyContent::Center),
        3 => Some(JustifyContent::End),
        6 => Some(JustifyContent::SpaceBetween),
        7 => Some(JustifyContent::SpaceAround),
        8 => Some(JustifyContent::SpaceEvenly),
        _ => None,
    }
}
