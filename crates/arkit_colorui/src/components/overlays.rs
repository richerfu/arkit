//! ColorUI sheet / drawer / bottom-modal — `.cu-modal.drawer-modal` / `.bottom-modal`.

use arkit_hooks::{OverlayLayer, Portal};
use arkit_prelude::*;

use super::chrome::{
    colorui_bottom_portal, colorui_right_portal, dialog_fill, provide_close, CuBarHeader,
    DRAWER_WIDTH, OVERLAY, PADDING,
};
use crate::theme::use_colorui_theme;

fn side_is_horizontal(side: &str) -> bool {
    matches!(side, "left" | "right")
}

fn sheet_panel(title: String, children: Element, dark: bool, horizontal: bool) -> Element {
    if horizontal {
        rsx! {
            column {
                width: DRAWER_WIDTH,
                height: "100%",
                background_color: dialog_fill(dark),
                CuBarHeader {
                    title,
                    show_close: Some(true),
                }
                column {
                    width: "100%",
                    padding_top: PADDING,
                    padding_right: PADDING,
                    padding_bottom: PADDING,
                    padding_left: PADDING,
                    {children}
                }
            }
        }
    } else {
        rsx! {
            column {
                width: "100%",
                background_color: dialog_fill(dark),
                CuBarHeader {
                    title,
                    show_close: Some(true),
                }
                column {
                    width: "100%",
                    padding_top: PADDING,
                    padding_right: PADDING,
                    padding_bottom: PADDING,
                    padding_left: PADDING,
                    {children}
                }
            }
        }
    }
}

fn colorui_left_portal(open: bool, panel: Element, on_dismiss: EventHandler<()>) -> Element {
    if !open {
        return rsx! {};
    }
    rsx! {
        Portal {
            layer: OverlayLayer::Modal,
            stack {
                width: "100%",
                height: "100%",
                background_color: OVERLAY,
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
    let theme = use_colorui_theme();
    let dark = matches!(theme.mode, arkit_component::style::ThemeMode::Dark);
    let mut internal = use_signal(|| default_open.unwrap_or(false));
    let current = match open {
        Some(v) => v,
        None => *internal.read(),
    };
    let controlled = open.is_some();
    let side_name = side.unwrap_or_else(|| "right".into());
    let horizontal = side_is_horizontal(&side_name);

    let close = EventHandler::new(move |_: ()| {
        if !controlled {
            internal.set(false);
        }
        if let Some(handler) = on_close {
            handler.call(());
        }
    });

    let panel = provide_close(close, sheet_panel(title, children, dark, horizontal));

    if side_name == "left" {
        colorui_left_portal(current, panel, close)
    } else if side_name == "right" {
        colorui_right_portal(current, panel, close)
    } else {
        colorui_bottom_portal(current, panel, close)
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
    let _ = (show_handle, show_backdrop);
    let theme = use_colorui_theme();
    let dark = matches!(theme.mode, arkit_component::style::ThemeMode::Dark);
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
                background_color: dialog_fill(dark),
                if show_header {
                    CuBarHeader {
                        title,
                        show_close: Some(true),
                    }
                }
                column {
                    width: "100%",
                    padding_top: PADDING,
                    padding_right: PADDING,
                    padding_bottom: PADDING,
                    padding_left: PADDING,
                    {children}
                }
            }
        },
    );

    colorui_bottom_portal(current, panel, close)
}
