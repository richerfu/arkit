//! Accordion — collapsible sections (single-open mode by default).
//!
//! Ported from the legacy Elm builder `accordion.rs`. Each item has a trigger
//! row (title + rotating chevron) and a content panel that expands/collapses.
//! The open item is held in a `Signal<Option<String>>`; toggling an open item
//! closes it when `collapsible` is true. The legacy native transform-based
//! chevron rotation is approximated by swapping `chevron-down`/`chevron-up`.

use super::motion::ExpandPresence;
use crate::theme::*;
use arkit_prelude::*;

const ACCORDION_TRIGGER_GAP: f32 = spacing::LG;
const ACCORDION_ICON_SIZE: f32 = 16.0;

/// A single accordion item spec: title, value, content, disabled flag.
#[derive(Debug, Clone, PartialEq)]
pub struct AccordionItemSpec {
    pub title: String,
    pub value: String,
    pub content: Element,
    pub disabled: bool,
}

impl AccordionItemSpec {
    pub fn new(title: impl Into<String>, value: impl Into<String>, content: Element) -> Self {
        Self {
            title: title.into(),
            value: value.into(),
            content,
            disabled: false,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

#[component]
pub fn Accordion(
    items: Vec<AccordionItemSpec>,
    value: Option<Option<String>>,
    default_value: Option<String>,
    collapsible: bool,
    on_value_change: Option<EventHandler<Option<String>>>,
) -> Element {
    let theme = use_theme();
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

    let colors = &theme.colors;
    let md = theme.radii.md;

    rsx! {
        column {
            width: "100%",
            align_items: "start",
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
                    let chevron_icon = if is_open { "chevron-up" } else { "chevron-down" };
                    let fg = colors.foreground;
                    let border = colors.border;
                    rsx! {
                        column {
                            width: "100%",
                            row {
                                width: "100%",
                                align_items: "start",
                                justify_content: "start",
                                padding_top: spacing::LG,
                                padding_bottom: spacing::LG,
                                border_radius: md,
                                opacity: if disabled { 0.5f32 } else { 1.0f32 },
                                onclick: move |_: dioxus_core::Event<_>| {
                                    if disabled { return; }
                                    let next = if current_value_inner.as_deref() == Some(item_value.as_str()) {
                                        if collapsible_inner { None } else { Some(item_value.clone()) }
                                    } else {
                                        Some(item_value.clone())
                                    };
                                    set_value_inner.call(next);
                                },
                                column {
                                    layout_weight: 1.0,
                                    align_items: "start",
                                    margin_right: ACCORDION_TRIGGER_GAP,
                                    text {
                                        font_size: typography::SM,
                                        font_weight: 500i32,
                                        font_color: fg,
                                        line_height: 20.0,
                                        {title}
                                    }
                                }
                                row {
                                    width: ACCORDION_ICON_SIZE,
                                    height: ACCORDION_ICON_SIZE,
                                    align_items: "center",
                                    justify_content: "center",
                                    {crate::icon::icon_placeholder(chevron_icon, ACCORDION_ICON_SIZE, colors.muted_foreground)}
                                }
                            }
                            ExpandPresence {
                                open: is_open,
                                column {
                                    width: "100%",
                                    align_items: "start",
                                    padding_bottom: spacing::LG,
                                    {content}
                                }
                            }
                            row {
                                width: "100%",
                                height: 1.0,
                                background_color: border,
                            }
                        }
                    }
                }
            }
        }
    }
}
