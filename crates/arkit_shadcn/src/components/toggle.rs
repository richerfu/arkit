//! Toggle — official outline/default two-state button (`h-9`).

use arkit_component::components::ToggleVariant;
use arkit_prelude::*;

use crate::spec;
use crate::theme::use_theme;

#[component]
pub fn Toggle(
    label: String,
    #[props(default)] icon: Option<String>,
    #[props(default)] variant: ToggleVariant,
    #[props(default)] checked: Option<bool>,
    #[props(default)] default_checked: bool,
    #[props(default)] on_change: EventHandler<bool>,
) -> Element {
    let theme = use_theme();
    let mut local = use_signal(|| default_checked);
    let active = checked.unwrap_or_else(|| *local.read());
    let outline = matches!(variant, ToggleVariant::Outline);
    let background = if active {
        theme.colors.secondary
    } else if outline {
        theme.colors.background
    } else {
        0x01000000u32
    };
    let foreground = theme.colors.foreground;
    let is_icon = icon.is_some();
    rsx! {
        button {
            button_type: "normal",
            height: spec::BTN_HEIGHT_SM,
            width: if is_icon { Some(spec::BTN_HEIGHT_SM) } else { None },
            padding_left: if is_icon { 0.0 } else { 10.0 },
            padding_right: if is_icon { 0.0 } else { 10.0 },
            background_color: background,
            border_width: if outline { 1.0 } else { 0.0 },
            border_color: theme.colors.input,
            border_radius: spec::RADIUS_MD,
            shadow: if outline { "sm" },
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
                if let Some(name) = icon {
                    {arkit_icon::icon(name, 16.0, foreground)}
                } else {
                    text {
                        content: label,
                        font_size: spec::TEXT_SM,
                        font_weight: spec::FONT_MEDIUM,
                        font_color: foreground,
                    }
                }
            }
        }
    }
}
