//! shadcn AlertDialog — stacked mobile actions (`flex-col-reverse` on small).

use arkit_component::components::{use_dialog_close, Button as HeadlessButton, ButtonVariant};
use arkit_prelude::*;

use super::chrome::{centered_portal, provide_close, DialogShell};
use crate::spec;
use crate::theme::use_theme;

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
    let theme = use_theme();
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
                DialogShell {
                    column {
                        width: "100%",
                        text {
                            width: "100%",
                            content: title,
                            font_size: spec::TEXT_LG,
                            font_weight: spec::FONT_SEMIBOLD,
                            font_color: theme.colors.foreground,
                            line_height: 24.0,
                        }
                        text {
                            width: "100%",
                            content: description,
                            margin_top: 8.0,
                            font_size: spec::TEXT_SM,
                            font_color: theme.colors.muted_foreground,
                            line_height: 20.0,
                        }
                    }
                    column {
                        width: "100%",
                        margin_top: 16.0,
                        if let Some(action) = action {
                            {action}
                        }
                        if has_action && has_cancel {
                            row { width: "100%", height: 8.0 }
                        }
                        if let Some(cancel) = cancel {
                            {cancel}
                        }
                        if !has_action && !has_cancel {
                            {children}
                        }
                    }
                }
            }
        },
    );
    centered_portal(current, panel, close)
}

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
