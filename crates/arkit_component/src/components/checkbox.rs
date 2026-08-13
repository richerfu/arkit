//! Checkbox — unstyled checkbox.

use super::ARKUI_BORDER_STYLE_SOLID;
use crate::appearance::{CheckboxAppearance, CheckboxStyleInput};
use crate::style::use_style_kit;
use arkit_prelude::*;

/// Props for [`Checkbox`].
#[derive(Props, Clone, PartialEq)]
pub struct CheckboxProps {
    pub label: Option<String>,
    pub checked: Option<bool>,
    pub default_checked: Option<bool>,
    pub checked_color: Option<u32>,
    pub disabled: Option<bool>,
    pub on_change: Option<EventHandler<bool>>,
    #[props(default)]
    pub appearance: Option<CheckboxAppearance>,
}

/// A checkbox with an optional label.
#[component]
pub fn Checkbox(props: CheckboxProps) -> Element {
    let kit = use_style_kit();
    let mut internal = use_signal(|| props.default_checked.unwrap_or(false));
    let controlled = props.checked.is_some();
    let current = props.checked.unwrap_or_else(|| *internal.read());
    let disabled = props.disabled.unwrap_or(false);
    let appearance = props.appearance.unwrap_or_else(|| {
        kit.checkbox(&CheckboxStyleInput {
            checked: current,
            checked_color: props.checked_color,
        })
    });
    let on_change = props.on_change;
    let label = props.label.clone();
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
            align_items: "center",
            justify_content: "start",
            opacity: if disabled { 0.5 } else { 1.0 },
            onclick: move |event| {
                event.stop_propagation();
                toggle.call(());
            },
            stack {
                width: appearance.size,
                height: appearance.size,
                alignment: "center",
                border_radius: appearance.radius,
                border_width: appearance.border_width,
                border_style: ARKUI_BORDER_STYLE_SOLID,
                border_color: appearance.border_color,
                background_color: appearance.background,
                clip: true,
                if current {
                    {arkit_icon::icon(
                        "check",
                        appearance.icon_size,
                        appearance.check_color,
                    )}
                }
            }
            if let Some(text) = label.as_ref() {
                row {
                    margin_left: appearance.label_gap,
                    onclick: move |event| {
                        event.stop_propagation();
                        toggle_from_label.call(());
                    },
                    text {
                        content: text.clone(),
                        font_size: appearance.label_size,
                        font_weight: 500,
                        font_color: appearance.label_color,
                        text_align: "start",
                    }
                }
            }
        }
    }
}
