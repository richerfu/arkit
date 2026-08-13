//! Sonner-style toast surfaces.
//!
//! `Sonner` owns the viewport-level stack while callers own the toast data.
//! The default presentation is mobile-first: a bottom-center **overlapping**
//! notification stack inset from the horizontal edges and the system safe
//! area, 40vp action targets, swipe-to-dismiss, and vertical scroll to cycle
//! the front toast. Call sites can also request a compact minimal toast that
//! skips notification chrome.

use std::time::Duration;
use std::{cell::RefCell, fmt, rc::Rc};

use super::motion::{TOAST_DISTANCE, TOAST_ENTER_MS, TOAST_EXIT_MS, TOAST_STACK_MS};
use super::{Spinner, ARKUI_BORDER_STYLE_SOLID};
use crate::icon::icon_placeholder;
use crate::theme::*;
use arkit_animation::{
    use_animate_presence, use_animation, use_animation_target, Animation, AnimationSelector,
    BuiltinEase, EaseDirection, Easing, Length, PresenceKey, PresenceMode, PresencePhase,
    PresenceTransition, TargetName, TimeSpan, Timeline, TimelinePosition, TransitionPreset,
    OPACITY, POSITION_X, POSITION_Y, WIDTH,
};
use arkit_prelude::*;

use super::floating_layer::{ALIGN_TOP, HIT_TEST_DEFAULT, HIT_TEST_NONE};

const DEFAULT_DURATION_MS: u64 = 4_000;
const DEFAULT_MAX_WIDTH: f32 = 420.0;
const DEFAULT_MIN_HEIGHT: f32 = 64.0;
const MINIMAL_MIN_HEIGHT: f32 = 36.0;
const MINIMAL_MAX_WIDTH: f32 = 240.0;
const MINIMAL_MIN_WIDTH: f32 = 96.0;
const TOAST_ICON_SIZE: f32 = 20.0;
const MINIMAL_ICON_SIZE: f32 = 14.0;
const TOAST_ACTION_HEIGHT: f32 = 40.0;
const TOAST_CLOSE_SIZE: f32 = 40.0;
const SWIPE_DISMISS_THRESHOLD: f32 = 56.0;
const HORIZONTAL_SWIPE_DISMISS_THRESHOLD: f32 = 72.0;
const STACK_EXPAND_THRESHOLD: f32 = 40.0;
/// Official Sonner `GAP` — vertical lift between collapsed peeks.
const DEFAULT_STACK_OFFSET: f32 = 14.0;
/// Official Sonner scale step: `1 - index * 0.05`.
const STACK_SCALE_STEP: f32 = 0.05;
/// Collapsed stack forces every card to the front toast height so peeks
/// stick out evenly (see sonner `front-toast-height`).
const FRONT_TOAST_HEIGHT: f32 = 72.0;
/// Approximate natural height when expanded (title + description + padding).
const EXPANDED_TOAST_HEIGHT: f32 = 72.0;

/// Semantic toast types supported by shadcn Sonner.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToastVariant {
    #[default]
    Default,
    Success,
    Info,
    Warning,
    Error,
    Loading,
}

/// Visual density of a toast card.
///
/// - [`ToastAppearance::Notification`] — full notification card (icon,
///   description, action, close). Multiple notifications overlap with the
///   newest in front; vertical swipe cycles the front card.
/// - [`ToastAppearance::Minimal`] — compact shadcn card, primary message only.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToastAppearance {
    #[default]
    Notification,
    Minimal,
}

/// Viewport anchors supported by Sonner.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SonnerPosition {
    TopLeft,
    TopCenter,
    TopRight,
    BottomLeft,
    #[default]
    BottomCenter,
    BottomRight,
}

impl SonnerPosition {
    fn is_top(self) -> bool {
        matches!(self, Self::TopLeft | Self::TopCenter | Self::TopRight)
    }

    fn horizontal(self) -> HorizontalPosition {
        match self {
            Self::TopLeft | Self::BottomLeft => HorizontalPosition::Left,
            Self::TopRight | Self::BottomRight => HorizontalPosition::Right,
            Self::TopCenter | Self::BottomCenter => HorizontalPosition::Center,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HorizontalPosition {
    Left,
    Center,
    Right,
}

/// Direction used by a standalone [`Toast`] for vertical swipe dismissal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToastSwipeDirection {
    Up,
    #[default]
    Down,
}

/// Optional visual overrides for a toast card.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ToastStyle {
    pub background_color: Option<u32>,
    pub foreground_color: Option<u32>,
    pub description_color: Option<u32>,
    pub border_color: Option<u32>,
    pub icon_color: Option<u32>,
    pub action_background_color: Option<u32>,
    pub action_foreground_color: Option<u32>,
    pub border_radius: Option<f32>,
    pub min_height: Option<f32>,
    /// When `Some(true)` / `None`, apply small outer shadow; `Some(false)` disables it.
    pub shadow: Option<bool>,
}

/// Layout and card styling for the viewport-level toaster.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SonnerStyle {
    /// Maximum card width. Narrower phones use the available width.
    pub max_width: f32,
    /// Distance from the safe-area edge.
    pub offset: f32,
    /// Horizontal viewport inset.
    pub inset: f32,
    /// Gap between fully expanded toast cards (Sonner `--gap`).
    pub gap: f32,
    /// Collapsed-stack lift between peeks. Defaults to Sonner's 14vp `GAP`.
    pub stack_offset: f32,
    /// Optional toast card overrides.
    pub toast: ToastStyle,
}

impl Default for SonnerStyle {
    fn default() -> Self {
        Self {
            max_width: DEFAULT_MAX_WIDTH,
            offset: spacing::LG,
            inset: spacing::LG,
            gap: spacing::SM,
            stack_offset: DEFAULT_STACK_OFFSET,
            toast: ToastStyle::default(),
        }
    }
}

/// One controlled item in a [`Sonner`] stack.
#[derive(Clone)]
pub struct SonnerToast {
    /// Stable identity. Reusing an id for two live toasts is unsupported.
    pub id: u64,
    /// Presentation revision. Incrementing it remounts the entry and restarts
    /// its timer, which is required when a persistent loading toast is updated
    /// in place to a timed success or error toast.
    pub revision: u32,
    pub title: String,
    pub description: Option<String>,
    pub variant: ToastVariant,
    /// Full notification card vs compact minimal pill.
    pub appearance: ToastAppearance,
    pub action_label: Option<String>,
    pub icon: Option<String>,
    /// Zero keeps the toast visible until explicitly dismissed.
    pub duration_ms: u64,
    pub dismissible: bool,
    action_handler: Option<Rc<dyn Fn()>>,
    dismiss_handler: Option<Rc<dyn Fn()>>,
}

impl fmt::Debug for SonnerToast {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SonnerToast")
            .field("id", &self.id)
            .field("revision", &self.revision)
            .field("title", &self.title)
            .field("description", &self.description)
            .field("variant", &self.variant)
            .field("appearance", &self.appearance)
            .field("action_label", &self.action_label)
            .field("icon", &self.icon)
            .field("duration_ms", &self.duration_ms)
            .field("dismissible", &self.dismissible)
            .finish_non_exhaustive()
    }
}

impl PartialEq for SonnerToast {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.revision == other.revision
            && self.title == other.title
            && self.description == other.description
            && self.variant == other.variant
            && self.appearance == other.appearance
            && self.action_label == other.action_label
            && self.icon == other.icon
            && self.duration_ms == other.duration_ms
            && self.dismissible == other.dismissible
            && callback_eq(&self.action_handler, &other.action_handler)
            && callback_eq(&self.dismiss_handler, &other.dismiss_handler)
    }
}

impl SonnerToast {
    pub fn new(id: u64, title: impl Into<String>) -> Self {
        Self {
            id,
            revision: 0,
            title: title.into(),
            description: None,
            variant: ToastVariant::Default,
            appearance: ToastAppearance::Notification,
            action_label: None,
            icon: None,
            duration_ms: DEFAULT_DURATION_MS,
            dismissible: true,
            action_handler: None,
            dismiss_handler: None,
        }
    }

    pub fn success(id: u64, title: impl Into<String>) -> Self {
        Self::new(id, title).variant(ToastVariant::Success)
    }

    pub fn info(id: u64, title: impl Into<String>) -> Self {
        Self::new(id, title).variant(ToastVariant::Info)
    }

