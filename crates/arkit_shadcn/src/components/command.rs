//! Command palette — filterable input + command list.
//!
//! Ported from the legacy Elm builder `command.rs`. The input filters the
//! options by case-insensitive substring; selecting an option fires
//! `on_query_change` with the option text (mirroring the legacy behavior where
//! the query was set to the clicked option). The panel uses `panel_surface`
//! styling (`popover` fill, 1px `border`, `md` radius) with `XXS`
//! padding; the input uses `input_surface` styling and its bottom border
//! separates it from the option list.

use super::ARKUI_BORDER_STYLE_SOLID;
use crate::{i18n::use_component_i18n, theme::*};
use arkit_prelude::*;

#[component]
pub fn Command(
    query: String,
    options: Vec<String>,
    placeholder: Option<String>,
    on_query_change: Option<EventHandler<String>>,
) -> Element {
    let theme = use_theme();
    let i18n = use_component_i18n();
    let placeholder = placeholder.unwrap_or_else(|| i18n.command_placeholder());
    let colors = &theme.colors;
    let md = theme.radii.md;
    let keyword = query.to_lowercase();

    rsx! {
        column {
            focus_navigation: "vertical",
            width: "100%",
            background_color: colors.popover,
            border_radius: md,
            border_width: 1.0,
            border_color: colors.border,
            padding_top: spacing::XXS,
            padding_right: spacing::XXS,
            padding_bottom: spacing::XXS,
            padding_left: spacing::XXS,
            textinput {
                accessibility_role: "searchbox",
                accessibility_text: placeholder.clone(),
                value: query.clone(),
                placeholder,
                placeholder_color: with_alpha(colors.muted_foreground, 0x80),
                font_size: typography::SM,
                font_color: colors.foreground,
                line_height: 20.0,
                height: control::HEIGHT,
                border_style: ARKUI_BORDER_STYLE_SOLID,
                border_width: 1.0,
                border_color: colors.input,
                border_radius: md,
                background_color: colors.background,
                padding_top: 0.0,
                padding_right: 12.0,
                padding_bottom: 0.0,
                padding_left: 12.0,
                width: "100%",
                onchange: move |evt| {
                    if let Some(handler) = on_query_change {
                        handler.call(evt.data.string_value.clone());
                    }
                },
            }
            for option in options.iter() {
                {
                    let passes = keyword.is_empty() || option.to_lowercase().contains(&keyword);
                    let opt = option.clone();
                    let on_query_change_inner = on_query_change;
                    rsx! {
                        if passes {
                            CommandOption {
                                option: opt,
                                on_select: move |value| {
                                    if let Some(handler) = on_query_change_inner {
                                        handler.call(value);
                                    }
                                },
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn CommandOption(option: String, on_select: EventHandler<String>) -> Element {
    let theme = use_theme();
    let mut hovering = use_signal(|| false);
    let mut focused = use_signal(|| false);
    let click_value = option.clone();
    let key_value = option.clone();

    rsx! {
        row {
            accessibility_role: "button",
            accessibility_text: option.clone(),
            accessibility_group: true,
            accessibility_actions: "click",
            width: "100%",
            height: 32.0,
            align_items: "center",
            padding_top: 6.0,
            padding_right: spacing::SM,
            padding_bottom: 6.0,
            padding_left: spacing::SM,
            border_radius: theme.radii.sm,
            background_color: if hovering() || focused() {
                theme.colors.accent
            } else {
                0x00000000
            },
            focusable: true,
            focus_on_touch: true,
            onclick: move |_| on_select.call(click_value.clone()),
            onkey: move |event| {
                if event.data().activates() {
                    on_select.call(key_value.clone());
                }
            },
            onhover: move |event| hovering.set(event.data().is_hovering),
            onfocus: move |_| focused.set(true),
            onblur: move |_| focused.set(false),
            text {
                font_size: typography::SM,
                font_color: theme.colors.foreground,
                line_height: 20.0,
                {option.clone()}
            }
        }
    }
}
