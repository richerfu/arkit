//! Shared event payload types bridging ArkUI native events into dioxus.
//!
//! When an ArkUI event fires, the renderer wraps the payload in an
//! [`ArkEventData`] and forwards it to `Runtime::handle_event`. The listener
//! installed by `impl_event!` downcasts the `Rc<dyn Any>` back to
//! `ArkEventData` and converts it into the specific event data type
//! (e.g. [`ClickData`], [`ChangeData`]).
//!
//! ## Payload carrying
//! Each typed data type ([`ChangeData`], [`ScrollData`], ...) reads its fields
//! out of [`ArkEventData::payload`] via [`ArkEventPayload`]. When the runtime
//! sink does not yet populate the payload (the foundation runtime constructs an
//! empty [`ArkEventData::new`]), the `From` impls yield a default value, so the
//! event still fires and triggers a rerender — only the typed fields are
//! missing. Wiring the native event values through the sink is a runtime-level
//! follow-up.

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
