use arkit::prelude::*;
use arkit_shadcn as shadcn;

use super::constants::SHOWCASE_COMPONENTS;
use super::layout::{component_list_cell, nav_bar, page_scroll};

#[component]
pub(crate) fn catalog_home(search: Signal<String>) -> Element {
    let keyword = search.get().to_lowercase();
    let filtered = SHOWCASE_COMPONENTS
        .iter()
        .filter(|(_, name)| keyword.is_empty() || name.to_lowercase().contains(&keyword))
        .cloned()
        .collect::<Vec<_>>();

    let list = if filtered.is_empty() {
        vec![shadcn::card(vec![
            shadcn::card_title("No component found"),
            shadcn::card_description("Try a different keyword"),
        ])]
    } else {
        filtered
            .iter()
            .enumerate()
            .map(|(index, (slug, title))| {
                component_list_cell(*slug, *title, index == 0, index + 1 == filtered.len())
            })
            .collect::<Vec<_>>()
    };

    arkit::column(vec![
        nav_bar("Showcase", false),
        page_scroll(vec![
            shadcn::input("Components")
                .bind(search)
                .percent_width(1.0)
                .into(),
            arkit::column_component()
                .percent_width(1.0)
                .children(list)
                .into(),
        ]),
    ])
}
