//! ColorUI dialog — `.cu-modal` + `.cu-dialog` + `.cu-bar`.

use arkit_prelude::*;

use super::chrome::{
    colorui_centered_portal, provide_close, CuBarFooter, CuBarHeader, CuDialogShell, PADDING,
    PADDING_XL,
};
use crate::theme::use_colorui_theme;

#[component]
pub fn Dialog(
    open: Option<bool>,
    default_open: Option<bool>,
    on_close: Option<EventHandler<()>>,
    children: Element,
) -> Element {
    let mut internal = use_signal(|| default_open.unwrap_or(false));
    let current = match open {
        Some(v) => v,
        None => *internal.read(),
    };
    let controlled = open.is_some();

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
            row {
                width: "100%",
                justify_content: "center",
                CuDialogShell {
                    {children}
                }
            }
        },
    );

    colorui_centered_portal(current, panel, close)
}

#[component]
pub fn DialogHeader(title: String, description: Option<String>) -> Element {
    let tokens = use_colorui_theme().tokens();
    rsx! {
        column {
            width: "100%",
            CuBarHeader {
                title,
                show_close: Some(true),
            }
            if let Some(description) = description.as_ref() {
                if !description.is_empty() {
                    text {
                        width: "100%",
                        content: description.clone(),
                        font_size: 14.0,
                        font_color: tokens.colors.muted_foreground,
                        line_height: 20.0,
                        padding_top: PADDING,
                        padding_right: PADDING_XL,
                        padding_bottom: 0.0,
                        padding_left: PADDING_XL,
                    }
                }
            }
        }
    }
}

#[component]
pub fn DialogFooter(children: Element) -> Element {
    rsx! {
        CuBarFooter {
            {children}
        }
    }
}
