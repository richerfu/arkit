//! RadioGroup — official 16px ring + 8px primary dot.

use arkit_prelude::*;

use crate::theme::use_theme;

#[component]
pub fn RadioGroup(
    options: Vec<String>,
    #[props(default)] selected: Option<String>,
    #[props(default)] default_selected: String,
    #[props(default)] on_select: EventHandler<String>,
) -> Element {
    let theme = use_theme();
    let controlled = selected.is_some();
    let local = use_signal(|| default_selected.clone());
    let current = selected.unwrap_or_else(|| local.read().clone());
    let rows: Vec<Element> = options
        .iter()
        .enumerate()
        .map(|(index, option)| {
            let checked = current == *option;
            let click = option.clone();
            let mut local = local;
            rsx! {
                row {
                    width: "100%",
                    align_items: "center",
                    margin_top: if index == 0 { 0.0 } else { 12.0 },
                    onclick: move |_| {
                        if !controlled {
                            local.set(click.clone());
                        }
                        on_select.call(click.clone());
                    },
                    stack {
                        width: 16.0,
                        height: 16.0,
                        alignment: "center",
                        border_radius: 999.0,
                        border_width: 1.0,
                        border_color: theme.colors.primary,
                        background_color: theme.colors.background,
                        if checked {
                            row {
                                width: 8.0,
                                height: 8.0,
                                border_radius: 999.0,
                                background_color: theme.colors.primary,
                            }
                        }
                    }
                    text {
                        content: option.clone(),
                        margin_left: 8.0,
                        font_size: 14.0,
                        font_weight: 500,
                        font_color: theme.colors.foreground,
                    }
                }
            }
        })
        .collect();
    rsx! {
        column { width: "100%", {rows.into_iter()} }
    }
}
