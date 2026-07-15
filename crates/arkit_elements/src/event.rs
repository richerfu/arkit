//! Shared event payload types bridging ArkUI native events into dioxus.
//!
//! When an ArkUI event fires, the renderer wraps the payload in an
//! [`ArkEventData`] and forwards it to `Runtime::handle_event`. The listener
//! installed by `impl_event!` downcasts the `Rc<dyn Any>` back to
//! `ArkEventData` and converts it into the specific event data type
//! (e.g. [`ClickData`], [`ChangeData`]).
//!
//! ## Payload carrying
//! Each typed data type ([`ChangeData`], [`ScrollData`], ...) reads fields from
//! [`ArkEventData::payload`]. Native payloads are populated at the ArkUI event
//! boundary; `None` remains the explicit representation for payload-free
//! events such as refresh and submit-fire.

/// Semantic identity of an ArkUI event listener.
///
/// Dioxus passes listener names to renderers after removing the leading
/// `on`, while the public RSX surface intentionally accepts both compact
/// (`onclick`) and readable (`on_click`) spellings. Keeping this classification
/// in the event-owning crate prevents the renderer and runtime from developing
/// different alias tables.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ArkEventKind {
    Click,
    LongPress,
    Change,
    Submit,
    Scroll,
    SwiperChange,
    Refresh,
    AreaChange,
    Focus,
    Blur,
    Hover,
    HoverMove,
    DragStart,
    DragMove,
    DragEnd,
    DragLeave,
    DragEnter,
    Touch,
}

impl ArkEventKind {
    /// Whether Dioxus should propagate this event through ancestor listeners.
    pub const fn bubbles(self) -> bool {
        matches!(self, Self::Click | Self::LongPress | Self::Touch)
    }
}

/// Classify either an RSX attribute name or the stripped listener name passed
/// by Dioxus to a renderer.
pub fn classify_event_name(name: &str) -> Option<ArkEventKind> {
    let name = name.strip_prefix("on").unwrap_or(name);
    let name = name.strip_prefix('_').unwrap_or(name);
    Some(match name {
        "click" | "press" => ArkEventKind::Click,
        "longpress" | "long_press" => ArkEventKind::LongPress,
        "change" | "input" | "toggle" => ArkEventKind::Change,
        "submit" => ArkEventKind::Submit,
        "scroll" => ArkEventKind::Scroll,
        "swiperchange" | "swiper_change" | "swiper" => ArkEventKind::SwiperChange,
        "refresh" => ArkEventKind::Refresh,
        "area" | "area_change" | "layout" | "layout_change" => ArkEventKind::AreaChange,
        "focus" => ArkEventKind::Focus,
        "blur" => ArkEventKind::Blur,
        "hover" => ArkEventKind::Hover,
        "hovermove" | "hover_move" => ArkEventKind::HoverMove,
        "dragstart" | "drag_start" => ArkEventKind::DragStart,
        "dragmove" | "drag_move" => ArkEventKind::DragMove,
        "dragend" | "drag_end" => ArkEventKind::DragEnd,
        "dragleave" | "drag_leave" => ArkEventKind::DragLeave,
        "dragenter" | "drag_enter" => ArkEventKind::DragEnter,
        "touch" => ArkEventKind::Touch,
        _ => return None,
    })
}

