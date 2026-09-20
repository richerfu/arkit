//! Textarea — shadcn-style multi-line text input.
//!
//! Migrated from the original Elm builder API to dioxus 0.7 `#[component]` +
//! `rsx!`. Preserves the original styling: input-surface shell with a
//! background fill, `[SM, MD, SM, MD]` padding, `text-sm`, 64vp min height,
//! and a translucent `muted_foreground` placeholder.

use crate::theme::*;
use arkit_prelude::*;

use super::ARKUI_BORDER_STYLE_SOLID;

/// Props for [`Textarea`].
#[derive(Props, Clone, PartialEq)]
pub struct TextareaProps {
    #[props(default)]
    pub accessibility_label: Option<String>,
    #[props(default)]
    pub accessibility_description: Option<String>,
    pub placeholder: Option<String>,
    pub value: Option<String>,
    pub height: Option<f32>,
    /// CSS width (`"100%"`, `"50%"`). Takes precedence over `full`.
    pub width: Option<String>,
    /// Fill the available parent width, matching shadcn's `w-full` default.
    #[props(default = true)]
    pub full: bool,
    /// Uses the destructive border treatment for validation failures.
    #[props(default)]
    pub invalid: bool,
    #[props(default)]
    pub required: bool,
    /// Prevents editing while preserving the field's dimensions.
    #[props(default)]
    pub disabled: bool,
    /// Requires an explicit click to focus instead of grabbing focus on touch-down.
    #[props(default)]
    pub click_to_focus: bool,
    pub on_change: Option<EventHandler<String>>,
    pub on_click: Option<EventHandler<()>>,
}

/// A multi-line text input.
#[component]
pub fn Textarea(props: TextareaProps) -> Element {
    let theme = use_theme();
    let on_change = props.on_change;
    let on_click = props.on_click;
    let disabled = props.disabled;
    let click_to_focus = props.click_to_focus;
    let width = props
        .width
        .or_else(|| props.full.then(|| "100%".to_string()));
    let mut focus_request = use_signal(|| false);
    let accessibility_description = props
        .accessibility_description
        .clone()
        .into_iter()
        .chain(props.invalid.then_some("Invalid".to_string()))
        .chain(props.required.then_some("Required".to_string()))
        .collect::<Vec<_>>()
        .join(". ");

    rsx! {
        textarea {
            accessibility_role: "text_area",
            accessibility_text: if let Some(label) = props.accessibility_label { label },
            accessibility_description: if !accessibility_description.is_empty() { accessibility_description },
            accessibility_disabled: disabled,
            value: if let Some(v) = props.value { v },
            placeholder: if let Some(p) = props.placeholder { p },
            placeholder_color: with_alpha(theme.colors.muted_foreground, 0x80),
            caret_color: theme.colors.primary,
            font_size: typography::SM,
            font_color: theme.colors.foreground,
            line_height: 20.0,
            height: props.height.unwrap_or(64.0),
            border_style: ARKUI_BORDER_STYLE_SOLID,
            border_width: 1.0,
            border_color: if props.invalid { theme.colors.destructive } else { theme.colors.input },
            border_radius: theme.radii.md,
            shadow: shadow::XS,
            background_color: theme.colors.background,
            opacity: if disabled { 0.5 } else { 1.0 },
            enabled: !disabled,
            focusable: !disabled,
            focus_on_touch: !click_to_focus,
            focused: focus_request(),
            padding_top: spacing::SM,
            padding_right: spacing::MD,
            padding_bottom: spacing::SM,
            padding_left: spacing::MD,
            width: if let Some(w) = width { w },
            onchange: move |evt| {
                if !disabled {
                    if let Some(handler) = on_change {
                        handler.call(evt.data().string_value.clone());
                    }
                }
            },
            onclick: move |_| {
                if !disabled {
                    if click_to_focus {
                        focus_request.set(true);
                    }
                    if let Some(handler) = on_click {
                        handler.call(());
                    }
                }
            },
            onfocus: move |_| focus_request.set(false),
        }
    }
}
