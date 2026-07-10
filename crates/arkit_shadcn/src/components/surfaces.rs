//! Surface components — toast notifications and sonner stack.
//!
//! Migrated from the original Elm builder API to dioxus 0.7 `#[component]` +
//! `rsx!`. Preserves the original `toast`, `toast_destructive` and `sonner`
//! styling: each toast is a `panel_surface` (`shadow-sm` + `rounded(md)` + 1px
//! `border` + `bg=popover` + `fg=popover_foreground`) row with
//! `[SM, LG, SM, LG]` padding and `body_text_regular` content. The destructive
//! variant overrides the border color to `destructive` and the text color to
//! `destructive`.

use super::Card;
use crate::theme::*;
use arkit_prelude::*;

/// Props for [`Toast`].
#[derive(Props, Clone, PartialEq)]
pub struct ToastProps {
    pub message: String,
}

/// A panel-surface toast displaying `message` in foreground body text.
#[component]
pub fn Toast(props: ToastProps) -> Element {
    let theme = use_theme();
    rsx! {
        row {
            percent_width: 1.0,
            align_items: "center",
            background_color: theme.colors.popover,
            foreground_color: theme.colors.popover_foreground,
            border_width: 1.0,
            border_color: theme.colors.border,
            border_radius: theme.radii.md,
            shadow: 1,
            padding_top: spacing::SM,
            padding_right: spacing::LG,
            padding_bottom: spacing::SM,
            padding_left: spacing::LG,
            text {
                content: props.message.clone(),
                font_size: typography::MD,
                font_color: theme.colors.foreground,
                line_height: 20.0,
                text_align: 0,
            }
        }
    }
}

/// Props for [`ToastDestructive`].
#[derive(Props, Clone, PartialEq)]
pub struct ToastDestructiveProps {
    pub message: String,
}

/// A destructive toast — panel surface with a `destructive` border and
/// destructive-colored body text.
#[component]
pub fn ToastDestructive(props: ToastDestructiveProps) -> Element {
    let theme = use_theme();
    rsx! {
        row {
            percent_width: 1.0,
            background_color: theme.colors.popover,
            foreground_color: theme.colors.popover_foreground,
            border_width: 1.0,
            border_color: theme.colors.destructive,
            border_radius: theme.radii.md,
            shadow: 1,
            padding_top: spacing::SM,
            padding_right: spacing::LG,
            padding_bottom: spacing::SM,
            padding_left: spacing::LG,
            text {
                content: props.message.clone(),
                font_size: typography::MD,
                font_color: theme.colors.destructive,
                line_height: 20.0,
                text_align: 0,
            }
        }
    }
}

/// Props for [`Sonner`].
#[derive(Props, Clone, PartialEq)]
pub struct SonnerProps {
    pub messages: Vec<String>,
}

/// A stack of toasts rendered inside a [`Card`].
#[component]
pub fn Sonner(props: SonnerProps) -> Element {
    let messages = props.messages.clone();
    rsx! {
        Card {
            for message in messages {
                Toast { message: message }
            }
        }
    }
}