    pub fn warning(id: u64, title: impl Into<String>) -> Self {
        Self::new(id, title).variant(ToastVariant::Warning)
    }

    pub fn error(id: u64, title: impl Into<String>) -> Self {
        Self::new(id, title).variant(ToastVariant::Error)
    }

    pub fn loading(id: u64, title: impl Into<String>) -> Self {
        Self::new(id, title)
            .variant(ToastVariant::Loading)
            .duration_ms(0)
    }

    /// Compact pill presentation (message only; no action/close chrome).
    pub fn minimal(id: u64, title: impl Into<String>) -> Self {
        Self::new(id, title)
            .appearance(ToastAppearance::Minimal)
            .dismissible(false)
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub const fn variant(mut self, variant: ToastVariant) -> Self {
        self.variant = variant;
        self
    }

    pub const fn appearance(mut self, appearance: ToastAppearance) -> Self {
        self.appearance = appearance;
        self
    }

    pub const fn revision(mut self, revision: u32) -> Self {
        self.revision = revision;
        self
    }

    pub fn action(mut self, label: impl Into<String>) -> Self {
        self.action_label = Some(label.into());
        self
    }

    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub const fn duration_ms(mut self, duration_ms: u64) -> Self {
        self.duration_ms = duration_ms;
        self
    }

    pub const fn dismissible(mut self, dismissible: bool) -> Self {
        self.dismissible = dismissible;
        self
    }

    /// Attach the behavior for the optional action button.
    pub fn on_action(mut self, handler: impl Fn() + 'static) -> Self {
        self.action_handler = Some(Rc::new(handler));
        self
    }

    /// Observe every dismissal path: timer, close button, swipe, or action.
    pub fn on_dismiss(mut self, handler: impl Fn() + 'static) -> Self {
        self.dismiss_handler = Some(Rc::new(handler));
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ToastIdentity {
    id: u64,
    revision: u32,
}

impl From<&SonnerToast> for ToastIdentity {
    fn from(toast: &SonnerToast) -> Self {
        Self {
            id: toast.id,
            revision: toast.revision,
        }
    }
}

#[derive(Debug)]
struct LegacyMessageSlot {
    message: String,
    revision: u32,
}

#[derive(Debug, Default)]
struct SonnerState {
    dismissed: Vec<ToastIdentity>,
    legacy_messages: Vec<LegacyMessageSlot>,
}

impl SonnerState {
    fn reconcile_legacy_messages(&mut self, messages: Vec<String>) -> Vec<SonnerToast> {
        let message_count = messages.len();
        let mut toasts = Vec::with_capacity(message_count);

        for (index, message) in messages.into_iter().enumerate() {
            let revision = match self.legacy_messages.get_mut(index) {
                Some(slot) if slot.message == message => slot.revision,
                Some(slot) => {
                    slot.message.clone_from(&message);
                    slot.revision = slot
                        .revision
                        .checked_add(1)
                        .expect("sonner legacy message revision space exhausted");
                    slot.revision
                }
                None => {
                    self.legacy_messages.push(LegacyMessageSlot {
                        message: message.clone(),
                        revision: 0,
                    });
                    0
                }
            };
            toasts.push(
                SonnerToast::new(u64::MAX.saturating_sub(index as u64), message).revision(revision),
            );
        }

        self.legacy_messages.truncate(message_count);
        toasts
    }

    fn reconcile_dismissals(&mut self, live: &[ToastIdentity]) {
        self.dismissed.retain(|identity| live.contains(identity));
    }

    fn dismiss(&mut self, identity: ToastIdentity) -> bool {
        if self.dismissed.contains(&identity) {
            false
        } else {
            self.dismissed.push(identity);
            true
        }
    }

    fn is_dismissed(&self, identity: ToastIdentity) -> bool {
        self.dismissed.contains(&identity)
    }
}

fn callback_eq(left: &Option<Rc<dyn Fn()>>, right: &Option<Rc<dyn Fn()>>) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => Rc::ptr_eq(left, right),
        (None, None) => true,
        _ => false,
    }
}

/// Props for a standalone toast card.
#[derive(Props, Clone, PartialEq)]
pub struct ToastProps {
    /// Primary line. Kept as `message` for compatibility with the old API.
    pub message: String,
    #[props(default)]
    pub description: Option<String>,
    #[props(default)]
    pub variant: ToastVariant,
    #[props(default)]
    pub appearance: ToastAppearance,
    #[props(default)]
    pub action_label: Option<String>,
    #[props(default)]
    pub icon: Option<String>,
    #[props(default = true)]
    pub dismissible: bool,
    #[props(default)]
    pub rich_colors: bool,
    #[props(default)]
    pub swipe_direction: ToastSwipeDirection,
    /// When true, reverse-axis vertical swipes expand/collapse the stack.
    #[props(default)]
    pub stackable: bool,
    /// Whether the notification stack is currently expanded.
    #[props(default)]
    pub expanded: bool,
    /// Behind-card shell in collapsed Sonner stack: fixed height, content hidden.
    #[props(default)]
    pub stacked_back: bool,
    /// Forced height for collapsed back cards (`front-toast-height`).
    #[props(default)]
    pub stacked_height: Option<f32>,
    #[props(default)]
    pub style: ToastStyle,
    #[props(default)]
    pub on_action: Option<EventHandler<()>>,
    #[props(default)]
    pub on_dismiss: Option<EventHandler<()>>,
    /// Toggle expanded list presentation for overlapping notification stacks.
    #[props(default)]
    pub on_expand_change: Option<EventHandler<bool>>,
}

/// A shadcn-styled mobile toast card.
#[component]
pub fn Toast(props: ToastProps) -> Element {
    let theme = use_theme();
    let is_minimal = props.appearance == ToastAppearance::Minimal;
    let palette = toast_palette(
        props.variant,
        theme,
        props.rich_colors,
        props.style,
        is_minimal,
    );
    let mut drag_start = use_signal(|| None::<(f32, f32)>);
    let on_action = props.on_action;
    let on_dismiss = props.on_dismiss;
    let on_expand_change = props.on_expand_change;
    let swipe_direction = props.swipe_direction;
    let stackable = props.stackable;
    let expanded = props.expanded;
    let stacked_back = props.stacked_back;
    let icon = props.icon.or_else(|| {
        if is_minimal {
            match props.variant {
                ToastVariant::Default | ToastVariant::Loading => None,
                _ => variant_icon(props.variant),
            }
        } else {
            variant_icon(props.variant)
        }
    });
    let has_icon = props.variant == ToastVariant::Loading || icon.is_some();
    let border_radius = props.style.border_radius.unwrap_or(if is_minimal {
        theme.radii.md
    } else {
        theme.radii.lg
    });
    let min_height = props
        .style
        .min_height
        .or(props.stacked_height)
        .unwrap_or(if is_minimal {
            MINIMAL_MIN_HEIGHT
        } else {
            DEFAULT_MIN_HEIGHT
        });
    let show_shadow = props.style.shadow.unwrap_or(true);
    let show_description = !is_minimal && props.description.is_some();
    let show_action = !is_minimal && props.action_label.is_some();
    let show_close = !is_minimal && props.dismissible;
    let icon_size = if is_minimal {
        MINIMAL_ICON_SIZE
    } else {
        TOAST_ICON_SIZE
    };
    let pad_y = if is_minimal { 8.0 } else { spacing::MD };
    let pad_x = if is_minimal { 12.0 } else { spacing::MD };
    let title_weight = if is_minimal { 500_i32 } else { 600_i32 };
    let chip_width = if is_minimal {
        Some(minimal_chip_width(&props.message, has_icon))
    } else {
        None
    };

    // Split minimal/notification roots so notification keeps percent width and
    // minimal gets an explicit content-sized chip width.
    if is_minimal {
        let width = chip_width.unwrap_or(MINIMAL_MIN_WIDTH);
        return rsx! {
            row {
                width,
                height: min_height,
                align_items: "center",
                justify_content: "center",
                padding_top: 0.0,
                padding_right: pad_x,
                padding_bottom: 0.0,
                padding_left: pad_x,
                background_color: palette.background,
                border_width: 1.0,
                border_color: palette.border,
                border_style: ARKUI_BORDER_STYLE_SOLID,
                border_radius,
                shadow: if show_shadow { "sm" },
                clip: true,
                hit_test_behavior: "default",
                on_touch: move |event| {
                    handle_toast_touch(
                        event,
                        &mut drag_start,
                        swipe_direction,
                        false,
                        expanded,
                        on_expand_change,
                        on_dismiss,
                    );
                },
                if props.variant == ToastVariant::Loading {
                    row {
                        width: 18.0,
                        height: min_height,
                        align_items: "center",
                        justify_content: "center",
                        Spinner {
                            size: icon_size,
                            color: Some(palette.icon),
                        }
                    }
                } else if let Some(icon_name) = icon {
                    row {
                        width: 18.0,
                        height: min_height,
                        align_items: "center",
                        justify_content: "center",
                        {icon_placeholder(&icon_name, icon_size, palette.icon)}
                    }
                }
                text {
                    content: props.message,
                    font_size: typography::SM,
                    font_weight: title_weight,
                    font_color: palette.foreground,
                    line_height: 20.0,
                    max_lines: 1_i32,
                    text_overflow: "ellipsis",
                }
            }
        };
    }

    rsx! {
        row {
            width: "100%",
            constraint_size: format!("0,100000,{min_height},100000"),
            align_items: "center",
            justify_content: "start",
            padding_top: pad_y,
            padding_right: if show_close { spacing::SM } else { pad_x },
            padding_bottom: pad_y,
            padding_left: pad_x,
            background_color: palette.background,
            border_width: 1.0,
            border_color: palette.border,
            border_style: ARKUI_BORDER_STYLE_SOLID,
            border_radius,
            shadow: if show_shadow { "sm" },
            clip: true,
            // The Sonner layer is intentionally pass-through. Re-enable hit
            // testing on the card itself so ArkUI delivers touch sequences to
            // the swipe recognizer while the empty overlay remains inert.
            hit_test_behavior: if stacked_back { "none" } else { "default" },
            on_touch: move |event| {
                handle_toast_touch(
                    event,
                    &mut drag_start,
                    swipe_direction,
                    stackable,
                    expanded,
                    on_expand_change,
                    on_dismiss,
                );
            },
            SonnerPeekContent {
                visible: !stacked_back,
            if props.variant == ToastVariant::Loading {
                row {
                    width: 28.0,
                    height: TOAST_CLOSE_SIZE,
                    align_items: "center",
                    justify_content: "start",
                    Spinner {
                        size: icon_size,
                        color: Some(palette.icon),
                    }
                }
            } else if let Some(icon_name) = icon {
                row {
                    width: 28.0,
                    height: TOAST_CLOSE_SIZE,
                    align_items: "center",
                    justify_content: "start",
                    {icon_placeholder(&icon_name, icon_size, palette.icon)}
                }
            }
            column {
                layout_weight: 1.0,
                align_items: "start",
                justify_content: "center",
                text {
                    width: "100%",
                    content: props.message,
                    font_size: typography::SM,
                    font_weight: title_weight,
                    font_color: palette.foreground,
                    line_height: 20.0,
                    max_lines: 2_i32,
                    text_overflow: "ellipsis",
                }
                if show_description {
                    if let Some(description) = props.description.clone() {
                        row { height: spacing::XXS }
                        text {
                            width: "100%",
                            content: description,
                            font_size: typography::XS,
                            font_weight: 400_i32,
                            font_color: palette.description,
                            line_height: 18.0,
                            max_lines: 3_i32,
                            text_overflow: "ellipsis",
                        }
                    }
                }
            }
            if show_action {
                if let Some(action_label) = props.action_label.clone() {
                    row { width: spacing::SM }
                    button {
                        button_type: "normal",
                        height: TOAST_ACTION_HEIGHT,
                        padding_top: 0.0,
                        padding_right: spacing::MD,
                        padding_bottom: 0.0,
                        padding_left: spacing::MD,
                        background_color: palette.action_background,
                        foreground_color: palette.action_foreground,
                        border_width: 0.0,
                        border_style: ARKUI_BORDER_STYLE_SOLID,
                        border_radius: theme.radii.md,
                        focusable: false,
                        focus_on_touch: false,
                        alignment: "center",
                        onclick: move |event| {
                            event.stop_propagation();
                            if let Some(handler) = on_action {
                                handler.call(());
                            }
                        },
                        text {
                            content: action_label,
                            font_size: typography::XS,
                            font_weight: 600_i32,
                            font_color: palette.action_foreground,
                            line_height: 18.0,
                        }
                    }
                }
            }
            if show_close {
                button {
                    button_type: "normal",
                    width: TOAST_CLOSE_SIZE,
                    height: TOAST_CLOSE_SIZE,
                    padding: 0.0,
                    background_color: "#00000000",
                    border_width: 0.0,
                    border_style: ARKUI_BORDER_STYLE_SOLID,
                    border_radius: theme.radii.md,
                    focusable: false,
                    focus_on_touch: false,
                    alignment: "center",
                    onclick: move |event| {
                        event.stop_propagation();
                        if let Some(handler) = on_dismiss {
                            handler.call(());
                        }
                    },
                    {icon_placeholder("x", 16.0, palette.description)}
                }
            }
            }
        }
    }
}

/// Compatibility wrapper for the former destructive-only toast component.
#[derive(Props, Clone, PartialEq)]
pub struct ToastDestructiveProps {
    pub message: String,
}

#[component]
pub fn ToastDestructive(props: ToastDestructiveProps) -> Element {
    rsx! {
        Toast {
            message: props.message,
            variant: ToastVariant::Error,
        }
    }
}

/// Props for the viewport-level Sonner stack.
#[derive(Props, Clone, PartialEq)]
pub struct SonnerProps {
    /// Structured toast items, ordered oldest to newest.
    #[props(default)]
    pub toasts: Vec<SonnerToast>,
    /// Legacy plain-message input. Prefer `toasts` for new call sites.
    #[props(default)]
    pub messages: Vec<String>,
    #[props(default)]
    pub position: SonnerPosition,
    /// Maximum number of collapsed notification cards to paint. Timers still
    /// run for every live toast.
    #[props(default = 3usize)]
    pub visible_toasts: usize,
    #[props(default)]
    pub rich_colors: bool,
    #[props(default)]
    pub style: SonnerStyle,
    /// Controlled dismissal callback. When supplied, the caller owns removal
    /// from `toasts`; Sonner does not retain a local dismissal tombstone.
    #[props(default)]
    pub on_dismiss: Option<EventHandler<u64>>,
}

/// A root-level, safe-area-aware stack of toast notifications.
#[component]
pub fn Sonner(props: SonnerProps) -> Element {
    let theme = use_theme();
    let state = use_hook(|| Rc::new(RefCell::new(SonnerState::default())));
    let mut state_version = use_signal(|| 0_u64);
    let _ = state_version();
    let mut items = props.toasts;
    items.extend(state.borrow_mut().reconcile_legacy_messages(props.messages));
    let live = items.iter().map(ToastIdentity::from).collect::<Vec<_>>();
    {
        let mut state = state.borrow_mut();
        state.reconcile_dismissals(&live);
        items.retain(|toast| !state.is_dismissed(ToastIdentity::from(toast)));
    }

    let controlled_dismiss = props.on_dismiss;
    let dismiss_state = state.clone();
    let dismiss = EventHandler::new(move |identity: ToastIdentity| {
        if let Some(handler) = controlled_dismiss {
            handler.call(identity.id);
        } else if dismiss_state.borrow_mut().dismiss(identity) {
            state_version += 1;
        }
    });

    let presence = use_animate_presence(
        PresenceMode::PopLayout,
        items.iter().map(|toast| {
            (
                PresenceKey::new(format!("{}:{}", toast.id, toast.revision)),
                toast.clone(),
            )
        }),
    );
    let presence_entries = presence.entries();
    let painted = presence_entries
        .iter()
        .map(|entry| ToastPresenceItem {
            toast: entry.value.clone(),
            key: entry.key.as_str().to_string(),
            phase: entry.phase,
            popped: entry.popped_from_layout,
        })
        .collect::<Vec<_>>();
    let on_presence_terminal = {
        let presence = presence.clone();
        EventHandler::new(move |(key, phase): (String, PresencePhase)| {
            let key = PresenceKey::new(key);
            match phase {
                PresencePhase::Entering => {
                    presence.mark_present(&key);
                }
                PresencePhase::Leaving => {
                    presence.settle_exit(&key);
                }
                PresencePhase::Present => {}
            }
        })
    };
    let open = !painted.is_empty();
    // Timer ownership stays in Sonner rather than the visible overlay deck.
    // Collapsing, expanding, or clipping the deck must not restart or delay it.
    let timer_items = items.clone();
    let layer = rsx! {
        SonnerLayer {
            entries: painted,
            position: props.position,
            visible_toasts: props.visible_toasts.max(1),
            rich_colors: props.rich_colors,
            style: props.style,
            theme,
            on_dismiss: dismiss,
            on_presence_terminal,
        }
    };
    rsx! {
        for toast in timer_items {
            {
                let timer_key = format!("{}:{}", toast.id, toast.revision);
                rsx! {
                    SonnerToastTimer {
                        key: "{timer_key}",
                        toast,
                        on_dismiss: dismiss,
                    }
                }
            }
        }
        if open {
            arkit_hooks::Portal {
                layer: arkit_hooks::OverlayLayer::Transient,
                {layer}
            }
        }
    }
}

#[derive(Clone, PartialEq)]
struct ToastPresenceItem {
    toast: SonnerToast,
    key: String,
    phase: PresencePhase,
    popped: bool,
}

#[component]
fn SonnerLayer(
    entries: Vec<ToastPresenceItem>,
    position: SonnerPosition,
    visible_toasts: usize,
    rich_colors: bool,
    style: SonnerStyle,
    theme: Theme,
    on_dismiss: EventHandler<ToastIdentity>,
    on_presence_terminal: EventHandler<(String, PresencePhase)>,
) -> Element {
    let safe_area = arkit_hooks::use_safe_area();
    let viewport_width = viewport_width_vp();
    let available_width =
        (viewport_width - safe_area.left - safe_area.right - (style.inset * 2.0)).max(1.0);
    let notification_width = available_width.min(style.max_width.max(1.0));

    // Newest last in the source vec → reverse to newest-first for stacking.
    let ordered = entries.into_iter().rev().collect::<Vec<_>>();
    let mut notifications = Vec::new();
    let mut minimals = Vec::new();
    for entry in ordered {
        match entry.toast.appearance {
            ToastAppearance::Minimal => minimals.push(entry),
            ToastAppearance::Notification => notifications.push(entry),
        }
    }
    // Only the latest live minimal chip is shown — leaving chips still paint
    // so their hide timeline can finish. Extra live minimals stay hidden.
    let mut seen_live_minimal = false;
    minimals.retain(|entry| {
        if entry.popped {
            return true;
        }
        if seen_live_minimal {
            return false;
        }
        seen_live_minimal = true;
        true
    });

    let live_notifications = notifications.iter().filter(|entry| !entry.popped).count();
    let mut expanded = use_signal(|| false);
    if live_notifications <= 1 && expanded() {
        expanded.set(false);
    }
    let is_expanded = expanded() && live_notifications > 1;
    let visible_cap = visible_toasts.max(1);
    // Newest-first deck (index 0 = front), matching Sonner's toast index.
    // Leaving cards stay mounted but are popped from stack geometry.
    let mut live_deck = Vec::new();
    let mut leaving_deck = Vec::new();
    for entry in notifications {
        if entry.popped {
            leaving_deck.push(entry);
        } else if is_expanded || live_deck.len() < visible_cap {
            live_deck.push(entry);
        }
    }
    let count = live_deck.len();
    let stackable = count > 1;
    let is_top = position.is_top();
    let horizontal = position.horizontal();
    let swipe_direction = if is_top {
        ToastSwipeDirection::Up
    } else {
        ToastSwipeDirection::Down
    };
    let gap = if is_expanded {
        style.gap.max(0.0)
    } else {
        style.stack_offset.max(0.0)
    };
    // Absolute `position` is top-left of the stack. Layouts are computed as
    // distance-from-anchor then converted so the front toast stays on the
    // safe-area edge whether we pin top or bottom.
    let (layouts, stack_height) =
        sonner_stack_layouts(count, is_expanded, is_top, gap, notification_width);
    let layout_cache = use_hook(|| {
        Rc::new(RefCell::new(std::collections::HashMap::<
            String,
            SonnerCardLayout,
        >::new()))
    });
    {
        let mut cache = layout_cache.borrow_mut();
        for (index, entry) in live_deck.iter().enumerate() {
            if let Some(layout) = layouts.get(index).copied() {
                cache.insert(entry.key.clone(), layout);
            }
        }
        cache.retain(|key, _| {
            live_deck.iter().any(|entry| entry.key == *key)
                || leaving_deck.iter().any(|entry| entry.key == *key)
        });
    }
    // Always top-align inside the stack; bottom placement is done by the outer
    // column spacer so `position.y` stays a simple top-left coordinate.
    let stack_alignment = ALIGN_TOP;

    let mut expand_signal = expanded;
    let on_expand_change = EventHandler::new(move |next: bool| {
        expand_signal.set(next);
    });

    // Paint back peeks first so the front card is the topmost child. Leaving
    // cards draw last (highest z) so they can slide off over the restack.
    let painted = live_deck
        .into_iter()
        .enumerate()
        .rev()
        .collect::<Vec<(usize, ToastPresenceItem)>>();
    let has_notifications = !painted.is_empty() || !leaving_deck.is_empty();
    let has_minimals = !minimals.is_empty();

    rsx! {
        column {
            width: "100%",
            height: "100%",
            padding_top: safe_area.top + style.offset,
            padding_right: safe_area.right + style.inset,
            padding_bottom: safe_area.bottom + style.offset,
            padding_left: safe_area.left + style.inset,
            hit_test_behavior: "none",
            if !is_top {
                row {
                    width: "100%",
                    layout_weight: 1.0,
                    hit_test_behavior: "none",
                }
            }
            // Minimal chips sit above notifications when bottom-anchored so the
            // compact toast is never buried under the stack.
            if is_top {
                {render_minimal_row(minimals.clone(), horizontal, rich_colors, style.toast, theme, on_dismiss, on_presence_terminal, swipe_direction, is_top)}
                if has_minimals && has_notifications {
                    row { height: spacing::SM, hit_test_behavior: "none" }
                }
            }
            row {
                width: "100%",
                align_items: if is_top { "start" } else { "end" },
                hit_test_behavior: "none",
                if horizontal != HorizontalPosition::Left {
                    row { layout_weight: 1.0, hit_test_behavior: "none" }
                }
                if has_notifications {
                    stack {
                        width: notification_width,
                        height: stack_height,
                        alignment: stack_alignment,
                        clip: false,
                        hit_test_behavior: "none",
                        for (index, entry) in painted {
                            {
                                render_sonner_card(
                                    entry,
                                    layouts
                                        .get(index)
                                        .copied()
                                        .unwrap_or(SonnerCardLayout::front(notification_width)),
                                    index,
                                    is_expanded,
                                    stackable,
                                    rich_colors,
                                    swipe_direction,
                                    style.toast,
                                    theme,
                                    is_top,
                                    on_dismiss,
                                    on_presence_terminal,
                                    if index == 0 {
                                        Some(on_expand_change)
                                    } else {
                                        None
                                    },
                                )
                            }
                        }
                        for entry in leaving_deck {
                            {
                                let layout = layout_cache
                                    .borrow()
                                    .get(&entry.key)
                                    .copied()
                                    .unwrap_or(SonnerCardLayout::front(notification_width));
                                render_sonner_card(
                                    entry,
                                    layout,
                                    0,
                                    is_expanded,
                                    false,
                                    rich_colors,
                                    swipe_direction,
                                    style.toast,
                                    theme,
                                    is_top,
                                    on_dismiss,
                                    on_presence_terminal,
                                    None,
                                )
                            }
                        }
                    }
                }
                if horizontal != HorizontalPosition::Right {
                    row { layout_weight: 1.0, hit_test_behavior: "none" }
                }
            }
            if !is_top {
                if has_minimals && has_notifications {
                    row { height: spacing::SM, hit_test_behavior: "none" }
                }
                {render_minimal_row(minimals, horizontal, rich_colors, style.toast, theme, on_dismiss, on_presence_terminal, swipe_direction, is_top)}
            }
            if is_top {
                row {
                    width: "100%",
                    layout_weight: 1.0,
                    hit_test_behavior: "none",
                }
            }
        }
    }
}

/// Per-card geometry for the official Sonner stack model.
#[derive(Debug, Clone, Copy, PartialEq)]
struct SonnerCardLayout {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    /// Distance from the stack's anchor edge (top or bottom).
    offset: f32,
    stack_height: f32,
}

impl SonnerCardLayout {
    fn front(width: f32) -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width,
            height: FRONT_TOAST_HEIGHT,
            offset: 0.0,
            stack_height: FRONT_TOAST_HEIGHT,
        }
    }
}

