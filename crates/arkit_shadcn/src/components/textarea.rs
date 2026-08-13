//! Textarea — official `min-h-16` / `text-base` / `rounded-md` field.

use arkit_prelude::*;

use crate::spec;
use crate::theme::use_theme;

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
    let theme = use_theme();
    rsx! {
        textarea {
            value: if let Some(v) = value { v },
            placeholder: if let Some(p) = placeholder { p },
            placeholder_color: with_alpha(theme.colors.muted_foreground, 0x80),
            caret_color: theme.colors.primary,
            font_size: spec::TEXT_BASE,
            font_color: theme.colors.foreground,
            line_height: 20.0,
            height: height.unwrap_or(64.0),
            border_width: 1.0,
            border_color: if invalid {
                theme.colors.destructive
            } else {
                theme.colors.input
            },
            border_radius: spec::RADIUS_MD,
            background_color: 0x00000000u32,
            opacity: if disabled { spec::DISABLED_OPACITY } else { 1.0 },
            enabled: !disabled,
            padding_top: 8.0,
            padding_right: 12.0,
            padding_bottom: 8.0,
            padding_left: 12.0,
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

fn with_alpha(color: u32, alpha: u32) -> u32 {
    (color & 0x00FF_FFFF) | (alpha << 24)
}
