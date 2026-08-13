//! Alert dialog — centered modal with RN Reusables alert-specific header and
//! footer semantics. The full-screen modal/backdrop is a declarative portal.

use super::dialog::{dialog_portal, use_dialog_close, DialogCloseProvider, DIALOG_MAX_WIDTH};
use crate::style::*;
use arkit_prelude::*;

/// Modal alert dialog.
///
/// # Controlled vs uncontrolled
///
/// - **Controlled**: pass `open: Some(bool)` and update it from `on_close` /
///   action handlers.
/// - **Uncontrolled**: omit `open` (or pass `None`) and use `default_open`.
///   Backdrop dismiss and [`use_dialog_close`] both flip the internal open
///   flag. Custom `cancel` / `action` buttons must call [`use_dialog_close`]
///   (or unmount the dialog); empty `onclick` handlers will not close it.
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

    // Parent modal shell is full-width; panel fills it up to DIALOG_MAX_WIDTH.
    let panel = rsx! {
        DialogCloseProvider {
            close,
            column {
                width: "100%",
                max_width: DIALOG_MAX_WIDTH,
                padding_top: spacing::XXL,
                padding_right: spacing::XXL,
                padding_bottom: spacing::XXL,
                padding_left: spacing::XXL,
                border_radius: theme.radii.lg,
                border_width: 1.0,
                border_color: theme.colors.border,
                background_color: theme.colors.background,
                shadow: "sm",
                column {
                    width: "100%",
                    text {
                        width: "100%",
                        font_size: typography::XL,
                        font_weight: 600,
                        font_color: theme.colors.foreground,
                        line_height: 24.0,
                        text_align: "start",
                        "{title}"
                    }
                    text {
                        width: "100%",
                        margin_top: spacing::SM,
                        font_size: typography::MD,
                        font_color: theme.colors.muted_foreground,
                        line_height: 20.0,
                        text_align: "start",
                        "{description}"
                    }
                }
                column {
                    width: "100%",
                    margin_top: spacing::LG,
                    if let Some(action) = action {
                        {action}
                    }
                    if has_action && has_cancel {
                        row {
                            width: "100%",
                            height: spacing::SM,
                        }
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
    };

    dialog_portal(current, panel, close)
}

/// Convenience cancel/action button that always invokes [`use_dialog_close`].
///
/// Use inside [`AlertDialog`] slots so uncontrolled dialogs dismiss correctly
/// even when the parent does not own an `open` signal.
#[component]
pub fn AlertDialogAction(
    #[props(default)] variant: Option<super::button::ButtonVariant>,
    #[props(default)] width: Option<String>,
    #[props(default)] onclick: Option<EventHandler<()>>,
    children: Element,
) -> Element {
    let close = use_dialog_close();
    let variant = variant.unwrap_or(super::button::ButtonVariant::Default);
    rsx! {
        super::button::Button {
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
