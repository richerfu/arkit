//! Command palette — filterable input + command list.
//!
//! Ported from the legacy Elm builder `command.rs`. The input filters the
//! options by case-insensitive substring; selecting an option fires
//! `on_query_change` with the option text (mirroring the legacy behavior where
//! the query was set to the clicked option). The panel uses `panel_surface`
//! styling (`popover` fill, 1px `border`, `md` radius, `shadow-sm`) with `XXS`
//! padding; the input uses `input_surface` styling and its bottom border
//! separates it from the option list.

use super::ARKUI_BORDER_STYLE_SOLID;
use crate::theme::*;
use arkit_prelude::*;

#[component]
pub fn Command(
    query: String,
    options: Vec<String>,
    on_query_change: Option<EventHandler<String>>,
) -> Element {
    let theme = use_theme();
    let colors = &theme.colors;
    let sm = theme.radii.sm;
    let md = theme.radii.md;
    let keyword = query.to_lowercase();

    rsx! {
        column {
            percent_width: 1.0,
            background_color: colors.popover,
            border_radius: md,
            border_width: 1.0,
            border_color: colors.border,
            shadow: 1i32,
            padding_top: spacing::XXS,
            padding_right: spacing::XXS,
            padding_bottom: spacing::XXS,
            padding_left: spacing::XXS,
            textinput {
                value: query.clone(),
                placeholder: "Search command...".to_string(),
                placeholder_color: with_alpha(colors.muted_foreground, 0x80),
                font_size: typography::MD,
                font_color: colors.foreground,
                line_height: 20.0,
                height: 40.0,
                border_style: ARKUI_BORDER_STYLE_SOLID,
                border_width: 1.0,
                border_color: colors.input,
                border_radius: md,
                background_color: colors.background,
                padding_top: 0.0,
                padding_right: 12.0,
                padding_bottom: 0.0,
                padding_left: 12.0,
                percent_width: 1.0,
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
                            row {
                                percent_width: 1.0,
                                height: 32.0,
                                align_items: "center",
                                padding_top: 6.0,
                                padding_right: spacing::SM,
                                padding_bottom: 6.0,
                                padding_left: spacing::SM,
                                border_radius: sm,
                                onclick: move |_: dioxus_core::Event<_>| {
                                    if let Some(handler) = on_query_change_inner {
                                        handler.call(opt.clone());
                                    }
                                },
                                text {
                                    font_size: typography::SM,
                                    font_color: colors.foreground,
                                    line_height: 20.0,
                                    {option.clone()}
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
