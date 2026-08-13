//! Accordion — official trigger row + chevron, borderless stack.

use arkit_component::components::AccordionItemSpec;
use arkit_prelude::*;

use crate::spec;
use crate::theme::use_theme;

#[component]
pub fn Accordion(
    items: Vec<AccordionItemSpec>,
    value: Option<Option<String>>,
    default_value: Option<String>,
    collapsible: bool,
    on_value_change: Option<EventHandler<Option<String>>>,
) -> Element {
    let theme = use_theme();
    let mut internal = use_signal(|| default_value.clone());
    let controlled = value.is_some();
    let current = value.unwrap_or_else(|| (*internal.read()).clone());
    let set_value = EventHandler::new(move |next: Option<String>| {
        if !controlled {
            internal.set(next.clone());
        }
        if let Some(handler) = on_value_change {
            handler.call(next);
        }
    });
    rsx! {
        column {
            width: "100%",
            for item in items.iter() {
                {
                    let open = current.as_deref() == Some(item.value.as_str());
                    let item_value = item.value.clone();
                    let current_inner = current.clone();
                    let title = item.title.clone();
                    let disabled = item.disabled;
                    let content = item.content.clone();
                    rsx! {
                        column {
                            width: "100%",
                            row {
                                width: "100%",
                                padding_top: 16.0,
                                padding_bottom: 16.0,
                                align_items: "center",
                                justify_content: "space-between",
                                opacity: if disabled { spec::DISABLED_OPACITY } else { 1.0 },
                                onclick: move |_| {
                                    if disabled {
                                        return;
                                    }
                                    let next = if current_inner.as_deref() == Some(item_value.as_str()) {
                                        if collapsible { None } else { Some(item_value.clone()) }
                                    } else {
                                        Some(item_value.clone())
                                    };
                                    set_value.call(next);
                                },
                                text {
                                    content: title,
                                    font_size: spec::TEXT_SM,
                                    font_weight: spec::FONT_MEDIUM,
                                    font_color: theme.colors.foreground,
                                }
                                {arkit_icon::icon(
                                    if open { "chevron-up" } else { "chevron-down" },
                                    16.0,
                                    theme.colors.muted_foreground,
                                )}
                            }
                            if open {
                                column {
                                    width: "100%",
                                    padding_bottom: 16.0,
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
