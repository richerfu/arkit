//! Popover / Tooltip / HoverCard — ColorUI card / `cu-info` chrome.

use arkit_hooks::{OverlayLayer, Portal};
use arkit_prelude::*;

use super::chrome::PADDING;
use crate::spec;
use crate::theme::use_colorui_theme;

fn set_open_state(
    controlled: bool,
    mut internal: Signal<bool>,
    on_open_change: Option<EventHandler<bool>>,
    on_close: Option<EventHandler<()>>,
    next: bool,
) {
    if !controlled {
        internal.set(next);
    }
    if let Some(handler) = on_open_change {
        handler.call(next);
    }
    if !next {
        if let Some(handler) = on_close {
            handler.call(());
        }
    }
}

#[component]
pub fn Tooltip(
    trigger: Element,
    content: String,
    open: Option<bool>,
    default_open: Option<bool>,
    on_close: Option<EventHandler<()>>,
    on_open_change: Option<EventHandler<bool>>,
) -> Element {
    let internal = use_signal(|| default_open.unwrap_or(false));
    let current = open.unwrap_or_else(|| *internal.read());
    let controlled = open.is_some();

    rsx! {
        column {
            align_items: "center",
            onclick: move |_| {
                set_open_state(controlled, internal, on_open_change, on_close, !current);
            },
            {trigger}
            if current {
                row {
                    margin_top: 6.0,
                    padding_left: 6.0,
                    padding_right: 6.0,
                    padding_top: 4.0,
                    padding_bottom: 4.0,
                    background_color: spec::CHAT_INFO,
                    border_radius: spec::RADIUS,
                    text {
                        content,
                        font_size: spec::TEXT_SM,
                        font_color: spec::INK_ON_FILL,
                    }
                }
            }
        }
    }
}

#[component]
pub fn Popover(
    trigger: Element,
    open: Option<bool>,
    default_open: Option<bool>,
    on_close: Option<EventHandler<()>>,
    on_open_change: Option<EventHandler<bool>>,
    width: Option<f32>,
    padding: Option<f32>,
    children: Element,
) -> Element {
    let tokens = use_colorui_theme().tokens();
    let internal = use_signal(|| default_open.unwrap_or(false));
    let current = open.unwrap_or_else(|| *internal.read());
    let controlled = open.is_some();
    let panel_width = width.unwrap_or(280.0);
    let pad = padding.unwrap_or(PADDING);

    rsx! {
        column {
            {rsx! {
                row {
                    onclick: move |_| {
                        set_open_state(controlled, internal, on_open_change, on_close, !current);
                    },
                    {trigger}
                }
            }}
            if current {
                Portal {
                    layer: OverlayLayer::Floating,
                    stack {
                        width: "100%",
                        height: "100%",
                        background_color: spec::OVERLAY,
                        alignment: "center",
                        onclick: move |_| {
                            set_open_state(controlled, internal, on_open_change, on_close, false);
                        },
                        column {
                            width: panel_width,
                            background_color: tokens.colors.card,
                            border_radius: spec::RADIUS_CARD,
                            padding_top: pad,
                            padding_right: pad,
                            padding_bottom: pad,
                            padding_left: pad,
                            onclick: move |evt| evt.stop_propagation(),
                            {children}
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn HoverCard(
    trigger: Element,
    open: Option<bool>,
    default_open: Option<bool>,
    on_close: Option<EventHandler<()>>,
    on_open_change: Option<EventHandler<bool>>,
    width: Option<f32>,
    children: Element,
) -> Element {
    rsx! {
        Popover {
            trigger,
            open,
            default_open,
            on_close,
            on_open_change,
            width,
            padding: Some(PADDING),
            {children}
        }
    }
}
