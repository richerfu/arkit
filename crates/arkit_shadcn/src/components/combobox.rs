//! Combobox — editable input + filterable dropdown of options.
//!
//! Ported from the legacy Elm builder `combobox.rs`. The floating-panel overlay
//! collapses to inline rendering toggled on click. The trigger shows a search
//! icon, the selected value (or placeholder), and a chevrons-up-down icon; the
//! dropdown lists options with a check mark on the active one.

use super::Popover;
use crate::{i18n::use_component_i18n, theme::*};
use arkit_prelude::*;

const COMBOBOX_PANEL_FALLBACK_WIDTH: f32 = 240.0;

#[component]
pub fn Combobox(
    options: Vec<String>,
    placeholder: Option<String>,
    label: Option<String>,
    accessibility_label: Option<String>,
    #[props(default)] disabled: bool,
    selected: String,
    open: Option<bool>,
    default_open: bool,
    on_open_change: Option<EventHandler<bool>>,
    on_select: Option<EventHandler<String>>,
) -> Element {
    let theme = use_theme();
    let i18n = use_component_i18n();
    let mut internal_open = use_signal(|| default_open);
    let is_controlled = open.is_some();
    let current_open = open.unwrap_or_else(|| *internal_open.read());

    let set_open = EventHandler::new(move |value: bool| {
        if !is_controlled {
            internal_open.set(value);
        }
        if let Some(handler) = on_open_change {
            handler.call(value);
        }
    });

    let colors = &theme.colors;
    let md = theme.radii.md;
    let has_value = !selected.is_empty();
    let trigger_label = if has_value {
        selected.clone()
    } else {
        placeholder.unwrap_or_else(|| i18n.combobox_placeholder())
    };
    let panel_label = label
        .or_else(|| Some(i18n.combobox_label()))
        .filter(|label| !label.is_empty());
    let label_color = if has_value {
        colors.foreground
    } else {
        colors.muted_foreground
    };
    let panel_width = COMBOBOX_PANEL_FALLBACK_WIDTH;
    let accessible_name = accessibility_label
        .or_else(|| panel_label.clone())
        .unwrap_or_else(|| i18n.combobox_label());

    let trigger = rsx! {
        row {
            height: control::HEIGHT,
            width: "100%",
            background_color: colors.background,
            padding_top: 0.0,
            padding_right: spacing::MD,
            padding_bottom: 0.0,
            padding_left: spacing::MD,
            align_items: "center",
            justify_content: "space_between",
            border_radius: md,
            border_width: 1.0,
            border_color: colors.border,
            shadow: shadow::XS,
            row {
                align_items: "center",
                {crate::icon::icon_placeholder("search", 16.0, colors.muted_foreground)}
                row {
                    margin_left: spacing::SM,
                    text {
                        font_size: typography::SM,
                        font_color: label_color,
                        line_height: 20.0,
                        {trigger_label}
                    }
                }
            }
            {crate::icon::icon_placeholder("chevrons-up-down", 16.0, colors.muted_foreground)}
        }
    };

    rsx! {
        Popover {
            trigger,
            accessibility_label: Some(accessible_name),
            disabled,
            open: Some(current_open),
            default_open: Some(default_open),
            on_open_change: move |next| { if !disabled { set_open.call(next); } },
            width: panel_width,
            padding: 0.0,
            column {
                focus_navigation: "vertical",
                width: "100%",
                background_color: colors.popover,
                if let Some(label) = panel_label {
                    row {
                        padding_top: 6.0,
                        padding_right: spacing::SM,
                        padding_bottom: 6.0,
                        padding_left: spacing::SM,
                        text {
                            font_size: typography::XS,
                            font_color: colors.muted_foreground,
                            line_height: 16.0,
                            {label}
                        }
                    }
                }
                column {
                    width: "100%",
                    padding_top: spacing::XXS,
                    padding_right: spacing::XXS,
                    padding_bottom: spacing::XXS,
                    padding_left: spacing::XXS,
                    for option in options.iter() {
                        ComboboxOption {
                            option: option.clone(),
                            active: selected == *option,
                            on_select: move |value| {
                                if let Some(handler) = on_select {
                                    handler.call(value);
                                }
                                set_open.call(false);
                            },
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ComboboxOption(option: String, active: bool, on_select: EventHandler<String>) -> Element {
    let theme = use_theme();
    let mut hovering = use_signal(|| false);
    let mut focused = use_signal(|| false);
    let foreground = if active {
        theme.colors.accent_foreground
    } else {
        theme.colors.foreground
    };
    let click_value = option.clone();
    let key_value = option.clone();

    rsx! {
        row {
            accessibility_role: "radio",
            accessibility_text: option.clone(),
            accessibility_group: true,
            accessibility_actions: "click",
            accessibility_checked: active,
            accessibility_selected: active,
            width: "100%",
            height: control::HEIGHT_SM,
            align_items: "center",
            justify_content: "space_between",
            padding_top: 6.0,
            padding_right: spacing::SM,
            padding_bottom: 6.0,
            padding_left: spacing::SM,
            border_radius: theme.radii.sm,
            background_color: if active || hovering() || focused() {
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
                font_color: foreground,
                line_height: 20.0,
                {option.clone()}
            }
            if active {
                {crate::icon::icon_placeholder("check", 16.0, theme.colors.foreground)}
            } else {
                row { width: 16.0, height: 16.0 }
            }
        }
    }
}
