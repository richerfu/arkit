//! Toggle — shadcn-style two-state button.
//!
//! Migrated from the original Elm builder API to dioxus 0.7 `#[component]` +
//! `rsx!`. Preserves the original variants (`Default`, `Outline`), the default
//! and icon size styles, the active/inactive visual style mapping, and the
//! content row (icon + optional label) layout. The size/visual helpers are
//! `pub(crate)` so [`super::toggle_group`] can reuse them.

use crate::theme::*;
use arkit_prelude::*;

use super::ARKUI_BORDER_STYLE_SOLID;

pub(crate) const TOGGLE_TRANSPARENT: u32 = 0x00000000;
/// Visually transparent, but alpha=1 so ArkUI still hit-tests the surface.
/// Fully transparent (`0x00000000`) paints skip hit testing on many devices.
pub(crate) const TOGGLE_HIT_FILL: u32 = 0x01000000;

fn paint_or_hit_fill(color: u32) -> u32 {
    if color & 0xFF00_0000 == 0 {
        TOGGLE_HIT_FILL
    } else {
        color
    }
}

/// Toggle visual variant. `Default` is borderless; `Outline` adds an input
/// border and a small shadow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToggleVariant {
    #[default]
    Default,
    Outline,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ToggleSizeStyle {
    pub(crate) height: f32,
    pub(crate) width: Option<f32>,
    pub(crate) padding: [f32; 4],
    pub(crate) icon_size: f32,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ToggleVisualStyle {
    pub(crate) background: u32,
    pub(crate) foreground: u32,
    pub(crate) border_color: u32,
    shadow: bool,
}

pub(crate) struct ToggleSurfaceStyle {
    pub(crate) active: bool,
    pub(crate) variant: ToggleVariant,
    pub(crate) size: ToggleSizeStyle,
    pub(crate) border_width: f32,
    pub(crate) border_radius: String,
    pub(crate) shadow: Option<bool>,
    pub(crate) width: Option<String>,
    /// Overrides the surface fill. Toggle groups pass a clear fill and paint
    /// selection on the rectangular segment shell so corners stay square.
    pub(crate) background: Option<u32>,
}

pub(crate) fn toggle_default_size() -> ToggleSizeStyle {
    ToggleSizeStyle {
        height: 40.0,
        width: None,
        padding: [8.0, 10.0, 8.0, 10.0],
        icon_size: 16.0,
    }
}

pub(crate) fn toggle_icon_size() -> ToggleSizeStyle {
    ToggleSizeStyle {
        height: 40.0,
        width: Some(40.0),
        padding: [0.0, 0.0, 0.0, 0.0],
        icon_size: 16.0,
    }
}

pub(crate) fn toggle_visual_style(
    variant: ToggleVariant,
    active: bool,
    theme: &Theme,
) -> ToggleVisualStyle {
    // Active fill uses `secondary` (same token as shadcn accent). On light Zinc
    // that is `#F4F4F5` — visible on white, still subtle. Inactive stays clear
    // (hit-testable via `paint_or_hit_fill`).
    match variant {
        ToggleVariant::Default => ToggleVisualStyle {
            background: if active {
                theme.colors.secondary
            } else {
                TOGGLE_TRANSPARENT
            },
            foreground: if active {
                theme.colors.secondary_foreground
            } else {
                theme.colors.foreground
            },
            border_color: TOGGLE_TRANSPARENT,
            shadow: false,
        },
        ToggleVariant::Outline => ToggleVisualStyle {
            background: if active {
                theme.colors.secondary
            } else {
                theme.colors.background
            },
            foreground: if active {
                theme.colors.secondary_foreground
            } else {
                theme.colors.foreground
            },
            border_color: theme.colors.input,
            shadow: true,
        },
    }
}

/// Build the inner content row: an optional leading icon followed by an
/// optional label (the label is inset `8.0` when an icon precedes it).
///
/// Content is decorative only — the surface uses `HitTestMode::Block` so
/// Image/Text children cannot steal the press from the toggle's `onclick`.
pub(crate) fn toggle_content_row(
    label: Option<String>,
    icon: Option<String>,
    foreground: u32,
    icon_size: f32,
) -> Element {
    let mut children: Vec<Element> = Vec::new();
    if let Some(name) = icon {
        children.push(arkit_icon::icon(name, icon_size, foreground));
    }
    if let Some(text) = label {
        let text_el = rsx! {
            text {
                content: text,
                font_size: typography::SM,
                font_color: foreground,
                font_weight: 500,
                line_height: 20.0,
            }
        };
        if children.is_empty() {
            children.push(text_el);
        } else {
            children.push(rsx! { row { margin_left: 8.0, {text_el} } });
        }
    }
    rsx! {
        row {
            align_items: "center",
            justify_content: "center",
            {children.into_iter()}
        }
    }
}

/// Render the toggle surface with the given visual + size configuration.
///
/// Uses a clickable `row` (not ArkUI `Button`) so:
/// - inactive / group segments can stay visually transparent without losing hits
/// - group segments keep rectangular corners (Button skin forces capsules)
///
/// `on_click` fires on activation. `border_radius` is a comma-separated
/// `[top, right, bottom, left]` string so callers can express per-corner radii
/// (used by [`super::toggle_group`]).
pub(crate) fn toggle_surface(
    content: Element,
    style: ToggleSurfaceStyle,
    on_click: impl FnMut() + 'static,
    theme: &Theme,
) -> Element {
    let _ = theme;
    let visual = toggle_visual_style(style.variant, style.active, theme);
    let height = style.size.height;
    let pt = style.size.padding[0];
    let pr = style.size.padding[1];
    let pb = style.size.padding[2];
    let pl = style.size.padding[3];
    let border_color = visual.border_color;
    let background = paint_or_hit_fill(style.background.unwrap_or(visual.background));
    let shadow_on = style.shadow.unwrap_or(visual.shadow);
    let mut on_click = on_click;
    // Prefer CSS width when provided (stretched group segments); otherwise
    // explicit size width (icon-only). Avoid always emitting `width: Option`.
    let fixed_width = style.size.width;
    let css_width = style.width;
    rsx! {
        row {
            border_radius: style.border_radius,
            clip: true,
            border_style: ARKUI_BORDER_STYLE_SOLID,
            border_width: style.border_width,
            border_color: border_color,
            align_self: "start",
            align_items: "center",
            justify_content: "center",
            padding_top: pt,
            padding_right: pr,
            padding_bottom: pb,
            padding_left: pl,
            background_color: background,
            height: height,
            width: if let Some(w) = css_width {
                w
            } else if let Some(w) = fixed_width {
                format!("{w}")
            },
            shadow: if shadow_on { "sm" },
            // HitTestMode::Block — this node takes the hit; children (Image/Text)
            // are excluded so icon content cannot absorb the press without
            // bubbling `onclick` to the surface.
            hit_test_behavior: "block",
            onclick: move |_| on_click(),
            {content}
        }
    }
}

/// Props for [`Toggle`].
#[derive(Props, Clone, PartialEq)]
pub struct ToggleProps {
    /// Text label shown when `icon` is absent.
    pub label: String,
    /// When set, renders an icon-only toggle using this lucide icon name.
    #[props(default)]
    pub icon: Option<String>,
    #[props(default)]
    pub variant: ToggleVariant,
    /// Controlled value. When `Some`, the toggle is controlled.
    #[props(default)]
    pub checked: Option<bool>,
    #[props(default)]
    pub default_checked: bool,
    #[props(default)]
    pub on_change: EventHandler<bool>,
}

/// A two-state button. Supports text and icon variants, `Default`/`Outline`
/// visuals, and controlled/unchecked usage.
#[component]
pub fn Toggle(props: ToggleProps) -> Element {
    let theme = use_theme();
    let mut local = use_signal(|| props.default_checked);
    let active = props.checked.unwrap_or_else(|| *local.read());

    let is_icon = props.icon.is_some();
    let size_style = if is_icon {
        toggle_icon_size()
    } else {
        toggle_default_size()
    };
    let variant = props.variant;
    let foreground = toggle_visual_style(variant, active, &theme).foreground;
    let label_opt = if is_icon {
        None
    } else {
        Some(props.label.clone())
    };
    let content = toggle_content_row(
        label_opt,
        props.icon.clone(),
        foreground,
        size_style.icon_size,
    );

    let on_change = props.on_change;
    let r = theme.radii.md;
    let radius = format!("{r},{r},{r},{r}");
    // Read pressed state inside the handler so each click uses the latest
    // controlled/uncontrolled value (not a stale copy closed over at render).
    let checked_prop = props.checked;
    let border_width = match variant {
        ToggleVariant::Outline => 1.0,
        ToggleVariant::Default => 0.0,
    };
    rsx! {
        {toggle_surface(
            content,
            ToggleSurfaceStyle {
                active,
                variant,
                size: size_style,
                border_width,
                border_radius: radius,
                shadow: None,
                width: None,
                background: None,
            },
            move || {
                let current = checked_prop.unwrap_or_else(|| *local.read());
                let next = !current;
                if checked_prop.is_none() {
                    local.set(next);
                }
                on_change.call(next);
            },
            &theme,
        )}
    }
}
