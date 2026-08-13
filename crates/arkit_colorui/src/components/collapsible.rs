//! Collapsible — `.cu-list.menu` row.

use arkit_prelude::*;

use crate::spec;

#[component]
pub fn Collapsible(
    title: String,
    children: Element,
    #[props(default)] open: Option<bool>,
    #[props(default)] default_open: bool,
    #[props(default)] on_open_change: EventHandler<bool>,
) -> Element {
    let mut local = use_signal(|| default_open);
    let current = open.unwrap_or_else(|| *local.read());
    let controlled = open.is_some();

    rsx! {
        column {
            width: "100%",
            background_color: spec::BG_WHITE,
            row {
                width: "100%",
                min_height: spec::LIST_ITEM,
                align_items: "center",
                justify_content: "space-between",
                padding_left: spec::PADDING,
                padding_right: spec::PADDING,
                onclick: move |_| {
                    let next = !current;
                    if !controlled {
                        local.set(next);
                    }
                    on_open_change.call(next);
                },
                text {
                    content: title,
                    font_size: spec::TEXT_DF,
                    font_color: spec::TEXT,
                }
                {arkit_icon::icon(
                    if current { "chevron-up" } else { "chevron-down" },
                    16.0,
                    spec::TEXT_GREY,
                )}
            }
            if current {
                column {
                    width: "100%",
                    padding_left: spec::PADDING,
                    padding_right: spec::PADDING,
                    padding_bottom: spec::PADDING,
                    background_color: spec::PAGE_BG,
                    {children}
                }
            }
        }
    }
}
