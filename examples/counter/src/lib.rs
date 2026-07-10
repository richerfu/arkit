//! Counter example — dioxus `rsx!` + `use_signal` driven by an ArkUI renderer.

use arkit::entry;
use arkit::prelude::*;

#[entry]
fn app() -> Element {
    let mut count = use_signal(|| 0);

    rsx! {
        column {
            percent_width: 1.0,
            percent_height: 1.0,
            align_items: "center",
            justify_content: "center",

            text {
                font_size: 28.0,
                line_height: 32.0,
                "count = {count}"
            }

            button {
                margin_top: 12.0,
                onclick: move |_| count += 1,
                "increment"
            }
        }
    }
}
