//! Combobox — editable input + filterable dropdown of options.
//!
//! Ported from the legacy Elm builder `combobox.rs`. The floating-panel overlay
//! collapses to inline rendering toggled on click. The trigger shows a search
//! icon, the selected value (or placeholder), and a chevrons-up-down icon; the
//! dropdown lists options with a check mark on the active one.

use super::motion::ExpandPresence;
use crate::{i18n::use_component_i18n, theme::*};
use arkit_prelude::*;

const COMBOBOX_PANEL_FALLBACK_WIDTH: f32 = 240.0;

#[component]
pub fn Combobox(
    options: Vec<String>,
    placeholder: Option<String>,
    label: Option<String>,
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
    let sm = theme.radii.sm;
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

    rsx! {
        column {
            align_items: "start",
            row {
                height: 40.0,
                width: "100%",
                background_color: colors.background,
                padding_top: 8.0,
                padding_right: spacing::MD,
                padding_bottom: 8.0,
                padding_left: spacing::MD,
                align_items: "center",
                justify_content: "space_between",
                border_radius: md,
                border_width: 1.0,
                border_color: colors.border,
                shadow: "sm",
                onclick: move |_: dioxus_core::Event<_>| {
                    set_open.call(!current_open);
                },
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
            ExpandPresence {
                open: current_open,
                column {
                    width: panel_width,
                    background_color: colors.popover,
                    border_radius: md,
                    border_width: 1.0,
                    border_color: colors.border,
                    shadow: "sm",
                    if let Some(label) = panel_label {
                        row {
                            padding_top: 8.0,
                            padding_right: spacing::SM,
                            padding_bottom: 8.0,
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
                            {
                                let active = selected == *option;
                                let opt = option.clone();
                                let on_select_inner = on_select;
                                let fg = if active { colors.accent_foreground } else { colors.foreground };
                                rsx! {
                                    row {
                                        width: "100%",
                                        height: 36.0,
                                        align_items: "center",
                                        justify_content: "space_between",
                                        padding_top: 8.0,
                                        padding_right: spacing::SM,
                                        padding_bottom: 8.0,
                                        padding_left: spacing::SM,
                                        border_radius: sm,
                                        background_color: if active { colors.accent } else { 0x00000000 },
                                        onclick: move |_: dioxus_core::Event<_>| {
                                            if let Some(handler) = on_select_inner {
                                                handler.call(opt.clone());
                                            }
                                            set_open.call(false);
                                        },
                                        text {
                                            font_size: typography::SM,
                                            font_color: fg,
                                            line_height: 20.0,
                                            {option.clone()}
                                        }
                                        if active {
                                            {crate::icon::icon_placeholder("check", 16.0, colors.foreground)}
                                        } else {
                                            row { width: 16.0, height: 16.0 }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
