//! Sonner-style toast surfaces.
//!
//! `Sonner` owns the viewport-level stack while callers own the toast data.
//! The default presentation is mobile-first: a bottom-center stack inset from
//! the horizontal edges and the system safe area, 40vp action targets, and
//! swipe-to-dismiss in the direction of the selected position.

use std::time::Duration;
use std::{fmt, rc::Rc};

use super::floating_layer::SHADOW_SM;
use super::{Spinner, ARKUI_BORDER_STYLE_SOLID, ARKUI_BUTTON_TYPE_NORMAL};
use crate::icon::icon_placeholder;
use crate::theme::*;
use arkit_prelude::*;

const DEFAULT_DURATION_MS: u64 = 4_000;
const DEFAULT_MAX_WIDTH: f32 = 420.0;
const DEFAULT_MIN_HEIGHT: f32 = 64.0;
const TOAST_ICON_SIZE: f32 = 20.0;
const TOAST_ACTION_HEIGHT: f32 = 40.0;
const TOAST_CLOSE_SIZE: f32 = 40.0;
const SWIPE_DISMISS_THRESHOLD: f32 = 56.0;
const HORIZONTAL_SWIPE_DISMISS_THRESHOLD: f32 = 72.0;

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
    pub shadow: Option<i32>,
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
    /// Space between visible toast cards.
    pub gap: f32,
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
            toast: ToastStyle::default(),
        }
    }
}

