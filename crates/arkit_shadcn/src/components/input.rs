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

const NUMBER_INPUT_FILTER: &str = "[0-9]";

/// Input semantics and native keyboard profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputMode {
    /// General single-line text.
    #[default]
    Text,
    /// Password text, masked by ArkUI with a trailing visibility toggle.
    Password,
    /// ASCII digits only, using the native numeric keyboard.
    Number,
}

impl InputMode {
    const fn native_input_type(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Password => "password",
            Self::Number => "number",
        }
    }

    const fn native_input_filter(self) -> Option<&'static str> {
        match self {
            Self::Number => Some(NUMBER_INPUT_FILTER),
            Self::Text | Self::Password => None,
        }
    }

    fn sanitize(self, value: String) -> String {
        if self == Self::Number {
            value
                .chars()
                .filter(char::is_ascii_digit)
                .collect::<String>()
        } else {
            value
        }
    }
}

/// Props for [`Input`].
#[derive(Props, Clone, PartialEq)]
pub struct InputProps {
    pub placeholder: Option<String>,
    pub value: Option<String>,
    /// Text, password, or digits-only input behavior.
    #[props(default)]
    pub mode: InputMode,
    #[props(default)]
    pub height: Option<f32>,
    /// CSS width (`"100%"`, `"50%"`). Unset leaves the field content-sized.
    pub width: Option<String>,
    /// Uses the destructive border treatment for validation failures.
    #[props(default)]
    pub invalid: bool,
    /// Prevents editing while preserving the field's dimensions.
    #[props(default)]
    pub disabled: bool,
    /// Keeps the normal input appearance while preventing focus and IME entry.
    /// Pair with `on_click` to use the field as a custom-keyboard trigger.
    #[props(default)]
    pub read_only: bool,
    pub on_change: Option<EventHandler<String>>,
    pub on_click: Option<EventHandler<()>>,
}

/// A single-line text input.
#[component]
pub fn Input(props: InputProps) -> Element {
    let theme = use_theme();
    let mode = props.mode;
    let value = props.value.map(|value| mode.sanitize(value));
    let input_type = mode.native_input_type();
    let input_filter = mode.native_input_filter();
    let on_change = props.on_change;
    let on_click = props.on_click;

    rsx! {
        textinput {
            value: if let Some(value) = value { value },
            placeholder: if let Some(p) = props.placeholder { p },
            input_type,
            input_filter: if let Some(filter) = input_filter { filter },
            show_password_icon: mode == InputMode::Password,
            placeholder_color: with_alpha(theme.colors.muted_foreground, 0x80),
            caret_color: if props.read_only {
                0x00000000
            } else {
                theme.colors.primary
            },
            font_size: typography::LG,
            font_color: theme.colors.foreground,
            line_height: 22.5,
            height: props.height.unwrap_or(48.0),
            border_style: ARKUI_BORDER_STYLE_SOLID,
            border_width: 1.0,
            border_color: if props.invalid { theme.colors.destructive } else { theme.colors.input },
            border_radius: theme.radii.md,
            background_color: theme.colors.background,
            opacity: if props.disabled { 0.5 } else { 1.0 },
            enabled: !props.disabled,
            focusable: !props.read_only,
            focus_on_touch: !props.read_only,
            padding_top: spacing::XXS,
            padding_right: spacing::MD,
            padding_bottom: spacing::XXS,
            padding_left: spacing::MD,
            width: if let Some(w) = props.width { w },
            on_change: move |evt| {
                if !props.disabled && !props.read_only {
                    if let Some(handler) = on_change {
                        handler.call(mode.sanitize(evt.data().string_value.clone()));
                    }
                }
            },
            onclick: move |_| {
                if !props.disabled {
                    if let Some(handler) = on_click {
                        handler.call(());
                    }
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modes_select_native_input_profiles() {
        assert_eq!(InputMode::default(), InputMode::Text);
        assert_eq!(InputMode::Text.native_input_type(), "text");
        assert_eq!(InputMode::Password.native_input_type(), "password");
        assert_eq!(InputMode::Number.native_input_type(), "number");
        assert_eq!(
            InputMode::Number.native_input_filter(),
            Some(NUMBER_INPUT_FILTER)
        );
        assert_eq!(InputMode::Password.native_input_filter(), None);
    }

    #[test]
    fn number_mode_rejects_non_ascii_digits() {
        assert_eq!(
            InputMode::Number.sanitize("12a ٣4.5-6".to_string()),
            "12456"
        );
        assert_eq!(
            InputMode::Password.sanitize("p@ss word".to_string()),
            "p@ss word"
        );
    }
}
