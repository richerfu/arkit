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
    /// Optional group width. When provided, the items share the available
    /// width evenly, which is useful for mobile mode selectors.
    #[props(default)]
    pub percent_width: Option<f32>,
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
    let stretched = props.percent_width.is_some();
    let group_shadow = props.shadow.unwrap_or(true);
    let on_change = props.on_change;
    let size_style = if icons {
        toggle_icon_size()
    } else {
        toggle_default_size()
    };

    let items: Vec<Element> = props
        .options
        .iter()
        .enumerate()
        .map(|(index, option)| {
            let active = selected.contains(option);
            let foreground = toggle_visual_style(TOGGLE_GROUP_VARIANT, active, &theme).foreground;
            let label_opt = if icons { None } else { Some(option.clone()) };
            let icon_opt = if icons { Some(option.clone()) } else { None };
            let content = toggle_content_row(label_opt, icon_opt, foreground, size_style.icon_size);

            let item_radius = {
                // [top, right, bottom, left] = [left_radius, right_radius, left_radius, right_radius]
                let left_radius = if index == 0 { theme.radii.md } else { 0.0 };
                let right_radius = if index + 1 == total {
                    theme.radii.md
                } else {
                    0.0
                };
                format!("{left_radius},{right_radius},{left_radius},{right_radius}")
            };

            let click_value = option.clone();
            let current_selected = selected.clone();
            let mut local = local;
            let surface = toggle_surface(
                content,
                ToggleSurfaceStyle {
                    active,
                    variant: TOGGLE_GROUP_VARIANT,
                    size: size_style,
                    border_width: 1.0,
                    border_radius: item_radius,
                    shadow: Some(false),
                    percent_width: if stretched { Some(1.0) } else { None },
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
            if stretched {
                rsx! {
                    row {
                        layout_weight: 1.0,
                        {surface}
                    }
                }
            } else {
                surface
            }
        })
        .collect();

    rsx! {
        row {
            percent_width: if let Some(width) = props.percent_width { width },
            align_items: "center",
            justify_content: "start",
            border_radius: theme.radii.md,
            clip: true,
            shadow: if group_shadow { 1 },
            {items.into_iter()}
        }
    }
}