fn stack_anchor_y(offset: f32, height: f32, stack_height: f32, is_top: bool) -> f32 {
    if is_top {
        offset
    } else {
        stack_height - height - offset
    }
}

/// Re-express `old` in `new`'s stack box so a bottom-aligned front card stays
/// put on screen when the stack grows upward.
fn stack_motion_origin(
    old: SonnerCardLayout,
    new: SonnerCardLayout,
    is_top: bool,
) -> SonnerCardLayout {
    SonnerCardLayout {
        x: old.x,
        y: stack_anchor_y(old.offset, old.height, new.stack_height, is_top),
        width: old.width,
        height: old.height,
        offset: old.offset,
        stack_height: new.stack_height,
    }
}

fn stack_easing() -> Easing {
    Easing::cubic_bezier(0.21, 1.02, 0.73, 1.0)
        .unwrap_or(Easing::Builtin(BuiltinEase::Cubic(EaseDirection::Out)))
}

fn configure_stack_tween(animation: Animation) -> Animation {
    animation.configure_last(
        stack_easing(),
        Default::default(),
        Default::default(),
        TimeSpan::ZERO,
        0,
    )
}

fn stack_layout_timeline(
    target: TargetName,
    from: SonnerCardLayout,
    to: SonnerCardLayout,
    duration: TimeSpan,
) -> Timeline {
    let animation = configure_stack_tween(Animation::new(AnimationSelector::Target(target)).tween(
        &POSITION_X,
        Length::vp(from.x),
        Length::vp(to.x),
        duration,
    ));
    let animation = configure_stack_tween(animation.tween(
        &POSITION_Y,
        Length::vp(from.y),
        Length::vp(to.y),
        duration,
    ));
    let animation = configure_stack_tween(animation.tween(
        &WIDTH,
        Length::vp(from.width),
        Length::vp(to.width),
        duration,
    ));
    Timeline::new().add(animation, TimelinePosition::START)
}

