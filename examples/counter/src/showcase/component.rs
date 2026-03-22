use arkit::prelude::*;

use super::constants::component_title;
use super::examples::{self, DemoContext};
use super::layout::nav_bar;

#[component]
pub(crate) fn component_page(name: String) -> Element {
    let title = component_title(&name);
    let demo_name = name.clone();

    arkit::column(vec![
        nav_bar(title, true),
        arkit::scope(move || {
            let ctx = DemoContext {
                active_tab: use_signal(|| 0usize),
                page: use_signal(|| 1_i32),
                radio_choice: use_signal(|| String::from("Comfortable")),
                select_choice: use_signal(String::new),
                query: use_signal(String::new),
                toggle_state: use_signal(|| false),
            };

            examples::render(&demo_name, ctx)
        }),
    ])
}
