//! Popover / Tooltip / HoverCard — official `w-72` / `px-3 py-1.5` / `w-64`.

use arkit_hooks::{OverlayLayer, Portal};
use arkit_prelude::*;

use crate::spec;
use crate::theme::use_theme;

fn flip(
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
    let theme = use_theme();
    let internal = use_signal(|| default_open.unwrap_or(false));
    let current = open.unwrap_or_else(|| *internal.read());
    let controlled = open.is_some();
    rsx! {
        column {
            align_items: "center",
            onclick: move |_| flip(controlled, internal, on_open_change, on_close, !current),
            {trigger}
            if current {
                row {
                    margin_top: 6.0,
                    padding_left: 12.0,
                    padding_right: 12.0,
                    padding_top: 6.0,
                    padding_bottom: 6.0,
                    background_color: theme.colors.popover,
                    border_width: 1.0,
                    border_color: theme.colors.border,
                    border_radius: spec::RADIUS_MD,
                    shadow: "sm",
                    text {
                        content,
                        font_size: spec::TEXT_SM,
                        font_color: theme.colors.popover_foreground,
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
    let theme = use_theme();
    let internal = use_signal(|| default_open.unwrap_or(false));
    let current = open.unwrap_or_else(|| *internal.read());
    let controlled = open.is_some();
    let panel_w = width.unwrap_or(spec::POPOVER_W);
    let pad = padding.unwrap_or(16.0);
    rsx! {
        column {
            row {
                onclick: move |_| flip(controlled, internal, on_open_change, on_close, !current),
                {trigger}
            }
            if current {
                Portal {
                    layer: OverlayLayer::Floating,
                    stack {
                        width: "100%",
                        height: "100%",
                        background_color: spec::OVERLAY,
                        alignment: "center",
                        onclick: move |_| flip(controlled, internal, on_open_change, on_close, false),
                        column {
                            width: panel_w,
                            background_color: theme.colors.popover,
                            border_width: 1.0,
                            border_color: theme.colors.border,
                            border_radius: spec::RADIUS_MD,
                            shadow: "sm",
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
            width: Some(width.unwrap_or(spec::HOVER_CARD_W)),
            padding: Some(16.0),
            {children}
        }
    }
}