fn peek_opacity_timeline(target: TargetName, visible: bool, duration: TimeSpan) -> Timeline {
    let (from, to) = if visible { (0.0, 1.0) } else { (1.0, 0.0) };
    let animation = configure_stack_tween(
        Animation::new(AnimationSelector::Target(target)).tween(&OPACITY, from, to, duration),
    );
    Timeline::new().add(animation, TimelinePosition::START)
}

/// Interpolate a card from its previous stack slot into `layout`.
///
/// Destination coordinates are used for both ends so a bottom-anchored stack
/// can grow upward without the front toast jumping.
#[component]
fn SonnerStackSlot(
    layout: SonnerCardLayout,
    is_top: bool,
    skip_motion: bool,
    z_index: i32,
    hit_test_behavior: &'static str,
    children: Element,
) -> Element {
    let name = use_hook(|| TargetName::owned(format!("sonner-stack-{:?}", current_scope_id())));
    let target = use_animation_target(name.as_str().to_owned());
    let target_ref = target.native_ref();
    let duration = TimeSpan::from_millis(TOAST_STACK_MS.max(0) as u64);
    let last = use_hook(|| Rc::new(std::cell::Cell::new(None::<SonnerCardLayout>)));
    let controls = use_animation(stack_layout_timeline(
        name.clone(),
        layout,
        layout,
        duration,
    ));
    let request = (layout, is_top, skip_motion);
    use_effect(use_reactive((&request,), move |(request,)| {
        let layout = request.0;
        let is_top = request.1;
        let skip_motion = request.2;
        let previous = last.get();
        last.set(Some(layout));
        if skip_motion || !target.is_ready() || !controls.is_ready() {
            return;
        }
        let Some(previous) = previous else {
            return;
        };
        if previous == layout {
            return;
        }
        let from = stack_motion_origin(previous, layout, is_top);
        if (from.x - layout.x).abs() < 0.5
            && (from.y - layout.y).abs() < 0.5
            && (from.width - layout.width).abs() < 0.5
        {
            return;
        }
        controls.set_timeline(stack_layout_timeline(name.clone(), from, layout, duration));
        controls.restart();
    }));
    rsx! {
        column {
            native_ref: target_ref,
            width: layout.width,
            height: layout.height,
            position: format!("{},{}", layout.x, layout.y),
            z_index,
            hit_test_behavior,
            {children}
        }
    }
}

