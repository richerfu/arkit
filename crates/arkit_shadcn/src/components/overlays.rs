//! Sheet / Drawer / BottomSheet — official `sheet.tsx` side panel + bottom drawer.

use arkit_hooks::{OverlayLayer, Portal};
use arkit_prelude::*;

use super::chrome::{bottom_portal, provide_close, right_portal, DialogCloseButton};
use super::dialog::DialogHeader;
use crate::spec;
use crate::theme::use_theme;

fn sheet_panel(
    title: String,
    children: Element,
    horizontal: bool,
    theme: &crate::theme::Theme,
) -> Element {
    if horizontal {
        rsx! {
            column {
                width: spec::SHEET_W,
                height: "100%",
                background_color: theme.colors.background,
                border_width: 1.0,
                border_color: theme.colors.border,
                padding_top: spec::DIALOG_PAD,
                padding_right: spec::DIALOG_PAD,
                padding_bottom: spec::DIALOG_PAD,
                padding_left: spec::DIALOG_PAD,
                DialogHeader {
                    title,
                    description: None,
                }
                column {
                    width: "100%",
                    margin_top: 16.0,
                    {children}
                }
                DialogCloseButton {}
            }
        }
    } else {
        rsx! {
            column {
                width: "100%",
                background_color: theme.colors.background,
                border_width: 1.0,
                border_color: theme.colors.border,
                padding_top: spec::DIALOG_PAD,
                padding_right: spec::DIALOG_PAD,
                padding_bottom: spec::DIALOG_PAD,
                padding_left: spec::DIALOG_PAD,
                DialogHeader {
                    title,
                    description: None,
                }
                column {
                    width: "100%",
                    margin_top: 16.0,
                    {children}
                }
                DialogCloseButton {}
            }
        }
    }
}

fn left_portal(open: bool, panel: Element, on_dismiss: EventHandler<()>) -> Element {
    if !open {
        return rsx! {};
    }
    rsx! {
        Portal {
            layer: OverlayLayer::Modal,
            stack {
                width: "100%",
                height: "100%",
                background_color: spec::OVERLAY,
                alignment: "start",
                onclick: move |_| on_dismiss.call(()),
                stack {
                    height: "100%",
                    onclick: move |evt| evt.stop_propagation(),
                    {panel}
                }
            }
        }
    }
}

#[component]
pub fn Sheet(
    title: String,
    side: Option<String>,
    open: Option<bool>,
    default_open: Option<bool>,
    on_close: Option<EventHandler<()>>,
    children: Element,
) -> Element {
    let theme = use_theme();
    let mut internal = use_signal(|| default_open.unwrap_or(false));
    let current = match open {
        Some(v) => v,
        None => *internal.read(),
    };
    let controlled = open.is_some();
    let side_name = side.unwrap_or_else(|| "right".into());
    let close = EventHandler::new(move |_: ()| {
        if !controlled {
            internal.set(false);
        }
        if let Some(handler) = on_close {
            handler.call(());
        }
    });
    let horizontal = matches!(side_name.as_str(), "left" | "right");
    let panel = provide_close(close, sheet_panel(title, children, horizontal, &theme));
    match side_name.as_str() {
        "left" => left_portal(current, panel, close),
        "right" => right_portal(current, panel, close),
        _ => bottom_portal(current, panel, close),
    }
}

#[component]
pub fn Drawer(
    title: String,
    side: Option<String>,
    open: Option<bool>,
    default_open: Option<bool>,
    on_close: Option<EventHandler<()>>,
    children: Element,
) -> Element {
    let side = side.unwrap_or_else(|| "bottom".into());
    rsx! {
        Sheet {
            title,
            side: Some(side),
            open,
            default_open,
            on_close,
            {children}
        }
    }
}

#[component]
pub fn BottomSheet(
    title: String,
    open: Option<bool>,
    default_open: Option<bool>,
    show_header: Option<bool>,
    show_backdrop: Option<bool>,
    show_handle: Option<bool>,
    on_close: Option<EventHandler<()>>,
    children: Element,
) -> Element {
    let _ = (show_backdrop, show_handle);
    let theme = use_theme();
    let mut internal = use_signal(|| default_open.unwrap_or(false));
    let current = match open {
        Some(v) => v,
        None => *internal.read(),
    };
    let controlled = open.is_some();
    let show_header = show_header.unwrap_or(true);
    let close = EventHandler::new(move |_: ()| {
        if !controlled {
            internal.set(false);
        }
        if let Some(handler) = on_close {
            handler.call(());
        }
    });
    let panel = provide_close(
        close,
        rsx! {
            column {
                width: "100%",
                background_color: theme.colors.background,
                border_radius: format!("{0},{0},0,0", spec::RADIUS_XL),
                padding_top: spec::DIALOG_PAD,
                padding_right: spec::DIALOG_PAD,
                padding_bottom: spec::DIALOG_PAD,
                padding_left: spec::DIALOG_PAD,
                if show_header {
                    DialogHeader {
                        title,
                        description: None,
                    }
                }
                column {
                    width: "100%",
                    margin_top: 16.0,
                    {children}
                }
            }
        },
    );
    bottom_portal(current, panel, close)
}