/// Typed payload carried by an [`ArkEventData`].
///
/// Variants mirror the value shapes the ArkUI native event API exposes
/// (`i32_value`, `f32_value`, `string_value`, ...). The renderer populates this
/// when it registers a native event callback; the typed `*Data` structs read
/// from it in their `From<&ArkEventData>` impls.
#[derive(Default, Clone, Debug)]
pub enum ArkEventPayload {
    /// No payload (click, refresh, submit-fire, etc.).
    #[default]
    None,
    /// A boolean value (checkbox/toggle/radio checked state).
    Bool(bool),
    /// A single float (slider value, refresh offset).
    Float(f32),
    /// A single integer (swiper index, submit return code).
    Int(i32),
    /// A string value (text input/area change).
    String(String),
    /// A scroll-index payload (list/grid/water-flow visible range).
    ScrollIndex(ScrollIndexPayload),
    /// A physical scroll offset payload (generic Scroll and scroll observers).
    ScrollOffset(ScrollOffsetPayload),
    /// Layout frame payload for element-bound area/layout change events.
    Layout(LayoutPayload),
    /// Pointer payload for click/touch/drag events.
    Pointer(PointerPayload),
}

/// Scroll-index payload shared by list/grid/water-flow `on_scroll` events.
#[derive(Default, Clone, Copy, Debug)]
pub struct ScrollIndexPayload {
    /// First visible item index.
    pub first: i32,
    /// Last visible item index.
    pub last: i32,
    /// Center index (list only; 0 for grid/water-flow).
    pub center: i32,
}

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct ScrollOffsetPayload {
    pub x: f32,
    pub y: f32,
}

/// Element layout frame in physical pixels, relative to the window.
#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct LayoutPayload {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl LayoutPayload {
    pub fn is_measured(self) -> bool {
        self.width > 0.0 && self.height > 0.0
    }
}

/// Pointer coordinates and target bounds carried by ArkUI input events.
#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct PointerPayload {
    /// Native touch phase. Click/drag events that do not expose a touch phase
    /// use [`PointerAction::Unknown`].
    pub action: PointerAction,
    /// Monotonic platform event timestamp in nanoseconds, or zero when the
    /// native event does not expose one.
    pub timestamp_nanos: u64,
    /// Stable native contact identifier for the current pointer.
    pub pointer_id: i32,
    /// Pressed mouse/stylus buttons represented as a platform bit mask.
    pub buttons: u64,
    /// Contact pressure in the platform-normalized range when available.
    pub pressure: f32,
    /// Pointer x relative to the event target.
    pub x: f32,
    /// Pointer y relative to the event target.
    pub y: f32,
    /// Pointer x relative to the window.
    pub window_x: f32,
    /// Pointer y relative to the window.
    pub window_y: f32,
    /// Event target x relative to the window/global display.
    pub target_x: f32,
    /// Event target y relative to the window/global display.
    pub target_y: f32,
    /// Event target width.
    pub target_width: f32,
    /// Event target height.
    pub target_height: f32,
}

/// Platform-neutral pointer phase used by touch-capable components.
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum PointerAction {
    #[default]
    Unknown,
    Cancel,
    Down,
    Move,
    Up,
}

impl PointerPayload {
    pub fn has_target_bounds(self) -> bool {
        self.target_x.is_finite()
            && self.target_y.is_finite()
            && self.target_width.is_finite()
            && self.target_height.is_finite()
            && self.target_width > 0.0
            && self.target_height > 0.0
    }

    pub fn has_window_position(self) -> bool {
        self.window_x.is_finite()
            && self.window_y.is_finite()
            && (self.window_x != 0.0 || self.window_y != 0.0)
    }
}

/// Platform event payload carrying an ArkUI native event.
#[derive(Default)]
pub struct ArkEventData {
    /// Typed native event payload. `None` variant when the runtime sink did not
    /// populate it (foundation runtime); the typed `*Data` structs fall back to
    /// defaults in that case.
    pub payload: ArkEventPayload,
}

impl ArkEventData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_payload(payload: ArkEventPayload) -> Self {
        Self { payload }
    }
}

/// Data for a click/press event.
#[derive(Default, Clone, Copy, Debug)]
pub struct ClickData {
    pub pointer: Option<PointerPayload>,
}

impl From<ArkEventData> for ClickData {
    fn from(data: ArkEventData) -> Self {
        Self::from(&data)
    }
}

