//! ColorUI radio group — 24px filled circle, matching `radio` in main.css.

use arkit_prelude::*;

use super::chrome::PADDING;
use crate::theme::use_colorui_theme;

const RADIO_SIZE: f32 = 24.0;
const RADIO_DOT: f32 = 8.0;

#[component]
pub fn RadioGroup(
    options: Vec<String>,
    #[props(default)] selected: Option<String>,
    #[props(default)] default_selected: String,
    #[props(default)] on_select: EventHandler<String>,
) -> Element {
    let tokens = use_colorui_theme().tokens();
    let controlled = selected.is_some();
    let local = use_signal(|| default_selected.clone());
    let current: String = selected.clone().unwrap_or_else(|| local.read().clone());

    let rows: Vec<Element> = options
        .iter()
        .enumerate()
        .map(|(_index, option)| {
            let checked = current == *option;
            let on_select = on_select;
            let mut local = local;
            let click_value = option.clone();
            let label = option.clone();
            rsx! {
                row {
                    width: "100%",
                    min_height: 50.0,
                    align_items: "center",
                    justify_content: "space-between",
                    padding_left: PADDING,
                    padding_right: PADDING,
                    background_color: tokens.colors.card,
                    onclick: move |_| {
                        if !controlled {
                            local.set(click_value.clone());
                        }
                        on_select.call(click_value.clone());
                    },
                    row {
                        layout_weight: 1.0,
                        text {
                            content: label,
                            font_size: 15.0,
                            font_color: tokens.colors.foreground,
                        }
                    }
                    stack {
                        width: RADIO_SIZE,
                        height: RADIO_SIZE,
                        alignment: "center",
                        border_radius: 999.0,
                        border_width: if checked { 0.0 } else { 1.0 },
                        border_color: if checked {
                            tokens.colors.primary
                        } else {
                            0xFFCCCCCCu32
                        },
                        background_color: if checked {
                            tokens.colors.primary
                        } else {
                            0xFFFFFFFFu32
                        },
                        if checked {
                            row {
                                width: RADIO_DOT,
                                height: RADIO_DOT,
                                border_radius: 999.0,
                                background_color: 0xFFFFFFFFu32,
                            }
                        }
                    }
                }
            }
        })
        .collect();

    rsx! {
        column {
            width: "100%",
            {rows.into_iter()}
        }
    }
}
