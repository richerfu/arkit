//! Command — ColorUI search bar + menu list.

use arkit_prelude::*;

use crate::spec;
use crate::theme::{swatch, use_colorui_theme};

#[component]
pub fn Command(
    query: String,
    options: Vec<String>,
    placeholder: Option<String>,
    on_query_change: Option<EventHandler<String>>,
) -> Element {
    let fill = swatch(use_colorui_theme().primary).fill;
    let placeholder = placeholder.unwrap_or_else(|| "搜索".into());
    let keyword = query.to_lowercase();
    let rows: Vec<Element> = options
        .iter()
        .filter(|option| option.to_lowercase().contains(&keyword))
        .map(|option| {
            let value = option.clone();
            rsx! {
                row {
                    width: "100%",
                    min_height: spec::LIST_ITEM,
                    padding_left: spec::PADDING,
                    padding_right: spec::PADDING,
                    align_items: "center",
                    background_color: spec::BG_WHITE,
                    onclick: move |_| {
                        if let Some(handler) = on_query_change {
                            handler.call(value.clone());
                        }
                    },
                    text {
                        content: option.clone(),
                        font_size: spec::TEXT_DF,
                        font_color: spec::TEXT,
                    }
                }
            }
        })
        .collect();

    rsx! {
        column {
            width: "100%",
            background_color: spec::BG_WHITE,
            row {
                width: "100%",
                height: spec::BAR_HEIGHT,
                align_items: "center",
                padding_left: spec::PADDING,
                padding_right: spec::PADDING,
                background_color: spec::SEARCH_BG,
                textinput {
                    value: query.clone(),
                    placeholder,
                    font_size: spec::TEXT_SM,
                    font_color: spec::TEXT,
                    background_color: 0x00000000u32,
                    border_width: 0.0,
                    height: 32.0,
                    caret_color: fill,
                    on_change: move |evt| {
                        if let Some(handler) = on_query_change {
                            handler.call(evt.data().string_value.clone());
                        }
                    },
                }
            }
            {rows.into_iter()}
        }
    }
}
