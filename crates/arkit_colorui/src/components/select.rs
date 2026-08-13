//! ColorUI select — form-group trigger + bottom-modal option list.

use arkit_prelude::*;

use super::chrome::{colorui_bottom_portal, dialog_fill, provide_close, CuBarHeader, PADDING};
use crate::theme::use_colorui_theme;

#[component]
pub fn Select(
    options: Vec<String>,
    placeholder: Option<String>,
    label: Option<String>,
    selected: Option<String>,
    default_selected: String,
    open: Option<bool>,
    default_open: bool,
    on_open_change: Option<EventHandler<bool>>,
    on_select: Option<EventHandler<String>>,
) -> Element {
    let theme = use_colorui_theme();
    let tokens = theme.tokens();
    let dark = matches!(theme.mode, arkit_component::style::ThemeMode::Dark);
    let mut internal_open = use_signal(|| default_open);
    let mut internal_selected = use_signal(|| default_selected.clone());
    let open_controlled = open.is_some();
    let selected_controlled = selected.is_some();
    let current_open = open.unwrap_or_else(|| *internal_open.read());
    let current_selected = selected
        .clone()
        .unwrap_or_else(|| (*internal_selected.read()).clone());

    let set_open = EventHandler::new(move |value: bool| {
        if !open_controlled {
            internal_open.set(value);
        }
        if let Some(handler) = on_open_change {
            handler.call(value);
        }
    });
    let set_selected = EventHandler::new(move |value: String| {
        if !selected_controlled {
            internal_selected.set(value.clone());
        }
        if let Some(handler) = on_select {
            handler.call(value);
        }
    });

    let has_value = !current_selected.is_empty();
    let trigger_label = if has_value {
        current_selected.clone()
    } else {
        placeholder.unwrap_or_else(|| "请选择".into())
    };
    let label_color = if has_value {
        0xFF555555u32
    } else {
        0xFF888888u32
    };
    let title = label
        .clone()
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "请选择".into());
    let dismiss = EventHandler::new(move |_: ()| set_open.call(false));

    let option_rows: Vec<Element> = options
        .iter()
        .map(|option| {
            let active = current_selected == *option;
            let opt = option.clone();
            let set_selected = set_selected;
            let dismiss = dismiss;
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
                        set_selected.call(opt.clone());
                        dismiss.call(());
                    },
                    row {
                        layout_weight: 1.0,
                        text {
                            content: option.clone(),
                            font_size: 15.0,
                            font_color: if active {
                                tokens.colors.primary
                            } else {
                                tokens.colors.foreground
                            },
                        }
                    }
                    if active {
                        {arkit_icon::icon("check", 16.0, tokens.colors.primary)}
                    }
                }
            }
        })
        .collect();

    let panel = provide_close(
        dismiss,
        rsx! {
            column {
                width: "100%",
                background_color: dialog_fill(dark),
                CuBarHeader {
                    title: title.clone(),
                    show_close: Some(true),
                }
                column {
                    width: "100%",
                    {option_rows.into_iter()}
                }
            }
        },
    );

    rsx! {
        row {
            width: "100%",
            min_height: 50.0,
            align_items: "center",
            justify_content: "space-between",
            background_color: tokens.colors.card,
            padding_left: PADDING,
            padding_right: PADDING,
            onclick: move |_| set_open.call(!current_open),
            if let Some(label) = label.filter(|value| !value.is_empty()) {
                text {
                    content: label,
                    font_size: 15.0,
                    font_color: tokens.colors.foreground,
                    margin_right: 12.0,
                }
            }
            row {
                layout_weight: 1.0,
                align_items: "center",
                justify_content: "end",
                text {
                    content: trigger_label,
                    font_size: 14.0,
                    font_color: label_color,
                    text_align: "end",
                }
                {arkit_icon::icon("chevron-right", 16.0, 0xFF8799A3)}
            }
        }
        {colorui_bottom_portal(current_open, panel, dismiss)}
    }
}
