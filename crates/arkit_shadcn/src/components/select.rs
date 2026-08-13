//! Select — official outline trigger + popover list (`w-full`, `h-9` mapped).

use arkit_prelude::*;

use super::chrome::{bottom_portal, provide_close, DialogCloseButton};
use crate::spec;
use crate::theme::use_theme;

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
    let theme = use_theme();
    let mut internal_open = use_signal(|| default_open);
    let mut internal_selected = use_signal(|| default_selected.clone());
    let open_controlled = open.is_some();
    let selected_controlled = selected.is_some();
    let current_open = open.unwrap_or_else(|| *internal_open.read());
    let current = selected.unwrap_or_else(|| (*internal_selected.read()).clone());
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
    let has = !current.is_empty();
    let trigger = if has {
        current.clone()
    } else {
        placeholder.unwrap_or_else(|| "Select".into())
    };
    let dismiss = EventHandler::new(move |_: ()| set_open.call(false));
    let rows: Vec<Element> = options
        .iter()
        .map(|option| {
            let active = current == *option;
            let opt = option.clone();
            rsx! {
                row {
                    width: "100%",
                    height: 36.0,
                    padding_left: 12.0,
                    padding_right: 12.0,
                    align_items: "center",
                    justify_content: "space-between",
                    border_radius: spec::RADIUS_MD,
                    background_color: if active {
                        theme.colors.accent
                    } else {
                        0x00000000u32
                    },
                    onclick: move |_| {
                        set_selected.call(opt.clone());
                        dismiss.call(());
                    },
                    text {
                        content: option.clone(),
                        font_size: spec::TEXT_SM,
                        font_color: theme.colors.foreground,
                    }
                    if active {
                        {arkit_icon::icon("check", 16.0, theme.colors.foreground)}
                    }
                }
            }
        })
        .collect();
    let title = label
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| "Select".into());
    let panel = provide_close(
        dismiss,
        rsx! {
            column {
                width: "100%",
                background_color: theme.colors.popover,
                border_radius: format!("{0},{0},0,0", spec::RADIUS_LG),
                padding_top: 12.0,
                padding_right: 12.0,
                padding_bottom: 12.0,
                padding_left: 12.0,
                row {
                    width: "100%",
                    justify_content: "space-between",
                    align_items: "center",
                    text {
                        content: title,
                        font_size: spec::TEXT_SM,
                        font_color: theme.colors.muted_foreground,
                    }
                    DialogCloseButton {}
                }
                column { width: "100%", margin_top: 8.0, {rows.into_iter()} }
            }
        },
    );
    rsx! {
        row {
            width: "100%",
            height: spec::BTN_HEIGHT,
            padding_left: 12.0,
            padding_right: 12.0,
            align_items: "center",
            justify_content: "space-between",
            background_color: theme.colors.background,
            border_width: 1.0,
            border_color: theme.colors.border,
            border_radius: spec::RADIUS_MD,
            shadow: "sm",
            onclick: move |_| set_open.call(!current_open),
            text {
                content: trigger,
                font_size: spec::TEXT_SM,
                font_color: if has {
                    theme.colors.foreground
                } else {
                    theme.colors.muted_foreground
                },
            }
            {arkit_icon::icon("chevron-down", 16.0, theme.colors.muted_foreground)}
        }
        {bottom_portal(current_open, panel, dismiss)}
    }
}
