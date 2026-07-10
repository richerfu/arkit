//! Textarea — shadcn-style multi-line text input.
//!
//! Migrated from the original Elm builder API to dioxus 0.7 `#[component]` +
//! `rsx!`. Preserves the original styling: input-surface shell with a
//! transparent fill, `[SM, MD, SM, MD]` padding, `md` font size, 64px height,
//! and a translucent `muted_foreground` placeholder.

use crate::theme::*;
use arkit_prelude::*;

use super::ARKUI_BORDER_STYLE_SOLID;

/// Props for [`Textarea`].
#[derive(Props, Clone, PartialEq)]
pub struct TextareaProps {
    pub placeholder: Option<String>,
    pub value: Option<String>,
    pub height: Option<f32>,
    pub percent_width: Option<f32>,
    pub on_change: Option<EventHandler<String>>,
}

/// A multi-line text input.
#[component]
pub fn Textarea(props: TextareaProps) -> Element {
    let theme = use_theme();
    let on_change = props.on_change;

    rsx! {
        textarea {
            value: if let Some(v) = props.value { v },
            placeholder: if let Some(p) = props.placeholder { p },
            placeholder_color: with_alpha(theme.colors.muted_foreground, 0x80),
            caret_color: theme.colors.primary,
            font_size: typography::MD,
            line_height: 20.0,
            height: props.height.unwrap_or(64.0),
            border_style: ARKUI_BORDER_STYLE_SOLID,
            border_width: 1.0,
            border_color: theme.colors.input,
            border_radius: theme.radii.md,
            background_color: 0x00000000,
            padding_top: spacing::SM,
            padding_right: spacing::MD,
            padding_bottom: spacing::SM,
            padding_left: spacing::MD,
            percent_width: if let Some(w) = props.percent_width { w },
            on_change: move |evt| {
                if let Some(handler) = on_change {
                    handler.call(evt.data().string_value.clone());
                }
            },
        }
    }
}