impl From<&ArkEventData> for ClickData {
    fn from(data: &ArkEventData) -> Self {
        let pointer = match &data.payload {
            ArkEventPayload::Pointer(pointer) => Some(*pointer),
            _ => None,
        };
        ClickData { pointer }
    }
}

/// Data for a hover event. `is_hovering` is true on pointer enter, false on
/// exit.
#[derive(Default, Clone, Copy, Debug)]
pub struct HoverData {
    pub is_hovering: bool,
}

/// Data for native focus and blur events.
#[derive(Default, Clone, Copy, Debug)]
pub struct FocusData {
    pub focused: bool,
}

impl From<ArkEventData> for FocusData {
    fn from(data: ArkEventData) -> Self {
        Self::from(&data)
    }
}

impl From<&ArkEventData> for FocusData {
    fn from(data: &ArkEventData) -> Self {
        let focused = match &data.payload {
            ArkEventPayload::Bool(focused) => *focused,
            _ => false,
        };
        Self { focused }
    }
}

impl From<ArkEventData> for HoverData {
    fn from(data: ArkEventData) -> Self {
        Self::from(&data)
    }
}

impl From<&ArkEventData> for HoverData {
    fn from(data: &ArkEventData) -> Self {
        let is_hovering = match &data.payload {
            ArkEventPayload::Bool(b) => *b,
            _ => false,
        };
        HoverData { is_hovering }
    }
}

/// Data for a drag / touch event.
#[derive(Default, Clone, Copy, Debug)]
pub struct PointerData {
    pub pointer: Option<PointerPayload>,
}

impl From<ArkEventData> for PointerData {
    fn from(data: ArkEventData) -> Self {
        Self::from(&data)
    }
}

impl From<&ArkEventData> for PointerData {
    fn from(data: &ArkEventData) -> Self {
        let pointer = match &data.payload {
            ArkEventPayload::Pointer(pointer) => Some(*pointer),
            _ => None,
        };
        PointerData { pointer }
    }
}

/// Data for a value-change event (checkbox/toggle/radio/slider/text input).
///
/// `bool_value`/`float_value`/`string_value` are populated depending on the
/// source component; the others remain at their defaults.
#[derive(Default, Clone, Debug)]
pub struct ChangeData {
    /// Checked state for checkbox/toggle/radio.
    pub bool_value: bool,
    /// Numeric value for slider/progress.
    pub float_value: f32,
    /// Text value for text input/area.
    pub string_value: String,
}

impl From<ArkEventData> for ChangeData {
    fn from(data: ArkEventData) -> Self {
        Self::from(&data)
    }
}

impl From<&ArkEventData> for ChangeData {
    fn from(data: &ArkEventData) -> Self {
        let mut out = ChangeData::default();
        match &data.payload {
            ArkEventPayload::Bool(b) => out.bool_value = *b,
            ArkEventPayload::Float(f) => out.float_value = *f,
            ArkEventPayload::Int(i) => out.float_value = *i as f32,
            ArkEventPayload::String(s) => out.string_value = s.clone(),
            ArkEventPayload::ScrollIndex(_)
            | ArkEventPayload::ScrollOffset(_)
            | ArkEventPayload::Layout(_)
            | ArkEventPayload::Pointer(_)
            | ArkEventPayload::None => {}
        }
        out
    }
}

/// Data for an element-bound area/layout change event.
#[derive(Default, Clone, Copy, Debug)]
pub struct AreaData {
    pub frame: LayoutPayload,
}

impl From<ArkEventData> for AreaData {
    fn from(data: ArkEventData) -> Self {
        Self::from(&data)
    }
}

impl From<&ArkEventData> for AreaData {
    fn from(data: &ArkEventData) -> Self {
        match &data.payload {
            ArkEventPayload::Layout(frame) => AreaData { frame: *frame },
            _ => AreaData::default(),
        }
    }
}

