//! ToggleGroup — shadcn-style segmented toggle group.
//!
//! Migrated from the original Elm builder API to dioxus 0.7 `#[component]` +
//! `rsx!`. Preserves the original outline-variant shell (rounded, clipped,
//! small shadow) and items, single/multiple selection, and the text/icon item
//! variants. Reuses the shared toggle helpers from [`super::toggle`].

use crate::theme::*;
use arkit_prelude::*;

use super::toggle::{
    toggle_content_row, toggle_default_size, toggle_icon_size, toggle_surface, toggle_visual_style,
    ToggleSurfaceStyle, ToggleVariant,
};
use super::ARKUI_BORDER_STYLE_SOLID;

const TOGGLE_GROUP_VARIANT: ToggleVariant = ToggleVariant::Outline;

/// Props for [`ToggleGroup`].
#[derive(Props, Clone, PartialEq)]
pub struct ToggleGroupProps {
    pub options: Vec<String>,
    /// Controlled selection. When `Some`, the group is controlled.
    #[props(default)]
    pub selected: Option<Vec<String>>,
    #[props(default)]
    pub default_selected: Vec<String>,
    /// Render icon-only items (each option is a lucide icon name).
    #[props(default)]
    pub icons: bool,
    /// Allow multiple selections.
    #[props(default)]
    pub multi: bool,
    /// Optional group CSS width (`"100%"`). When provided, the items share the
    /// available width evenly, which is useful for mobile mode selectors.
    #[props(default)]
    pub width: Option<String>,
    /// Optional exact item height override for compact toolbars.
    #[props(default)]
    pub height: Option<f32>,
    /// Override the outline group's default small elevation.
    #[props(default)]
    pub shadow: Option<bool>,
    #[props(default)]
    pub on_change: EventHandler<Vec<String>>,
}

/// A segmented group of outline toggles. Supports single/multiple selection
/// and text/icon item variants.
#[component]
pub fn ToggleGroup(props: ToggleGroupProps) -> Element {
    let theme = use_theme();
    let controlled = props.selected.is_some();
    let local = use_signal(|| props.default_selected.clone());
    let selected: Vec<String> = props
        .selected
        .clone()
        .unwrap_or_else(|| local.read().clone());

    let total = props.options.len();
    let multi = props.multi;
    let icons = props.icons;
    let stretched = props.width.is_some();
    let group_shadow = props.shadow.unwrap_or(true);
    let on_change = props.on_change;
    let mut size_style = if icons {
        toggle_icon_size()
    } else {
        toggle_default_size()
    };
    if let Some(height) = props.height {
        size_style.height = height;
    }

    let mut items: Vec<Element> = Vec::with_capacity(total.saturating_mul(2).saturating_sub(1));
    for (index, option) in props.options.iter().enumerate() {
        let active = selected.contains(option);
        let visual = toggle_visual_style(TOGGLE_GROUP_VARIANT, active, &theme);
        let foreground = visual.foreground;
        let item_background = visual.background;
        let label_opt = if icons { None } else { Some(option.clone()) };
        let icon_opt = if icons { Some(option.clone()) } else { None };
        let content = toggle_content_row(label_opt, icon_opt, foreground, size_style.icon_size);

        let click_value = option.clone();
        let current_selected = selected.clone();
        let mut local = local;
        let surface = toggle_surface(
            content,
            ToggleSurfaceStyle {
                active,
                variant: TOGGLE_GROUP_VARIANT,
                size: size_style,
                // The group owns its outline. Per-item outlines create
                // doubled seams and fractional gaps on dense screens.
                border_width: 0.0,
                border_radius: "0".to_owned(),
                shadow: Some(false),
                width: if stretched { Some("100%".into()) } else { None },
                background: Some(0x00000000),
            },
            move || {
                let next = if multi {
                    let mut v = current_selected.clone();
                    if let Some(pos) = v.iter().position(|value| value == &click_value) {
                        v.remove(pos);
                    } else {
                        v.push(click_value.clone());
                    }
                    v
                } else {
                    vec![click_value.clone()]
                };
                if !controlled {
                    local.set(next.clone());
                }
                on_change.call(next);
            },
            &theme,
        );
        // Selection chrome is painted on the segment shell; the inner surface
        // stays clear for rectangular corners. Both layers use hit-testable
        // fills so presses land even when the segment is inactive/clear.
        // Inactive segments stay canvas-colored (opaque + hit-testable); active
        // segments use the accent/secondary fill from `toggle_visual_style`.
        let shell_background = if (item_background & 0xFF00_0000) == 0 {
            theme.colors.background
        } else {
            item_background
        };
        items.push(if stretched {
            rsx! {
                row {
                    layout_weight: 1.0,
                    height: size_style.height,
                    background_color: shell_background,
                    {surface}
                }
            }
        } else {
            rsx! {
                row {
                    height: size_style.height,
                    background_color: shell_background,
                    {surface}
                }
            }
        });
        if index + 1 < total {
            items.push(rsx! {
                row {
                    width: 1.0,
                    height: size_style.height,
                    background_color: theme.colors.input,
                }
            });
        }
    }

    rsx! {
        row {
            width: if let Some(width) = props.width { width },
            align_items: "center",
            justify_content: "start",
            border_style: ARKUI_BORDER_STYLE_SOLID,
            border_width: 1.0,
            border_color: theme.colors.input,
            border_radius: theme.radii.md,
            clip: true,
            shadow: if group_shadow { "sm" },
            {items.into_iter()}
        }
    }
}
