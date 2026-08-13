//! shadcn Dialog — `dialog.tsx` `DialogContent` / `DialogHeader` / `DialogFooter`.

use arkit_prelude::*;

use super::chrome::{centered_portal, provide_close, DialogCloseButton, DialogShell};
use crate::spec;
use crate::theme::use_theme;

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
                stack {
                    width: "100%",
                    max_width: spec::DIALOG_MAX_W,
                    DialogShell {
                        {children}
                    }
                    row {
                        width: "100%",
                        padding_top: spec::DIALOG_CLOSE,
                        padding_right: spec::DIALOG_CLOSE,
                        justify_content: "end",
                        hit_test_behavior: "transparent",
                        DialogCloseButton {}
                    }
                }
            }
        },
    );
    centered_portal(current, panel, close)
}

#[component]
pub fn DialogHeader(title: String, description: Option<String>) -> Element {
    let theme = use_theme();
    rsx! {
        column {
            width: "100%",
            align_items: "start",
            text {
                width: "100%",
                content: title,
                font_size: spec::TEXT_LG,
                font_weight: spec::FONT_SEMIBOLD,
                font_color: theme.colors.foreground,
                line_height: 20.0,
                text_align: "start",
            }
            if let Some(description) = description {
                if !description.is_empty() {
                    text {
                        width: "100%",
                        content: description,
                        margin_top: 8.0,
                        font_size: spec::TEXT_SM,
                        font_color: theme.colors.muted_foreground,
                        line_height: 20.0,
                    }
                }
            }
        }
    }
}

#[component]
pub fn DialogFooter(children: Element) -> Element {
    rsx! {
        row {
            width: "100%",
            margin_top: 16.0,
            align_items: "center",
            justify_content: "end",
            {children}
        }
    }
}