/// Fade toast internals when a collapsed peek becomes a full card.
#[component]
fn SonnerPeekContent(visible: bool, children: Element) -> Element {
    let name = use_hook(|| TargetName::owned(format!("sonner-peek-{:?}", current_scope_id())));
    let target = use_animation_target(name.as_str().to_owned());
    let target_ref = target.native_ref();
    let duration = TimeSpan::from_millis((TOAST_STACK_MS * 3 / 4).max(0) as u64);
    let last = use_hook(|| Rc::new(std::cell::Cell::new(None::<bool>)));
    let controls = use_animation(peek_opacity_timeline(name.clone(), visible, duration));
    use_effect(use_reactive((&visible,), move |(visible,)| {
        let previous = last.get();
        last.set(Some(visible));
        if !target.is_ready() || !controls.is_ready() {
            return;
        }
        if previous == Some(visible) {
            return;
        }
        if previous.is_none() {
            return;
        }
        controls.set_timeline(peek_opacity_timeline(name.clone(), visible, duration));
        controls.restart();
    }));
    rsx! {
        row {
            native_ref: target_ref,
            layout_weight: 1.0,
            align_items: "center",
            justify_content: "start",
            opacity: if visible { 1.0 } else { 0.0 },
            hit_test_behavior: if visible { "default" } else { "none" },
            {children}
        }
    }
}

/// Compute Sonner-style absolute layouts in **top-left** stack coordinates.
///
/// Official Sonner uses `bottom:0` + `translateY(-offset)` for bottom anchors.
/// ArkUI `position` is top-left, so we keep a logical distance-from-edge and
/// flip it for bottom placement:
/// `y = stack_height - card_height - offset_from_edge`.
///
/// Collapsed:
/// - offset = `index * gap`
/// - scale = `1 - index * 0.05` (width + centered x)
/// - non-front height locked to front height
///
/// Expanded:
/// - offset = `sum(heights_before) + index * gap`
/// - full width / natural height
fn sonner_stack_layouts(
    count: usize,
    expanded: bool,
    is_top: bool,
    gap: f32,
    full_width: f32,
) -> (Vec<SonnerCardLayout>, f32) {
    if count == 0 {
        return (Vec::new(), FRONT_TOAST_HEIGHT);
    }

    struct Logical {
        offset: f32,
        width: f32,
        height: f32,
        x: f32,
    }

    let mut logical = Vec::with_capacity(count);
    let mut height_before = 0.0_f32;
    for index in 0..count {
        if expanded {
            let height = EXPANDED_TOAST_HEIGHT;
            let offset = height_before + gap * index as f32;
            logical.push(Logical {
                offset,
                width: full_width,
                height,
                x: 0.0,
            });
            height_before += height;
        } else {
            let scale = (1.0 - index as f32 * STACK_SCALE_STEP).max(0.7);
            let width = (full_width * scale).max(1.0);
            let x = (full_width - width) * 0.5;
            logical.push(Logical {
                offset: gap * index as f32,
                width,
                height: FRONT_TOAST_HEIGHT,
                x,
            });
        }
    }

    // Span from the anchor edge through the farthest card.
    let stack_height = logical
        .iter()
        .map(|card| card.offset + card.height)
        .fold(FRONT_TOAST_HEIGHT, f32::max)
        .max(FRONT_TOAST_HEIGHT);

    let layouts = logical
        .into_iter()
        .map(|card| {
            let y = if is_top {
                // Front at y=0 (top edge); older cards push downward.
                card.offset
            } else {
                // Front flush with the bottom edge; older cards sit above it.
                stack_height - card.height - card.offset
            };
            SonnerCardLayout {
                x: card.x,
                y,
                width: card.width,
                height: card.height,
                offset: card.offset,
                stack_height,
            }
        })
        .collect();

    (layouts, stack_height)
}

