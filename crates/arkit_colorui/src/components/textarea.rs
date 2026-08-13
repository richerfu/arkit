//! ColorUI textarea — form-group multi-line field, no shadcn chrome.

use arkit_prelude::*;

use crate::theme::use_colorui_theme;

#[component]
pub fn Textarea(
    placeholder: Option<String>,
    value: Option<String>,
    height: Option<f32>,
    width: Option<String>,
    #[props(default)] invalid: bool,
    #[props(default)] disabled: bool,
    on_change: Option<EventHandler<String>>,
) -> Element {
    let tokens = use_colorui_theme().tokens();
    let on_change = on_change;

    rsx! {
        textarea {
            value: if let Some(v) = value { v },
            placeholder: if let Some(p) = placeholder { p },
            placeholder_color: 0xFF888888u32,
            caret_color: tokens.colors.primary,
            font_size: 14.0,
            font_color: 0xFF555555u32,
            line_height: 20.0,
            height: height.unwrap_or(92.0),
            border_width: 0.0,
            background_color: tokens.colors.card,
            opacity: if disabled { 0.6 } else { 1.0 },
            enabled: !disabled,
            padding_top: 8.0,
            padding_right: 0.0,
            padding_bottom: 8.0,
            padding_left: 0.0,
            width: if let Some(w) = width { w },
            on_change: move |evt| {
                if !disabled {
                    if let Some(handler) = on_change {
                        handler.call(evt.data().string_value.clone());
                    }
                }
            },
        }
    }
}
