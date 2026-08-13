//! RadioGroup — shadcn-style single-choice radio group.
//!
//! Migrated from the original Elm builder API to dioxus 0.7 `#[component]` +
//! `rsx!`. Matches the React Native Reusables primitive: a 16x16 primary
//! border ring with an 8x8 primary dot when checked, full radius, 8px row gap,
//! and medium-weight `SM` labels.

use super::ARKUI_BORDER_STYLE_SOLID;
use crate::style::*;
use arkit_prelude::*;

const RADIO_SIZE: f32 = 16.0;
const RADIO_DOT_SIZE: f32 = 8.0;
const RADIO_BORDER_WIDTH: f32 = 1.0;

/// Render the radio indicator ring (with a primary dot when `checked`).
fn radio_indicator(checked: bool, theme: &Theme) -> Element {
    rsx! {
        stack {
            width: RADIO_SIZE,
            height: RADIO_SIZE,
            alignment: "center",
            border_radius: theme.radii.full,
            border_width: RADIO_BORDER_WIDTH,
            border_style: ARKUI_BORDER_STYLE_SOLID,
            border_color: theme.colors.primary,
            background_color: theme.colors.background,
            clip: true,
            if checked {
                row {
                    width: RADIO_DOT_SIZE,
                    height: RADIO_DOT_SIZE,
                    border_radius: theme.radii.full,
                    background_color: theme.colors.primary,
                }
            }
        }
    }
}

/// Props for [`RadioGroup`].
#[derive(Props, Clone, PartialEq)]
pub struct RadioGroupProps {
    pub options: Vec<String>,
    /// Controlled selection. When `Some`, the group is controlled.
    #[props(default)]
    pub selected: Option<String>,
    #[props(default)]
    pub default_selected: String,
    #[props(default)]
    pub on_select: EventHandler<String>,
}

/// A vertical group of radio options. Selecting an option fires
/// [`RadioGroupProps::on_select`] with its value.
#[component]
pub fn RadioGroup(props: RadioGroupProps) -> Element {
    let theme = use_theme();
    let controlled = props.selected.is_some();
    let local = use_signal(|| props.default_selected.clone());
    let selected: String = props
        .selected
        .clone()
        .unwrap_or_else(|| local.read().clone());

    let rows: Vec<Element> = props
        .options
        .iter()
        .enumerate()
        .map(|(index, option)| {
            let checked = selected == *option;
            let indicator = radio_indicator(checked, &theme);
            let on_select = props.on_select;
            let mut local = local;
            let click_value = option.clone();
            let label_color = theme.colors.foreground;
            let top_margin = if index == 0 { 0.0 } else { spacing::MD };
            let row = rsx! {
                row {
                    width: "100%",
                    align_items: "center",
                    justify_content: "start",
                    margin_top: top_margin,
                    onclick: move |_| {
                        if !controlled {
                            local.set(click_value.clone());
                        }
                        on_select.call(click_value.clone());
                    },
                    {indicator}
                    row {
                        margin_left: spacing::MD,
                        text {
                            content: option.clone(),
                            font_size: typography::SM,
                            font_weight: 500,
                            font_color: label_color,
                            text_align: "start",
                        }
                    }
                }
            };
            row
        })
        .collect();

    rsx! {
        column {
            width: "100%",
            {rows.into_iter()}
        }
    }
}