fn render_sonner_card(
    entry: ToastPresenceItem,
    layout: SonnerCardLayout,
    index: usize,
    is_expanded: bool,
    stackable: bool,
    rich_colors: bool,
    swipe_direction: ToastSwipeDirection,
    style: ToastStyle,
    theme: Theme,
    is_top: bool,
    on_dismiss: EventHandler<ToastIdentity>,
    on_presence_terminal: EventHandler<(String, PresencePhase)>,
    on_expand_change: Option<EventHandler<bool>>,
) -> Element {
    let leaving = entry.phase == PresencePhase::Leaving;
    let is_front = index == 0 && !leaving;
    let entry_key = entry.key.clone();
    let hit_test = if leaving {
        HIT_TEST_NONE
    } else if is_front || is_expanded {
        HIT_TEST_DEFAULT
    } else {
        HIT_TEST_NONE
    };
    rsx! {
        SonnerStackSlot {
            key: "{entry_key}",
            layout,
            is_top,
            skip_motion: entry.phase == PresencePhase::Entering || leaving,
            z_index: if leaving { 200 } else { 100 - index as i32 },
            hit_test_behavior: hit_test,
            SonnerToastEntry {
                toast: entry.toast,
                phase: entry.phase,
                presence_key: entry.key,
                rich_colors,
                swipe_direction,
                stackable: is_front && stackable,
                expanded: is_expanded,
                stacked_back: !is_front && !is_expanded && !leaving,
                stacked_height: if !is_front && !is_expanded && !leaving {
                    Some(FRONT_TOAST_HEIGHT)
                } else {
                    None
                },
                interactive: !leaving && (is_front || is_expanded),
                is_top,
                style,
                theme,
                on_dismiss,
                on_presence_terminal,
                on_expand_change: if is_front { on_expand_change } else { None },
            }
        }
    }
}

fn render_minimal_row(
    minimals: Vec<ToastPresenceItem>,
    horizontal: HorizontalPosition,
    rich_colors: bool,
    style: ToastStyle,
    theme: Theme,
    on_dismiss: EventHandler<ToastIdentity>,
    on_presence_terminal: EventHandler<(String, PresencePhase)>,
    swipe_direction: ToastSwipeDirection,
    is_top: bool,
) -> Element {
    if minimals.is_empty() {
        return rsx! {};
    }
    rsx! {
        row {
            width: "100%",
            align_items: "center",
            hit_test_behavior: "none",
            if horizontal != HorizontalPosition::Left {
                row { layout_weight: 1.0, hit_test_behavior: "none" }
            }
            column {
                align_items: "center",
                hit_test_behavior: "none",
                for entry in minimals {
                    {
                        let leaving = entry.phase == PresencePhase::Leaving;
                        let entry_key = entry.key.clone();
                        rsx! {
                            column {
                                key: "{entry_key}",
                                hit_test_behavior: if leaving { HIT_TEST_NONE } else { HIT_TEST_DEFAULT },
                                SonnerToastEntry {
                                    toast: entry.toast,
                                    phase: entry.phase,
                                    presence_key: entry.key,
                                    rich_colors,
                                    swipe_direction,
                                    stackable: false,
                                    expanded: false,
                                    stacked_back: false,
                                    stacked_height: None,
                                    interactive: !leaving,
                                    is_top,
                                    style,
                                    theme,
                                    on_dismiss,
                                    on_presence_terminal,
                                    on_expand_change: None,
                                }
                            }
                        }
                    }
                }
            }
            if horizontal != HorizontalPosition::Right {
                row { layout_weight: 1.0, hit_test_behavior: "none" }
            }
        }
    }
}

#[component]
fn SonnerToastEntry(
    toast: SonnerToast,
    phase: PresencePhase,
    presence_key: String,
    rich_colors: bool,
    swipe_direction: ToastSwipeDirection,
    stackable: bool,
    expanded: bool,
    stacked_back: bool,
    stacked_height: Option<f32>,
    interactive: bool,
    is_top: bool,
    style: ToastStyle,
    theme: Theme,
    on_dismiss: EventHandler<ToastIdentity>,
    on_presence_terminal: EventHandler<(String, PresencePhase)>,
    on_expand_change: Option<EventHandler<bool>>,
) -> Element {
    let identity = ToastIdentity::from(&toast);
    let action_handler = toast.action_handler.clone();
    let dismiss_handler = toast.dismiss_handler.clone();
    let terminal_key = presence_key.clone();
    let on_terminal = EventHandler::new(move |phase: PresencePhase| {
        on_presence_terminal.call((terminal_key.clone(), phase));
    });

    let action_dismiss_callback = dismiss_handler.clone();
    let action = EventHandler::new(move |_: ()| {
        if !interactive {
            return;
        }
        if let Some(handler) = action_handler.as_ref() {
            handler();
        }
        if let Some(handler) = action_dismiss_callback.as_ref() {
            handler();
        }
        on_dismiss.call(identity);
    });
    let dismiss = EventHandler::new(move |_: ()| {
        if !interactive {
            return;
        }
        if let Some(handler) = dismiss_handler.as_ref() {
            handler();
        }
        on_dismiss.call(identity);
    });
    let expand = on_expand_change.map(|handler| {
        EventHandler::new(move |next: bool| {
            if interactive {
                handler.call(next);
            }
        })
    });

    rsx! {
        ThemeProvider {
            theme,
            PresenceTransition {
                phase,
                on_terminal,
                preset: Some(if is_top {
                    TransitionPreset::SlideDown
                } else {
                    TransitionPreset::SlideUp
                }),
                duration_ms: Some(TOAST_ENTER_MS),
                exit_duration_ms: Some(TOAST_EXIT_MS),
                distance: Some(TOAST_DISTANCE),
                Toast {
                    message: toast.title,
                    description: toast.description,
                    variant: toast.variant,
                    appearance: toast.appearance,
                    action_label: toast.action_label,
                    icon: toast.icon,
                    dismissible: toast.dismissible && interactive,
                    rich_colors,
                    swipe_direction,
                    stackable,
                    expanded,
                    stacked_back,
                    stacked_height,
                    style,
                    on_action: action,
                    on_dismiss: dismiss,
                    on_expand_change: expand,
                }
            }
        }
    }
}

