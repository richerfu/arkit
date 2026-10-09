//! Checkbox — shadcn-style checkbox.
//!
//! Migrated from the original Elm builder API to dioxus 0.7 `#[component]` +
//! `rsx!`. Matches the React Native Reusables primitive: a 16x16 indicator
//! with `sm` radius, 1px primary border, primary fill only when checked, and a
//! 16px `check` lucide icon in `primary_foreground`. Supports
//! controlled/uncontrolled state, a custom selected color, `disabled`, an
//! optional label, and `on_change`.

use super::ARKUI_BORDER_STYLE_SOLID;
use crate::theme::*;
use arkit_prelude::*;

const CHECKBOX_SIZE: f32 = 16.0;
const CHECKBOX_BORDER_WIDTH: f32 = 1.0;
const CHECKBOX_ICON_SIZE: f32 = 16.0;

/// Props for [`Checkbox`].
#[derive(Props, Clone, PartialEq)]
pub struct CheckboxProps {
    pub label: Option<String>,
    /// Accessible name used when the visual label is absent or insufficient.
    #[props(default)]
    pub accessibility_label: Option<String>,
    pub checked: Option<bool>,
    pub default_checked: Option<bool>,
    pub checked_color: Option<u32>,
    pub disabled: Option<bool>,
    pub on_change: Option<EventHandler<bool>>,
}

/// A checkbox with an optional label.
#[component]
pub fn Checkbox(props: CheckboxProps) -> Element {
    let theme = use_theme();
    let mut hovering = use_signal(|| false);
    let mut focused = use_signal(|| false);
    let mut internal = use_signal(|| props.default_checked.unwrap_or(false));
    let controlled = props.checked.is_some();
    let current = props.checked.unwrap_or_else(|| *internal.read());
    let checked_color = props.checked_color.unwrap_or(theme.colors.primary);
    let disabled = props.disabled.unwrap_or(false);
    let on_change = props.on_change;
    let label = props.label.clone();
    let accessibility_label = props
        .accessibility_label
        .clone()
        .or_else(|| props.label.clone());
    let toggle = EventHandler::new(move |_: ()| {
        if disabled {
            return;
        }
        let next = !current;
        if !controlled {
            internal.set(next);
        }
        if let Some(handler) = on_change {
            handler.call(next);
        }
    });
    let toggle_from_label = toggle;

    rsx! {
        row {
            accessibility_role: "checkbox",
            accessibility_text: if let Some(label) = accessibility_label { label },
            accessibility_group: true,
            accessibility_actions: "click",
            accessibility_checked: current,
            accessibility_disabled: disabled,
            focusable: !disabled,
            focus_on_touch: true,
            enabled: !disabled,
            align_items: "center",
            justify_content: "start",
            padding: spacing::XXS,
            border_radius: theme.radii.md,
            background_color: if hovering() || focused() {
                theme.colors.accent
            } else {
                0x00000000
            },
            opacity: if disabled { 0.5 } else { 1.0 },
            onclick: move |event| {
                event.stop_propagation();
                toggle.call(());
            },
            onkey: move |event| {
                if event.data().activates() {
                    toggle.call(());
                }
            },
            onhover: move |event| hovering.set(event.data().is_hovering),
            onfocus: move |_| focused.set(true),
            onblur: move |_| focused.set(false),
            stack {
                width: CHECKBOX_SIZE,
                height: CHECKBOX_SIZE,
                alignment: "center",
                border_radius: theme.radii.sm,
                border_width: CHECKBOX_BORDER_WIDTH,
                border_style: ARKUI_BORDER_STYLE_SOLID,
                border_color: checked_color,
                background_color: if current { checked_color } else { theme.colors.background },
                shadow: shadow::XS,
                clip: false,
                if current {
                    {arkit_icon::icon(
                        "check",
                        CHECKBOX_ICON_SIZE,
                        theme.colors.primary_foreground,
                    )}
                }
            }
            if let Some(text) = label.as_ref() {
                row {
                    margin_left: spacing::SM,
                    onclick: move |event| {
                        event.stop_propagation();
                        toggle_from_label.call(());
                    },
                    text {
                        content: text.clone(),
                        font_size: typography::SM,
                        font_weight: 500,
                        font_color: theme.colors.foreground,
                        text_align: "start",
                    }
                }
            }
        }
    }
}
