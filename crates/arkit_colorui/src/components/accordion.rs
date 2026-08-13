//! ColorUI accordion — `.cu-list.menu` rows that expand in place.

use arkit_component::components::AccordionItemSpec;
use arkit_prelude::*;

use super::chrome::PADDING;
use crate::theme::use_colorui_theme;

#[component]
pub fn Accordion(
    items: Vec<AccordionItemSpec>,
    value: Option<Option<String>>,
    default_value: Option<String>,
    collapsible: bool,
    on_value_change: Option<EventHandler<Option<String>>>,
) -> Element {
    let tokens = use_colorui_theme().tokens();
    let mut internal_value = use_signal(|| default_value.clone());
    let is_controlled = value.is_some();
    let current_value = value
        .clone()
        .unwrap_or_else(|| (*internal_value.read()).clone());

    let set_value = EventHandler::new(move |next: Option<String>| {
        if !is_controlled {
            internal_value.set(next.clone());
        }
        if let Some(handler) = on_value_change {
            handler.call(next);
        }
    });

    rsx! {
        column {
            width: "100%",
            background_color: tokens.colors.card,
            for item in items.iter() {
                {
                    let is_open = current_value.as_deref() == Some(item.value.as_str());
                    let item_value = item.value.clone();
                    let current_value_inner = current_value.clone();
                    let collapsible_inner = collapsible;
                    let set_value_inner = set_value;
                    let title = item.title.clone();
                    let disabled = item.disabled;
                    let content = item.content.clone();
                    rsx! {
                        column {
                            width: "100%",
                            row {
                                width: "100%",
                                min_height: 50.0,
                                align_items: "center",
                                justify_content: "space-between",
                                padding_left: PADDING,
                                padding_right: PADDING,
                                opacity: if disabled { 0.5f32 } else { 1.0f32 },
                                onclick: move |_| {
                                    if disabled {
                                        return;
                                    }
                                    let next = if current_value_inner.as_deref()
                                        == Some(item_value.as_str())
                                    {
                                        if collapsible_inner {
                                            None
                                        } else {
                                            Some(item_value.clone())
                                        }
                                    } else {
                                        Some(item_value.clone())
                                    };
                                    set_value_inner.call(next);
                                },
                                row {
                                    layout_weight: 1.0,
                                    text {
                                        content: title,
                                        font_size: 15.0,
                                        font_color: tokens.colors.foreground,
                                    }
                                }
                                {arkit_icon::icon(
                                    if is_open { "chevron-up" } else { "chevron-down" },
                                    16.0,
                                    0xFF8799A3,
                                )}
                            }
                            if is_open {
                                column {
                                    width: "100%",
                                    padding_left: PADDING,
                                    padding_right: PADDING,
                                    padding_bottom: PADDING,
                                    background_color: 0xFFF1F1F1u32,
                                    {content}
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
