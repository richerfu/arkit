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
    AccessibilityAction,
    Click,
    LongPress,
    Change,
    Submit,
    Scroll,
    ReachEnd,
    SwiperChange,
    Refresh,
    AreaChange,
    Focus,
    Blur,
    FocusIn,
    FocusOut,
    Hover,
    HoverMove,
    DragStart,
    DragMove,
    DragEnd,
    DragLeave,
    DragEnter,
    Touch,
    Key,
    KeyPreIme,
    Mouse,
    Axis,
}

impl ArkEventKind {
    /// Whether Dioxus should propagate this event through ancestor listeners.
    pub const fn bubbles(self) -> bool {
        matches!(
            self,
            Self::Click
                | Self::LongPress
                | Self::Touch
                | Self::Key
                | Self::KeyPreIme
                | Self::Mouse
                | Self::FocusIn
                | Self::FocusOut
        )
    }
}

/// Classify an RSX listener name passed by Dioxus to a renderer.
///
/// Canonical listener names use the RSX spelling: `onclick`, `onchange`,
/// `onscroll`, `onarea`, and so on. Dioxus strips the leading `on` before
/// handing a listener name to the renderer, so the compact spelling (`click`)
/// resolves to the same identity.
///
/// The relation is generated from the listener declarations in
/// [`crate::events`], which is also what defines the listener functions
/// themselves — a name that exists there always classifies, and one that does
/// not never will.
pub fn classify_event_name(name: &str) -> Option<ArkEventKind> {
    crate::events::EVENT_KINDS
        .iter()
        .find_map(|(canonical, kind)| {
            (*canonical == name || canonical.get("on".len()..) == Some(name)).then_some(*kind)
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
    /// An ArkUI accessibility action bit (`click`, `long click`, `cut`,
    /// `copy`, or `paste`).
    AccessibilityAction(u32),
    /// A string value (text input/area change).
    String(String),
    /// A scroll-index payload (list/grid/water-flow visible range).
    ScrollIndex(ScrollIndexPayload),
    /// A physical scroll offset payload (generic Scroll and scroll observers).
    ScrollOffset(ScrollOffsetPayload),
    /// Layout frame payload for element-bound area/layout change events.
    Layout(LayoutPayload),
    /// Pointer payload for input events such as click, touch, and hover move.
    Pointer(PointerPayload),
    /// Physical keyboard input delivered to the focused node.
    Key(KeyPayload),
    /// Raw mouse input delivered to a desktop-capable node.
    Mouse(MousePayload),
    Axis(AxisPayload),
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
    /// Horizontal movement for this scroll frame, in vp.
    pub x: f32,
    /// Vertical movement for this scroll frame, in vp.
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
    pub source: PointerSource,
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

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum PointerSource {
    #[default]
    Unknown,
    Touch,
    Mouse,
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

/// Platform-neutral mouse button identity.
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum MouseButton {
    #[default]
    None,
    Primary,
    Secondary,
    Middle,
    Back,
    Forward,
    Side,
    Extra,
    Task,
    Other(i32),
}

/// Raw mouse values captured during the native callback.
#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct MousePayload {
    pub button: MouseButton,
    pub action: PointerAction,
    pub x: f32,
    pub y: f32,
    pub window_x: f32,
    pub window_y: f32,
    pub target_x: f32,
    pub target_y: f32,
    pub target_width: f32,
    pub target_height: f32,
    pub modifiers: KeyModifiers,
}

impl From<MousePayload> for PointerPayload {
    fn from(mouse: MousePayload) -> Self {
        Self {
            source: PointerSource::Mouse,
            action: mouse.action,
            x: mouse.x,
            y: mouse.y,
            window_x: mouse.window_x,
            window_y: mouse.window_y,
            target_x: mouse.target_x,
            target_y: mouse.target_y,
            target_width: mouse.target_width,
            target_height: mouse.target_height,
            ..Default::default()
        }
    }
}

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct AxisPayload {
    pub horizontal: f64,
    pub vertical: f64,
    pub x: f32,
    pub y: f32,
    pub modifiers: KeyModifiers,
}

#[derive(Default, Clone, Copy, Debug)]
pub struct AxisData {
    pub horizontal: f64,
    pub vertical: f64,
    pub x: f32,
    pub y: f32,
    pub modifiers: KeyModifiers,
}

impl From<ArkEventData> for AxisData {
    fn from(data: ArkEventData) -> Self {
        Self::from(&data)
    }
}
impl From<&ArkEventData> for AxisData {
    fn from(data: &ArkEventData) -> Self {
        match data.payload {
            ArkEventPayload::Axis(axis) => Self {
                horizontal: axis.horizontal,
                vertical: axis.vertical,
                x: axis.x,
                y: axis.y,
                modifiers: axis.modifiers,
            },
            _ => Self::default(),
        }
    }
}

/// Platform-neutral key identity used by desktop-capable components.
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyboardKey {
    #[default]
    Unknown,
    Enter,
    Space,
    Tab,
    Escape,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    Home,
    End,
    PageUp,
    PageDown,
    Delete,
    Backspace,
    Menu,
    F10,
    Function(u8),
    Character(char),
    Other(i32),
}

/// Directional-key policy for lists, tabs, and calendar grids.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyboardNavigation {
    Horizontal,
    Vertical,
    Grid(usize),
}

pub fn keyboard_navigation_target(
    current: usize,
    key: KeyboardKey,
    navigation: KeyboardNavigation,
    enabled: &[bool],
) -> Option<usize> {
    let count = enabled.len();
    if count == 0 {
        return None;
    }
    if key == KeyboardKey::Home {
        return enabled.iter().position(|enabled| *enabled);
    }
    if key == KeyboardKey::End {
        return enabled.iter().rposition(|enabled| *enabled);
    }
    let step = match (navigation, key) {
        (KeyboardNavigation::Horizontal | KeyboardNavigation::Grid(_), KeyboardKey::ArrowLeft)
        | (KeyboardNavigation::Vertical, KeyboardKey::ArrowUp) => -1,
        (KeyboardNavigation::Horizontal | KeyboardNavigation::Grid(_), KeyboardKey::ArrowRight)
        | (KeyboardNavigation::Vertical, KeyboardKey::ArrowDown) => 1,
        (KeyboardNavigation::Grid(columns), KeyboardKey::ArrowUp) => -(columns.max(1) as isize),
        (KeyboardNavigation::Grid(columns), KeyboardKey::ArrowDown) => columns.max(1) as isize,
        _ => return None,
    };
    let mut next = current.min(count - 1) as isize;
    for _ in 0..count {
        next += step;
        match navigation {
            KeyboardNavigation::Grid(_) if next < 0 || next >= count as isize => return None,
            KeyboardNavigation::Grid(_) => {}
            _ => next = next.rem_euclid(count as isize),
        }
        if enabled[next as usize] {
            return Some(next as usize);
        }
    }
    None
}

/// Native key lifecycle normalized across OpenHarmony keyboard sources.
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyAction {
    #[default]
    Unknown,
    Down,
    Up,
    Repeat,
    Click,
}

/// Modifier snapshot attached to a physical key event.
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyModifiers {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub function: bool,
}

/// Raw and normalized keyboard values captured during the native callback.
#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct KeyPayload {
    pub key: KeyboardKey,
    pub action: KeyAction,
    pub raw_code: i32,
    pub text: String,
    pub modifiers: KeyModifiers,
}

/// Declarative native key ownership. Unclaimed keys retain platform behavior.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct KeyboardCapture(u32);

impl KeyboardCapture {
    pub fn from_names(names: &str) -> Self {
        let mut mask = 0;
        for name in names.split([' ', ',', '|']).filter(|name| !name.is_empty()) {
            mask |= match name {
                "all" => u32::MAX,
                "enter" => 1,
                "space" => 2,
                "tab" => 4,
                "escape" => 8,
                "up" => 16,
                "down" => 32,
                "left" => 64,
                "right" => 128,
                "home" => 256,
                "end" => 512,
                "page_up" => 1024,
                "page_down" => 2048,
                _ => 0,
            };
        }
        Self(mask)
    }

    pub fn captures(self, key: &KeyPayload) -> bool {
        if self.0 == u32::MAX {
            return true;
        }
        if key.modifiers.ctrl || key.modifiers.alt {
            return false;
        }
        let bit = match key.key {
            KeyboardKey::Enter => 1,
            KeyboardKey::Space => 2,
            KeyboardKey::Tab => 4,
            KeyboardKey::Escape => 8,
            KeyboardKey::ArrowUp => 16,
            KeyboardKey::ArrowDown => 32,
            KeyboardKey::ArrowLeft => 64,
            KeyboardKey::ArrowRight => 128,
            KeyboardKey::Home => 256,
            KeyboardKey::End => 512,
            KeyboardKey::PageUp => 1024,
            KeyboardKey::PageDown => 2048,
            _ => 0,
        };
        self.0 & bit != 0
    }
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

/// Action requested by an accessibility service.
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct AccessibilityActionData {
    /// ArkUI action bit: click=1, long-click=2, cut=4, copy=8, paste=16.
    pub action: u32,
}

impl From<ArkEventData> for AccessibilityActionData {
    fn from(data: ArkEventData) -> Self {
        Self::from(&data)
    }
}

impl From<&ArkEventData> for AccessibilityActionData {
    fn from(data: &ArkEventData) -> Self {
        match &data.payload {
            ArkEventPayload::AccessibilityAction(action) => Self { action: *action },
            _ => Self::default(),
        }
    }
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

/// Data for a touch or pointer movement event. Drag lifecycle events currently
/// carry `None` because ArkUI exposes `ArkUI_DragEvent` rather than an input
/// event; use `NodeBuilder`'s raw drag callbacks when the native object is
/// required.
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

/// Data for a native mouse event.
pub type MouseData = MousePayload;

impl MouseData {
    pub fn secondary_down(self) -> bool {
        self.button == MouseButton::Secondary && self.action == PointerAction::Down
    }
}

impl From<ArkEventData> for MouseData {
    fn from(data: ArkEventData) -> Self {
        Self::from(&data)
    }
}

impl From<&ArkEventData> for MouseData {
    fn from(data: &ArkEventData) -> Self {
        match data.payload {
            ArkEventPayload::Mouse(payload) => payload,
            _ => Self::default(),
        }
    }
}

/// Data for a physical keyboard event.
#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct KeyData {
    pub key: KeyboardKey,
    pub action: KeyAction,
    pub raw_code: i32,
    pub text: String,
    pub modifiers: KeyModifiers,
}

impl KeyData {
    pub fn is_down(&self) -> bool {
        matches!(self.action, KeyAction::Down | KeyAction::Repeat)
    }

    pub fn activates(&self) -> bool {
        self.action == KeyAction::Down
            && !self.modifiers.ctrl
            && !self.modifiers.alt
            && matches!(self.key, KeyboardKey::Enter | KeyboardKey::Space)
    }
}

impl From<ArkEventData> for KeyData {
    fn from(data: ArkEventData) -> Self {
        Self::from(&data)
    }
}

impl From<&ArkEventData> for KeyData {
    fn from(data: &ArkEventData) -> Self {
        match &data.payload {
            ArkEventPayload::Key(payload) => Self {
                key: payload.key,
                action: payload.action,
                raw_code: payload.raw_code,
                text: payload.text.clone(),
                modifiers: payload.modifiers,
            },
            _ => Self::default(),
        }
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
            | ArkEventPayload::AccessibilityAction(_)
            | ArkEventPayload::Key(_)
            | ArkEventPayload::Mouse(_)
            | ArkEventPayload::Axis(_)
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
    /// Horizontal movement for this scroll frame, in vp.
    pub offset_x: f32,
    /// Vertical movement for this scroll frame, in vp.
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

/// Data for a scroll-container reach-end event. No payload.
#[derive(Default, Clone, Copy, Debug)]
pub struct ReachEndData;

impl From<ArkEventData> for ReachEndData {
    fn from(_data: ArkEventData) -> Self {
        ReachEndData
    }
}

impl From<&ArkEventData> for ReachEndData {
    fn from(_data: &ArkEventData) -> Self {
        ReachEndData
    }
}

#[cfg(test)]
mod tests {
    use super::{
        classify_event_name, ArkEventData, ArkEventKind, ArkEventPayload, KeyAction, KeyData,
        KeyPayload, KeyboardKey, MouseButton, MouseData, MousePayload, PointerAction,
    };

    #[test]
    fn canonical_event_names_share_one_semantic_identity() {
        for name in ["onclick", "click"] {
            assert_eq!(classify_event_name(name), Some(ArkEventKind::Click));
        }
        for name in ["onlongpress", "longpress"] {
            assert_eq!(classify_event_name(name), Some(ArkEventKind::LongPress));
        }
        for name in ["onfocus", "focus"] {
            assert_eq!(classify_event_name(name), Some(ArkEventKind::Focus));
        }
        for name in ["onblur", "blur"] {
            assert_eq!(classify_event_name(name), Some(ArkEventKind::Blur));
        }
        for name in ["onreachend", "reachend"] {
            assert_eq!(classify_event_name(name), Some(ArkEventKind::ReachEnd));
        }
        for name in ["onkey", "key"] {
            assert_eq!(classify_event_name(name), Some(ArkEventKind::Key));
        }
        for name in ["onmouse", "mouse"] {
            assert_eq!(classify_event_name(name), Some(ArkEventKind::Mouse));
        }
        assert!(ArkEventKind::Click.bubbles());
        assert!(ArkEventKind::LongPress.bubbles());
        assert!(!ArkEventKind::Change.bubbles());
    }

    #[test]
    fn keyboard_activation_requires_a_press_of_enter_or_space() {
        let enter = KeyData::from(ArkEventData::with_payload(ArkEventPayload::Key(
            KeyPayload {
                key: KeyboardKey::Enter,
                action: KeyAction::Down,
                ..Default::default()
            },
        )));
        assert!(enter.activates());

        let released_space = KeyData::from(ArkEventData::with_payload(ArkEventPayload::Key(
            KeyPayload {
                key: KeyboardKey::Space,
                action: KeyAction::Up,
                ..Default::default()
            },
        )));
        assert!(!released_space.activates());
        let repeated = KeyData {
            key: KeyboardKey::Enter,
            action: KeyAction::Repeat,
            ..Default::default()
        };
        assert!(repeated.is_down());
        assert!(!repeated.activates());
        let shortcut = KeyData {
            key: KeyboardKey::Enter,
            action: KeyAction::Down,
            modifiers: super::KeyModifiers {
                ctrl: true,
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(!shortcut.activates());
    }

    #[test]
    fn native_key_capture_preserves_unclaimed_editing_and_shortcuts() {
        let capture = super::KeyboardCapture::from_names("enter space left right escape");
        let mut key = KeyPayload {
            key: KeyboardKey::ArrowRight,
            action: KeyAction::Down,
            ..Default::default()
        };
        assert!(capture.captures(&key));
        key.modifiers.ctrl = true;
        assert!(!capture.captures(&key));
        key.key = KeyboardKey::Character('c');
        assert!(!capture.captures(&key));
        assert!(super::KeyboardCapture::from_names("all").captures(&key));
    }

    #[test]
    fn roving_navigation_skips_disabled_items_and_keeps_calendar_columns() {
        use super::{keyboard_navigation_target as next, KeyboardNavigation};
        let enabled = [true, false, true, true];
        assert_eq!(
            next(
                0,
                KeyboardKey::ArrowRight,
                KeyboardNavigation::Horizontal,
                &enabled
            ),
            Some(2)
        );
        assert_eq!(
            next(
                0,
                KeyboardKey::ArrowLeft,
                KeyboardNavigation::Horizontal,
                &enabled
            ),
            Some(3)
        );
        let grid = [true; 42];
        assert_eq!(
            next(
                8,
                KeyboardKey::ArrowDown,
                KeyboardNavigation::Grid(7),
                &grid
            ),
            Some(15)
        );
        assert_eq!(
            next(8, KeyboardKey::ArrowUp, KeyboardNavigation::Grid(7), &grid),
            Some(1)
        );
        assert_eq!(
            next(1, KeyboardKey::ArrowUp, KeyboardNavigation::Grid(7), &grid),
            None
        );
        assert_eq!(
            next(0, KeyboardKey::End, KeyboardNavigation::Vertical, &enabled),
            Some(3)
        );
        assert_eq!(
            next(
                0,
                KeyboardKey::ArrowRight,
                KeyboardNavigation::Horizontal,
                &[false; 3]
            ),
            None
        );
    }

    #[test]
    fn secondary_mouse_press_is_preserved() {
        let mouse = MouseData::from(ArkEventData::with_payload(ArkEventPayload::Mouse(
            MousePayload {
                button: MouseButton::Secondary,
                action: PointerAction::Down,
                window_x: 24.0,
                window_y: 48.0,
                ..Default::default()
            },
        )));
        assert!(mouse.secondary_down());
        assert_eq!((mouse.window_x, mouse.window_y), (24.0, 48.0));
    }

    /// The classifier is generated from the listener declarations, so every
    /// declared listener must resolve and no name may be declared twice.
    #[test]
    fn every_declared_listener_classifies_in_both_spellings() {
        let mut seen = std::collections::HashSet::new();
        assert!(!crate::events::EVENT_KINDS.is_empty());
        for (canonical, kind) in crate::events::EVENT_KINDS {
            assert!(canonical.starts_with("on"), "{canonical}");
            assert!(seen.insert(*canonical), "duplicate listener {canonical}");
            assert_eq!(classify_event_name(canonical), Some(*kind), "{canonical}");
            let compact = &canonical["on".len()..];
            assert_eq!(classify_event_name(compact), Some(*kind), "{compact}");
        }
    }

    #[test]
    fn historical_snake_case_aliases_are_rejected() {
        for name in [
            "on_press",
            "_press",
            "on_long_press",
            "_long_press",
            "on_focus",
            "_focus",
            "on_blur",
            "_blur",
            "on_reach_end",
            "_reach_end",
        ] {
            assert_eq!(classify_event_name(name), None, "{name}");
        }
    }
}
