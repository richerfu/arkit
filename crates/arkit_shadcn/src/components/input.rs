//! Input — shadcn-style single-line text input.
//!
//! Migrated from the original Elm builder API to dioxus 0.7 `#[component]` +
//! `rsx!`. Mirrors React Native Reusables native styling: 48px-tall
//! `TextInput` with an
//! input-surface shell (1px `input` border, `md` radius, `background` fill),
//! `lg` font size, and a translucent `muted_foreground` placeholder.

use crate::theme::*;
use arkit_prelude::*;

use super::ARKUI_BORDER_STYLE_SOLID;

/// Props for [`Input`].
#[derive(Props, Clone, PartialEq)]
pub struct InputProps {
    pub placeholder: Option<String>,
    pub value: Option<String>,
    #[props(default)]
    pub height: Option<f32>,
    pub percent_width: Option<f32>,
    /// Uses the destructive border treatment for validation failures.
    #[props(default)]
    pub invalid: bool,
    /// Prevents editing while preserving the field's dimensions.
    #[props(default)]
    pub disabled: bool,
    pub on_change: Option<EventHandler<String>>,
}

/// A single-line text input.
#[component]
pub fn Input(props: InputProps) -> Element {
    let theme = use_theme();
    let on_change = props.on_change;

    rsx! {
        textinput {
            value: if let Some(v) = props.value { v },
            placeholder: if let Some(p) = props.placeholder { p },
            placeholder_color: with_alpha(theme.colors.muted_foreground, 0x80),
            caret_color: theme.colors.primary,
            font_size: typography::LG,
            line_height: 22.5,
            height: props.height.unwrap_or(48.0),
            border_style: ARKUI_BORDER_STYLE_SOLID,
            border_width: 1.0,
            border_color: if props.invalid { theme.colors.destructive } else { theme.colors.input },
            border_radius: theme.radii.md,
            background_color: theme.colors.background,
            opacity: if props.disabled { 0.5 } else { 1.0 },
            enabled: !props.disabled,
            padding_top: spacing::XXS,
            padding_right: spacing::MD,
            padding_bottom: spacing::XXS,
            padding_left: spacing::MD,
            percent_width: if let Some(w) = props.percent_width { w },
            on_change: move |evt| {
                if !props.disabled {
                    if let Some(handler) = on_change {
                        handler.call(evt.data().string_value.clone());
                    }
                }
            },
        }
    }
}
