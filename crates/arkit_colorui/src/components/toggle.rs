//! ColorUI toggle — `cu-btn sm` filled when on, line when off.

use arkit_component::components::ToggleVariant;
use arkit_prelude::*;

use crate::theme::{swatch, use_colorui_theme};

#[component]
pub fn Toggle(
    label: String,
    #[props(default)] icon: Option<String>,
    #[props(default)] variant: ToggleVariant,
    #[props(default)] checked: Option<bool>,
    #[props(default)] default_checked: bool,
    #[props(default)] on_change: EventHandler<bool>,
) -> Element {
    let theme = use_colorui_theme();
    let tone = swatch(theme.primary);
    let mut local = use_signal(|| default_checked);
    let active = checked.unwrap_or_else(|| *local.read());
    let outline = matches!(variant, ToggleVariant::Outline) && !active;
    let background = if active { tone.fill } else { 0x00000000u32 };
    let foreground = if active { tone.ink } else { tone.fill };
    let border = if outline || !active {
        tone.fill
    } else {
        0x00000000u32
    };
    let is_icon = icon.is_some();

    rsx! {
        button {
            button_type: "normal",
            height: 32.0,
            width: if is_icon { Some(32.0) } else { None },
            padding_left: if is_icon { 0.0 } else { 12.0 },
            padding_right: if is_icon { 0.0 } else { 12.0 },
            background_color: background,
            border_width: if active { 0.0 } else { 1.0 },
            border_color: border,
            border_radius: 6.0,
            focusable: false,
            focus_on_touch: false,
            alignment: "center",
            onclick: move |_| {
                let current = checked.unwrap_or_else(|| *local.read());
                let next = !current;
                if checked.is_none() {
                    local.set(next);
                }
                on_change.call(next);
            },
            row {
                align_items: "center",
                justify_content: "center",
                if let Some(name) = icon {
                    {arkit_icon::icon(name, 14.0, foreground)}
                } else {
                    text {
                        content: label,
                        font_size: 14.0,
                        font_color: foreground,
                    }
                }
            }
        }
    }
}