/// Lifetime owner for one toast timer. This component is keyed by
/// `(id, revision)` and remains mounted even when the corresponding card is
/// outside the collapsed visual cap.
#[component]
fn SonnerToastTimer(toast: SonnerToast, on_dismiss: EventHandler<ToastIdentity>) -> Element {
    let identity = ToastIdentity::from(&toast);
    let duration_ms = toast.duration_ms;
    let timer_callback = toast.dismiss_handler.clone();
    let async_runtime = arkit_runtime::use_runtime_handle().tokio();
    let _dismiss_timer = use_future(move || {
        let async_runtime = async_runtime.clone();
        let timer_callback = timer_callback.clone();
        async move {
            if duration_ms == 0 {
                return;
            }
            let timer = async_runtime.spawn(async move {
                tokio::time::sleep(Duration::from_millis(duration_ms)).await;
            });
            if timer.await.is_ok() {
                if let Some(handler) = timer_callback {
                    handler();
                }
                on_dismiss.call(identity);
            }
        }
    });

    rsx! {}
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct ToastPalette {
    background: u32,
    foreground: u32,
    description: u32,
    border: u32,
    icon: u32,
    action_background: u32,
    action_foreground: u32,
}

fn toast_palette(
    variant: ToastVariant,
    theme: Theme,
    rich_colors: bool,
    style: ToastStyle,
    minimal: bool,
) -> ToastPalette {
    let semantic = semantic_palette(variant, theme.mode);
    // Minimal shares the shadcn popover surface (border + shadow), not a
    // separate inverted system pill. Rich colors still tint semantic variants.
    let mut palette = if rich_colors && variant != ToastVariant::Default {
        semantic
    } else {
        ToastPalette {
            background: theme.colors.popover,
            foreground: theme.colors.popover_foreground,
            description: theme.colors.muted_foreground,
            border: theme.colors.border,
            icon: semantic.icon,
            action_background: theme.colors.primary,
            action_foreground: theme.colors.primary_foreground,
        }
    };
    if minimal && !rich_colors {
        palette.icon = theme.colors.muted_foreground;
    }
    palette.background = style.background_color.unwrap_or(palette.background);
    palette.foreground = style.foreground_color.unwrap_or(palette.foreground);
    palette.description = style.description_color.unwrap_or(palette.description);
    palette.border = style.border_color.unwrap_or(palette.border);
    palette.icon = style.icon_color.unwrap_or(palette.icon);
    palette.action_background = style
        .action_background_color
        .unwrap_or(palette.action_background);
    palette.action_foreground = style
        .action_foreground_color
        .unwrap_or(palette.action_foreground);
    palette
}

fn semantic_palette(variant: ToastVariant, mode: ThemeMode) -> ToastPalette {
    let (background, foreground, description, border, icon) = match (variant, mode) {
        (ToastVariant::Success, ThemeMode::Light) => {
            (0xFFF0FDF4, 0xFF166534, 0xFF15803D, 0xFFBBF7D0, 0xFF16A34A)
        }
        (ToastVariant::Success, ThemeMode::Dark) => {
            (0xFF052E16, 0xFFBBF7D0, 0xFF86EFAC, 0xFF166534, 0xFF4ADE80)
        }
        (ToastVariant::Info, ThemeMode::Light) => {
            (0xFFEFF6FF, 0xFF1E40AF, 0xFF1D4ED8, 0xFFBFDBFE, 0xFF2563EB)
        }
        (ToastVariant::Info, ThemeMode::Dark) => {
            (0xFF172554, 0xFFBFDBFE, 0xFF93C5FD, 0xFF1E40AF, 0xFF60A5FA)
        }
        (ToastVariant::Warning, ThemeMode::Light) => {
            (0xFFFFFBEB, 0xFF92400E, 0xFFB45309, 0xFFFDE68A, 0xFFD97706)
        }
        (ToastVariant::Warning, ThemeMode::Dark) => {
            (0xFF451A03, 0xFFFDE68A, 0xFFFCD34D, 0xFF92400E, 0xFFFBBF24)
        }
        (ToastVariant::Error, ThemeMode::Light) => {
            (0xFFFEF2F2, 0xFF991B1B, 0xFFB91C1C, 0xFFFECACA, 0xFFDC2626)
        }
        (ToastVariant::Error, ThemeMode::Dark) => {
            (0xFF450A0A, 0xFFFECACA, 0xFFFCA5A5, 0xFF991B1B, 0xFFF87171)
        }
        (_, ThemeMode::Light) => (0xFFFFFFFF, 0xFF09090B, 0xFF71717A, 0xFFE4E4E7, 0xFF71717A),
        (_, ThemeMode::Dark) => (0xFF18181B, 0xFFFAFAFA, 0xFFA1A1AA, 0xFF27272A, 0xFFA1A1AA),
    };
    ToastPalette {
        background,
        foreground,
        description,
        border,
        icon,
        action_background: foreground,
        action_foreground: background,
    }
}

fn variant_icon(variant: ToastVariant) -> Option<String> {
    let name = match variant {
        ToastVariant::Default | ToastVariant::Loading => return None,
        ToastVariant::Success => "circle-check",
        ToastVariant::Info => "info",
        ToastVariant::Warning => "triangle-alert",
        ToastVariant::Error => "circle-x",
    };
    Some(name.to_string())
}

fn should_dismiss_swipe(delta_x: f32, delta_y: f32, direction: ToastSwipeDirection) -> bool {
    let vertical = match direction {
        ToastSwipeDirection::Up => -delta_y >= SWIPE_DISMISS_THRESHOLD,
        ToastSwipeDirection::Down => delta_y >= SWIPE_DISMISS_THRESHOLD,
    };
    vertical || delta_x.abs() >= HORIZONTAL_SWIPE_DISMISS_THRESHOLD
}

/// Expand / collapse the notification stack.
///
/// Bottom anchor: swipe up expands, swipe down collapses (when already expanded).
/// Top anchor is mirrored. Horizontal gestures are ignored so dismiss can own them.
fn stack_expand_gesture(
    delta_x: f32,
    delta_y: f32,
    direction: ToastSwipeDirection,
    expanded: bool,
) -> Option<bool> {
    if delta_x.abs() >= HORIZONTAL_SWIPE_DISMISS_THRESHOLD && delta_x.abs() >= delta_y.abs() {
        return None;
    }
    let expand_axis = match direction {
        ToastSwipeDirection::Down => -delta_y >= STACK_EXPAND_THRESHOLD,
        ToastSwipeDirection::Up => delta_y >= STACK_EXPAND_THRESHOLD,
    };
    let collapse_axis = match direction {
        ToastSwipeDirection::Down => delta_y >= STACK_EXPAND_THRESHOLD,
        ToastSwipeDirection::Up => -delta_y >= STACK_EXPAND_THRESHOLD,
    };
    if !expanded && expand_axis {
        Some(true)
    } else if expanded && collapse_axis {
        Some(false)
    } else {
        None
    }
}

fn handle_toast_touch(
    event: dioxus_core::Event<arkit_prelude::event::PointerData>,
    drag_start: &mut Signal<Option<(f32, f32)>>,
    swipe_direction: ToastSwipeDirection,
    stackable: bool,
    expanded: bool,
    on_expand_change: Option<EventHandler<bool>>,
    on_dismiss: Option<EventHandler<()>>,
) {
    let Some(pointer) = event.data().pointer else {
        return;
    };
    let x = if pointer.has_window_position() {
        pointer.window_x
    } else {
        pointer.x
    };
    let y = if pointer.has_window_position() {
        pointer.window_y
    } else {
        pointer.y
    };
    match pointer.action {
        dioxus_elements::event::PointerAction::Down => {
            drag_start.set(Some((x, y)));
        }
        dioxus_elements::event::PointerAction::Up => {
            let Some((start_x, start_y)) = drag_start() else {
                return;
            };
            drag_start.set(None);
            let delta_x = x - start_x;
            let delta_y = y - start_y;
            if stackable {
                if let Some(next) =
                    stack_expand_gesture(delta_x, delta_y, swipe_direction, expanded)
                {
                    if let Some(handler) = on_expand_change {
                        handler.call(next);
                    }
                    return;
                }
            }
            if should_dismiss_swipe(delta_x, delta_y, swipe_direction) {
                if let Some(handler) = on_dismiss {
                    handler.call(());
                }
            }
        }
        dioxus_elements::event::PointerAction::Cancel => {
            drag_start.set(None);
        }
        dioxus_elements::event::PointerAction::Move
        | dioxus_elements::event::PointerAction::Unknown => {}
    }
}

fn minimal_chip_width(message: &str, has_icon: bool) -> f32 {
    // Approximate content width: ~7.5vp per glyph at 14sp, plus chip padding.
    let text = (message.chars().count() as f32 * 7.5).clamp(24.0, MINIMAL_MAX_WIDTH - 40.0);
    let icon = if has_icon { 20.0 } else { 0.0 };
    (text + icon + 28.0).clamp(MINIMAL_MIN_WIDTH, MINIMAL_MAX_WIDTH)
}

fn display_vp_ratio() -> f32 {
    let ratio = ohos_display_binding::default_display_virtual_pixel_ratio();
    if ratio.is_finite() && ratio > 0.0 {
        ratio
    } else {
        1.0
    }
}

fn viewport_width_vp() -> f32 {
    let metrics = arkit_hooks::use_window_metrics();
    if metrics.content_rect.width > 0 && metrics.scale.is_finite() && metrics.scale > 0.0 {
        metrics.content_rect.width as f32 / metrics.scale
    } else {
        ohos_display_binding::default_display_width() as f32 / display_vp_ratio()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mobile_defaults_anchor_to_bottom_center() {
        let style = SonnerStyle::default();
        assert_eq!(SonnerPosition::default(), SonnerPosition::BottomCenter);
        assert_eq!(style.max_width, 420.0);
        assert_eq!(style.inset, 16.0);
        assert_eq!(style.offset, 16.0);
        assert_eq!(style.gap, 8.0);
        assert_eq!(style.stack_offset, DEFAULT_STACK_OFFSET);
        assert_eq!(style.stack_offset, 14.0);
    }

    #[test]
    fn sonner_collapsed_bottom_keeps_front_on_bottom_edge() {
        let (layouts, stack_height) = sonner_stack_layouts(3, false, false, 14.0, 360.0);
        assert_eq!(layouts.len(), 3);
        assert_eq!(stack_height, FRONT_TOAST_HEIGHT + 28.0);
        // Front flush with bottom of the stack box.
        assert_eq!(layouts[0].y, stack_height - FRONT_TOAST_HEIGHT);
        assert_eq!(layouts[0].width, 360.0);
        // Older peeks sit above (smaller y) with scale.
        assert!(layouts[1].y < layouts[0].y);
        assert!((layouts[1].width - 360.0 * 0.95).abs() < 0.01);
        assert!(layouts[2].y < layouts[1].y);
        assert!((layouts[2].width - 360.0 * 0.90).abs() < 0.01);
    }

    #[test]
    fn sonner_collapsed_top_keeps_front_on_top_edge() {
        let (layouts, _) = sonner_stack_layouts(3, false, true, 14.0, 360.0);
        assert_eq!(layouts[0].y, 0.0);
        assert_eq!(layouts[1].y, 14.0);
        assert_eq!(layouts[2].y, 28.0);
    }

    #[test]
    fn sonner_expanded_bottom_stacks_upward_from_front() {
        let (layouts, stack_height) = sonner_stack_layouts(3, true, false, 14.0, 360.0);
        let h = EXPANDED_TOAST_HEIGHT;
        assert_eq!(stack_height, 3.0 * h + 2.0 * 14.0);
        // Newest (index 0) on the bottom edge.
        assert_eq!(layouts[0].y, stack_height - h);
        assert_eq!(layouts[0].width, 360.0);
        // Older cards above it with gap.
        assert_eq!(layouts[1].y, stack_height - h - (h + 14.0));
        assert_eq!(layouts[2].y, stack_height - h - 2.0 * (h + 14.0));
        assert!(layouts[2].y < layouts[1].y && layouts[1].y < layouts[0].y);
    }

    #[test]
    fn sonner_expanded_top_stacks_downward_from_front() {
        let (layouts, _) = sonner_stack_layouts(3, true, true, 14.0, 360.0);
        let h = EXPANDED_TOAST_HEIGHT;
        assert_eq!(layouts[0].y, 0.0);
        assert_eq!(layouts[1].y, h + 14.0);
        assert_eq!(layouts[2].y, 2.0 * (h + 14.0));
    }

    #[test]
    fn stack_motion_keeps_bottom_front_card_on_screen() {
        let (collapsed, _) = sonner_stack_layouts(3, false, false, 14.0, 360.0);
        let (expanded, _) = sonner_stack_layouts(3, true, false, 14.0, 360.0);
        let origin = stack_motion_origin(collapsed[0], expanded[0], false);
        assert!(
            (origin.y - expanded[0].y).abs() < 0.01,
            "front card must not jump when the stack box grows"
        );
        let mid = stack_motion_origin(collapsed[1], expanded[1], false);
        assert!(
            mid.y > expanded[1].y,
            "back cards start closer to the front and travel away"
        );
    }

    #[test]
    fn stack_motion_keeps_top_front_card_on_screen() {
        let (collapsed, _) = sonner_stack_layouts(3, false, true, 14.0, 360.0);
        let (expanded, _) = sonner_stack_layouts(3, true, true, 14.0, 360.0);
        let origin = stack_motion_origin(collapsed[0], expanded[0], true);
        assert!((origin.y - expanded[0].y).abs() < 0.01);
        assert_eq!(origin.y, 0.0);
    }

    #[test]
    fn loading_toast_is_persistent_by_default() {
        let toast = SonnerToast::loading(7, "Uploading");
        assert_eq!(toast.variant, ToastVariant::Loading);
        assert_eq!(toast.duration_ms, 0);
        assert!(toast.dismissible);
    }

    #[test]
    fn minimal_toast_skips_notification_chrome() {
        let toast = SonnerToast::minimal(3, "Copied");
        assert_eq!(toast.appearance, ToastAppearance::Minimal);
        assert!(!toast.dismissible);
        assert_eq!(toast.duration_ms, DEFAULT_DURATION_MS);
    }

    #[test]
    fn revision_can_restart_a_controlled_toast_entry() {
        let toast = SonnerToast::success(7, "Uploaded").revision(3);
        assert_eq!(toast.id, 7);
        assert_eq!(toast.revision, 3);
    }

    #[test]
    fn dismissal_tombstone_is_released_after_source_removal() {
        let identity = ToastIdentity { id: 7, revision: 0 };
        let mut state = SonnerState::default();

        assert!(state.dismiss(identity));
        state.reconcile_dismissals(&[identity]);
        assert!(state.is_dismissed(identity));

        state.reconcile_dismissals(&[]);
        assert!(!state.is_dismissed(identity));
        assert!(state.dismiss(identity), "the removed id must be reusable");
    }

    #[test]
    fn a_new_revision_does_not_inherit_an_old_dismissal() {
        let old = ToastIdentity { id: 7, revision: 0 };
        let updated = ToastIdentity { id: 7, revision: 1 };
        let mut state = SonnerState::default();

        assert!(state.dismiss(old));
        state.reconcile_dismissals(&[updated]);

        assert!(!state.is_dismissed(old));
        assert!(!state.is_dismissed(updated));
    }

    #[test]
    fn replacing_a_legacy_message_advances_its_identity() {
        let mut state = SonnerState::default();
        let first = state.reconcile_legacy_messages(vec!["first".to_string()]);
        let first_identity = ToastIdentity::from(&first[0]);
        assert!(state.dismiss(first_identity));

        let second = state.reconcile_legacy_messages(vec!["second".to_string()]);
        let second_identity = ToastIdentity::from(&second[0]);
        state.reconcile_dismissals(&[second_identity]);

        assert_eq!(second_identity.id, first_identity.id);
        assert_eq!(second_identity.revision, first_identity.revision + 1);
        assert!(!state.is_dismissed(second_identity));
    }

    #[test]
    fn swipe_thresholds_use_logical_viewport_units() {
        assert!(should_dismiss_swipe(0.0, 56.0, ToastSwipeDirection::Down));
        assert!(should_dismiss_swipe(-72.0, 0.0, ToastSwipeDirection::Down));
        assert!(!should_dismiss_swipe(71.0, 55.0, ToastSwipeDirection::Down));
        assert!(should_dismiss_swipe(0.0, -56.0, ToastSwipeDirection::Up));
    }

    #[test]
    fn stack_expand_gesture_is_stable_and_axis_aligned() {
        assert_eq!(
            stack_expand_gesture(0.0, -40.0, ToastSwipeDirection::Down, false),
            Some(true)
        );
        assert_eq!(
            stack_expand_gesture(0.0, 40.0, ToastSwipeDirection::Down, true),
            Some(false)
        );
        assert_eq!(
            stack_expand_gesture(80.0, -10.0, ToastSwipeDirection::Down, false),
            None
        );
    }

    #[test]
    fn minimal_chip_width_hugs_short_copy() {
        let width = minimal_chip_width("Copied", false);
        assert!(width < 140.0, "short chip should stay compact, got {width}");
        assert!(width >= MINIMAL_MIN_WIDTH);
        let long = minimal_chip_width("This is a fairly long status message", true);
        assert!(long <= MINIMAL_MAX_WIDTH);
    }

    #[test]
    fn rich_error_palette_uses_semantic_surface() {
        let theme = Theme::default();
        let palette = toast_palette(
            ToastVariant::Error,
            theme,
            true,
            ToastStyle::default(),
            false,
        );
        assert_eq!(palette.background, 0xFFFEF2F2);
        assert_eq!(palette.icon, 0xFFDC2626);
    }

    #[test]
    fn card_overrides_win_over_semantic_colors() {
        let palette = toast_palette(
            ToastVariant::Success,
            Theme::default(),
            true,
            ToastStyle {
                background_color: Some(0xFF123456),
                icon_color: Some(0xFFABCDEF),
                ..ToastStyle::default()
            },
            false,
        );
        assert_eq!(palette.background, 0xFF123456);
        assert_eq!(palette.icon, 0xFFABCDEF);
    }

    #[test]
    fn minimal_palette_uses_shadcn_popover_surface() {
        let theme = Theme::default();
        let palette = toast_palette(
            ToastVariant::Default,
            theme,
            false,
            ToastStyle::default(),
            true,
        );
        assert_eq!(palette.background, theme.colors.popover);
        assert_eq!(palette.foreground, theme.colors.popover_foreground);
        assert_eq!(palette.border, theme.colors.border);
    }
}
