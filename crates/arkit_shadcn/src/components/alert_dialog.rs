//! Alert dialog — centered modal with RN Reusables alert-specific header and
//! footer semantics. The full-screen modal/backdrop native tree is owned by
//! `arkit_hooks::use_overlay`, matching Dialog.

use super::dialog::{use_dialog_overlay, DIALOG_MAX_WIDTH};
use super::floating_layer::SHADOW_SM;
use crate::theme::*;
use arkit_prelude::*;

/// Modal alert dialog.
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

    let panel = rsx! {
        column {
            percent_width: 1.0,
            max_width_constraint: DIALOG_MAX_WIDTH,
            padding_top: spacing::XXL,
            padding_right: spacing::XXL,
            padding_bottom: spacing::XXL,
            padding_left: spacing::XXL,
            border_radius: theme.radii.lg,
            border_width: 1.0,
            border_color: theme.colors.border,
            background_color: theme.colors.background,
            shadow: SHADOW_SM,
            column {
                percent_width: 1.0,
                text {
                    percent_width: 1.0,
                    font_size: typography::XL,
                    font_weight: 600_i32,
                    font_color: theme.colors.foreground,
                    line_height: 24.0,
                    text_align: 0,
                    "{title}"
                }
                text {
                    percent_width: 1.0,
                    margin_top: spacing::SM,
                    font_size: typography::MD,
                    font_color: theme.colors.muted_foreground,
                    line_height: 20.0,
                    text_align: 0,
                    "{description}"
                }
            }
            column {
                percent_width: 1.0,
                margin_top: spacing::LG,
                if let Some(action) = action {
                    {action}
                }
                if has_action && has_cancel {
                    row {
                        percent_width: 1.0,
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
    };

    use_dialog_overlay(current, panel, close);
    rsx! {}
}