/// Data for a scroll event (list/grid/water-flow/scroll).
#[derive(Default, Clone, Copy, Debug)]
pub struct ScrollData {
    /// First visible item index (scroll-index events).
    pub first_index: i32,
    /// Last visible item index (scroll-index events).
    pub last_index: i32,
    /// Center visible item index (list only).
    pub center_index: i32,
    /// Horizontal scroll offset in physical pixels.
    pub offset_x: f32,
    /// Vertical scroll offset in physical pixels.
    pub offset_y: f32,
    /// Whether this event carried offsets instead of visible indices.
    pub has_offset: bool,
}

impl From<ArkEventData> for ScrollData {
    fn from(data: ArkEventData) -> Self {
        Self::from(&data)
    }
}

impl From<&ArkEventData> for ScrollData {
    fn from(data: &ArkEventData) -> Self {
        match &data.payload {
            ArkEventPayload::ScrollIndex(s) => ScrollData {
                first_index: s.first,
                last_index: s.last,
                center_index: s.center,
                ..ScrollData::default()
            },
            ArkEventPayload::ScrollOffset(offset) => ScrollData {
                offset_x: offset.x,
                offset_y: offset.y,
                has_offset: true,
                ..ScrollData::default()
            },
            _ => ScrollData::default(),
        }
    }
}

/// Data for a submit event (text input/area enter key). Carries the optional
/// return code ArkUI passes to the submit callback.
#[derive(Default, Clone, Copy, Debug)]
pub struct SubmitData {
    pub return_code: i32,
}

impl From<ArkEventData> for SubmitData {
    fn from(data: ArkEventData) -> Self {
        Self::from(&data)
    }
}

impl From<&ArkEventData> for SubmitData {
    fn from(data: &ArkEventData) -> Self {
        let return_code = match &data.payload {
            ArkEventPayload::Int(i) => *i,
            _ => 0,
        };
        SubmitData { return_code }
    }
}

/// Data for a swiper change event. Carries the newly selected index.
#[derive(Default, Clone, Copy, Debug)]
pub struct SwiperChangeData {
    pub index: i32,
}

impl From<ArkEventData> for SwiperChangeData {
    fn from(data: ArkEventData) -> Self {
        Self::from(&data)
    }
}

impl From<&ArkEventData> for SwiperChangeData {
    fn from(data: &ArkEventData) -> Self {
        let index = match &data.payload {
            ArkEventPayload::Int(i) => *i,
            _ => 0,
        };
        SwiperChangeData { index }
    }
}

/// Data for a refresh event (refresh trigger). No payload.
#[derive(Default, Clone, Copy, Debug)]
pub struct RefreshData;

impl From<ArkEventData> for RefreshData {
    fn from(_data: ArkEventData) -> Self {
        RefreshData
    }
}

impl From<&ArkEventData> for RefreshData {
    fn from(_data: &ArkEventData) -> Self {
        RefreshData
    }
}

#[cfg(test)]
mod tests {
    use super::{classify_event_name, ArkEventKind};

    #[test]
    fn event_aliases_share_one_semantic_identity() {
        for name in ["onclick", "click", "on_press", "_press"] {
            assert_eq!(classify_event_name(name), Some(ArkEventKind::Click));
        }
        for name in ["onlongpress", "longpress", "on_long_press", "_long_press"] {
            assert_eq!(classify_event_name(name), Some(ArkEventKind::LongPress));
        }
        for name in ["onfocus", "focus", "on_focus", "_focus"] {
            assert_eq!(classify_event_name(name), Some(ArkEventKind::Focus));
        }
        for name in ["onblur", "blur", "on_blur", "_blur"] {
            assert_eq!(classify_event_name(name), Some(ArkEventKind::Blur));
        }
        assert!(ArkEventKind::Click.bubbles());
        assert!(ArkEventKind::LongPress.bubbles());
        assert!(!ArkEventKind::Change.bubbles());
    }
}