/// One controlled item in a [`Sonner`] stack.
#[derive(Clone)]
pub struct SonnerToast {
    /// Stable identity. Reusing an id for two live toasts is unsupported.
    pub id: u64,
    pub title: String,
    pub description: Option<String>,
    pub variant: ToastVariant,
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
            .field("title", &self.title)
            .field("description", &self.description)
            .field("variant", &self.variant)
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
            && self.title == other.title
            && self.description == other.description
            && self.variant == other.variant
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
            title: title.into(),
            description: None,
            variant: ToastVariant::Default,
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

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub const fn variant(mut self, variant: ToastVariant) -> Self {
        self.variant = variant;
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
    pub action_label: Option<String>,
    #[props(default)]
    pub icon: Option<String>,
    #[props(default = true)]
    pub dismissible: bool,
    #[props(default)]
    pub rich_colors: bool,
    #[props(default)]
    pub swipe_direction: ToastSwipeDirection,
    #[props(default)]
    pub style: ToastStyle,
    #[props(default)]
    pub on_action: Option<EventHandler<()>>,
    #[props(default)]
    pub on_dismiss: Option<EventHandler<()>>,
}

/// A shadcn-styled mobile toast card.
#[component]
pub fn Toast(props: ToastProps) -> Element {
    let theme = use_theme();
    let palette = toast_palette(props.variant, theme, props.rich_colors, props.style);
    let mut drag_start = use_signal(|| None::<(f32, f32)>);
    let on_action = props.on_action;
    let on_dismiss = props.on_dismiss;
    let swipe_direction = props.swipe_direction;
    let icon = props.icon.or_else(|| variant_icon(props.variant));
    let border_radius = props.style.border_radius.unwrap_or(theme.radii.lg);
    let min_height = props.style.min_height.unwrap_or(DEFAULT_MIN_HEIGHT);
    let shadow = props.style.shadow.unwrap_or(SHADOW_SM);

    rsx! {
        row {
            percent_width: 1.0,
            constraint_size: format!("0,100000,{min_height},100000"),
            align_items: "center",
            padding_top: spacing::MD,
            padding_right: spacing::SM,
            padding_bottom: spacing::MD,
            padding_left: spacing::MD,
            background_color: palette.background,
            border_width: 1.0,
            border_color: palette.border,
            border_style: ARKUI_BORDER_STYLE_SOLID,
            border_radius,
            shadow,
            clip: true,
            // The Sonner layer is intentionally pass-through. Re-enable hit
            // testing on the card itself so ArkUI delivers touch sequences to
            // the swipe recognizer while the empty overlay remains inert.
            hit_test_behavior: 0_i32,
            on_touch: move |event| {
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
                    dioxus_elements::event::PointerAction::Move
                    | dioxus_elements::event::PointerAction::Up => {
                        let Some((start_x, start_y)) = drag_start() else {
                            return;
                        };
                        // ArkUI pointer coordinates are already expressed in
                        // logical viewport units, matching component sizes.
                        let delta_x = x - start_x;
                        let delta_y = y - start_y;
                        if should_dismiss_swipe(delta_x, delta_y, swipe_direction) {
                            drag_start.set(None);
                            if let Some(handler) = on_dismiss {
                                handler.call(());
                            }
                        } else if matches!(
                            pointer.action,
                            dioxus_elements::event::PointerAction::Up
                        ) {
                            drag_start.set(None);
                        }
                    }
                    dioxus_elements::event::PointerAction::Cancel => drag_start.set(None),
                    dioxus_elements::event::PointerAction::Unknown => {}
                }
            },
            if props.variant == ToastVariant::Loading {
                row {
                    width: 28.0,
                    height: TOAST_CLOSE_SIZE,
                    align_items: "center",
                    justify_content: "start",
                    Spinner {
                        size: TOAST_ICON_SIZE,
                        color: Some(palette.icon),
                    }
                }
            } else if let Some(icon_name) = icon {
                row {
                    width: 28.0,
                    height: TOAST_CLOSE_SIZE,
                    align_items: "center",
                    justify_content: "start",
                    {icon_placeholder(&icon_name, TOAST_ICON_SIZE, palette.icon)}
                }
            }
            column {
                layout_weight: 1.0,
                align_items: "start",
                justify_content: "center",
                text {
                    percent_width: 1.0,
                    content: props.message,
                    font_size: typography::SM,
                    font_weight: 600_i32,
                    font_color: palette.foreground,
                    line_height: 20.0,
                    max_lines: 2_i32,
                    text_overflow: 2_i32,
                }
                if let Some(description) = props.description {
                    row { height: spacing::XXS }
                    text {
                        percent_width: 1.0,
                        content: description,
                        font_size: typography::XS,
                        font_weight: 400_i32,
                        font_color: palette.description,
                        line_height: 18.0,
                        max_lines: 3_i32,
                        text_overflow: 2_i32,
                    }
                }
            }
            if let Some(action_label) = props.action_label {
                row { width: spacing::SM }
                button {
                    button_type: ARKUI_BUTTON_TYPE_NORMAL,
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
                    alignment: 4_i32,
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
            if props.dismissible {
                button {
                    button_type: ARKUI_BUTTON_TYPE_NORMAL,
                    width: TOAST_CLOSE_SIZE,
                    height: TOAST_CLOSE_SIZE,
                    padding: 0.0,
                    background_color: 0x00000000,
                    border_width: 0.0,
                    border_style: ARKUI_BORDER_STYLE_SOLID,
                    border_radius: theme.radii.md,
                    focusable: false,
                    focus_on_touch: false,
                    alignment: 4_i32,
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
    #[props(default = 3usize)]
    pub visible_toasts: usize,
    #[props(default)]
    pub rich_colors: bool,
    #[props(default)]
    pub style: SonnerStyle,
}

/// A root-level, safe-area-aware stack of toast notifications.
#[component]
pub fn Sonner(props: SonnerProps) -> Element {
    let mut dismissed = use_signal(Vec::<u64>::new);
    let mut items = props.toasts;
    items.extend(
        props
            .messages
            .into_iter()
            .enumerate()
            .map(|(index, message)| {
                SonnerToast::new(u64::MAX.saturating_sub(index as u64), message)
            }),
    );
    items.retain(|toast| !dismissed().contains(&toast.id));

    let dismiss = EventHandler::new(move |id: u64| {
        dismissed.with_mut(|ids| {
            if !ids.contains(&id) {
                ids.push(id);
            }
        });
    });

    let layer = rsx! {
        SonnerLayer {
            toasts: items.clone(),
            position: props.position,
            visible_toasts: props.visible_toasts.max(1),
            rich_colors: props.rich_colors,
            style: props.style,
            on_dismiss: dismiss,
        }
    };
    use_sonner_overlay(!items.is_empty(), layer);
    rsx! {}
}

fn use_sonner_overlay(open: bool, layer: Element) {
    let overlay = arkit_hooks::use_overlay();
    let effect_overlay = overlay.clone();
    let effect_layer = layer.clone();
    let mut refresh = use_effect(move || {
        if open {
            let layer = effect_layer.clone();
            effect_overlay.show_floating(move || layer.clone());
        } else {
            effect_overlay.dismiss();
        }
    });
    refresh.mark_dirty();

    let cleanup_overlay = overlay.clone();
    use_drop(move || cleanup_overlay.dismiss());
}

#[component]
fn SonnerLayer(
    toasts: Vec<SonnerToast>,
    position: SonnerPosition,
    visible_toasts: usize,
    rich_colors: bool,
    style: SonnerStyle,
    on_dismiss: EventHandler<u64>,
) -> Element {
    let safe_area = arkit_hooks::use_safe_area();
    let viewport_width = viewport_width_vp();
    let available_width =
        (viewport_width - safe_area.left - safe_area.right - (style.inset * 2.0)).max(1.0);
    let toast_width = available_width.min(style.max_width.max(1.0));
    let is_top = position.is_top();
    let horizontal = position.horizontal();
    let swipe_direction = if is_top {
        ToastSwipeDirection::Up
    } else {
        ToastSwipeDirection::Down
    };
    let mut visible = toasts
        .into_iter()
        .rev()
        .take(visible_toasts)
        .collect::<Vec<_>>();
    if !is_top {
        visible.reverse();
    }
    let stack = visible
        .into_iter()
        .enumerate()
        .collect::<Vec<(usize, SonnerToast)>>();

    rsx! {
        column {
            percent_width: 1.0,
            percent_height: 1.0,
            padding_top: safe_area.top + style.offset,
            padding_right: safe_area.right + style.inset,
            padding_bottom: safe_area.bottom + style.offset,
            padding_left: safe_area.left + style.inset,
            hit_test_behavior: 2_i32,
            if !is_top {
                row {
                    percent_width: 1.0,
                    layout_weight: 1.0,
                    hit_test_behavior: 2_i32,
                }
            }
            row {
                percent_width: 1.0,
                align_items: "end",
                hit_test_behavior: 2_i32,
                if horizontal != HorizontalPosition::Left {
                    row { layout_weight: 1.0, hit_test_behavior: 2_i32 }
                }
                column {
                    width: toast_width,
                    align_items: "start",
                    for (index, toast) in stack {
                        column {
                            key: "{toast.id}",
                            percent_width: 1.0,
                            if index > 0 {
                                row { height: style.gap, hit_test_behavior: 2_i32 }
                            }
                            SonnerToastEntry {
                                toast,
                                rich_colors,
                                swipe_direction,
                                style: style.toast,
                                on_dismiss,
                            }
                        }
                    }
                }
                if horizontal != HorizontalPosition::Right {
                    row { layout_weight: 1.0, hit_test_behavior: 2_i32 }
                }
            }
            if is_top {
                row {
                    percent_width: 1.0,
                    layout_weight: 1.0,
                    hit_test_behavior: 2_i32,
                }
            }
        }
    }
}

#[component]
fn SonnerToastEntry(
    toast: SonnerToast,
    rich_colors: bool,
    swipe_direction: ToastSwipeDirection,
    style: ToastStyle,
    on_dismiss: EventHandler<u64>,
) -> Element {
    let id = toast.id;
    let duration_ms = toast.duration_ms;
    let action_handler = toast.action_handler.clone();
    let dismiss_handler = toast.dismiss_handler.clone();
    let timer_dismiss = on_dismiss;
    let timer_callback = dismiss_handler.clone();
    // Capture the runtime handle while rendering on the registered UI thread.
    // `tokio_handle()` is thread-local; resolving it lazily from inside a
    // Dioxus task can run outside that registration and silently stop the task.
    let async_runtime = arkit_runtime::tokio_handle();
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
                timer_dismiss.call(id);
            }
        }
    });

    let action_dismiss_callback = dismiss_handler.clone();
    let action = EventHandler::new(move |_: ()| {
        if let Some(handler) = action_handler.as_ref() {
            handler();
        }
        if let Some(handler) = action_dismiss_callback.as_ref() {
            handler();
        }
        on_dismiss.call(id);
    });
    let dismiss = EventHandler::new(move |_: ()| {
        if let Some(handler) = dismiss_handler.as_ref() {
            handler();
        }
        on_dismiss.call(id);
    });

    rsx! {
        Toast {
            message: toast.title,
            description: toast.description,
            variant: toast.variant,
            action_label: toast.action_label,
            icon: toast.icon,
            dismissible: toast.dismissible,
            rich_colors,
            swipe_direction,
            style,
            on_action: action,
            on_dismiss: dismiss,
        }
    }
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
) -> ToastPalette {
    let semantic = semantic_palette(variant, theme.mode);
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
    }

    #[test]
    fn loading_toast_is_persistent_by_default() {
        let toast = SonnerToast::loading(7, "Uploading");
        assert_eq!(toast.variant, ToastVariant::Loading);
        assert_eq!(toast.duration_ms, 0);
        assert!(toast.dismissible);
    }

    #[test]
    fn swipe_thresholds_use_logical_viewport_units() {
        assert!(should_dismiss_swipe(0.0, 56.0, ToastSwipeDirection::Down));
        assert!(should_dismiss_swipe(-72.0, 0.0, ToastSwipeDirection::Down));
        assert!(!should_dismiss_swipe(71.0, 55.0, ToastSwipeDirection::Down));
        assert!(should_dismiss_swipe(0.0, -56.0, ToastSwipeDirection::Up));
    }

    #[test]
    fn rich_error_palette_uses_semantic_surface() {
        let theme = Theme::default();
        let palette = toast_palette(ToastVariant::Error, theme, true, ToastStyle::default());
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
        );
        assert_eq!(palette.background, 0xFF123456);
        assert_eq!(palette.icon, 0xFFABCDEF);
    }
}
