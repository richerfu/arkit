//! Breadcrumb — ColorUI `.nav` text trail.

use arkit_prelude::*;

use crate::spec;

#[component]
pub fn Breadcrumb(items: Vec<String>) -> Element {
    let total = items.len();
    let parts: Vec<Element> = items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let last = index + 1 == total;
            rsx! {
                text {
                    content: item.clone(),
                    font_size: spec::TEXT_SM,
                    font_color: if last { spec::TEXT } else { spec::TEXT_MUTED },
                }
                if !last {
                    text {
                        content: " / ",
                        font_size: spec::TEXT_SM,
                        font_color: spec::TEXT_GREY,
                    }
                }
            }
        })
        .collect();
    rsx! {
        row {
            align_items: "center",
            {parts.into_iter()}
        }
    }
}
