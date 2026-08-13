//! ColorUI alert dialog — `.cu-dialog` with a bar header and action footer.

use arkit_component::components::{use_dialog_close, Button as HeadlessButton, ButtonVariant};
use arkit_prelude::*;

use super::chrome::{
    colorui_centered_portal, provide_close, CuBarFooter, CuBarHeader, CuDialogShell, PADDING_XL,
};
use crate::theme::use_colorui_theme;

#[component]
pub fn AlertDialog(
    title: String,
    description: String,
    open: Option<bool>,
    default_open: Option<bool>,
    on_close: Option<EventHandler<()>>,
    cancel: Option<Element>,
    action: Option<Element>,
    children: Element,
) -> Element {
    let tokens = use_colorui_theme().tokens();
    let mut internal = use_signal(|| default_open.unwrap_or(false));
    let current = match open {
        Some(v) => v,
        None => *internal.read(),
    };
    let controlled = open.is_some();
    let has_cancel = cancel.is_some();
    let has_action = action.is_some();

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
                    CuBarHeader {
                        title,
                        show_close: Some(true),
                    }
                    column {
                        width: "100%",
                        padding_top: PADDING_XL,
                        padding_right: PADDING_XL,
                        padding_bottom: PADDING_XL,
                        padding_left: PADDING_XL,
                        text {
                            width: "100%",
                            content: description,
                            font_size: 14.0,
                            font_color: tokens.colors.muted_foreground,
                            line_height: 20.0,
                            text_align: "center",
                        }
                    }
                    if has_action || has_cancel {
                        CuBarFooter {
                            if let Some(cancel) = cancel {
                                row {
                                    margin_right: 8.0,
                                    {cancel}
                                }
                            }
                            if let Some(action) = action {
                                {action}
                            }
                        }
                    } else {
                        {children}
                    }
                }
            }
        },
    );

    colorui_centered_portal(current, panel, close)
}

/// Cancel / action button that always invokes [`use_dialog_close`].
#[component]
pub fn AlertDialogAction(
    #[props(default)] variant: Option<ButtonVariant>,
    #[props(default)] width: Option<String>,
    #[props(default)] onclick: Option<EventHandler<()>>,
    children: Element,
) -> Element {
    let close = use_dialog_close();
    let variant = variant.unwrap_or(ButtonVariant::Default);
    rsx! {
        HeadlessButton {
            variant,
            width,
            onclick: move |_| {
                if let Some(handler) = onclick {
                    handler.call(());
                }
                if let Some(close) = close.as_ref() {
                    close.call();
                }
            },
            {children}
        }
    }
}
