use arkit::prelude::*;
use arkit_shadcn as shadcn;
use std::rc::Rc;

use super::constants::SHOWCASE_COMPONENTS;
use super::layout::{component_list_cell, max_width, nav_bar, v_stack};

#[component]
pub(crate) fn catalog_home(
    search: String,
    on_search: Rc<dyn Fn(String)>,
    on_open: Rc<dyn Fn(String)>,
) -> Element {
    let keyword = search.to_lowercase();
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
                component_list_cell(
                    *slug,
                    *title,
                    index == 0,
                    index + 1 == filtered.len(),
                    on_open.clone(),
                )
            })
            .collect::<Vec<_>>()
    };

    arkit::column(vec![
        nav_bar("Showcase", false, None),
        arkit::scroll_component()
            .width(Length::Fill)
            .style(ArkUINodeAttributeType::LayoutWeight, 1.0_f32)
            .background_color(shadcn::theme::color::BACKGROUND)
            .children(vec![max_width(
                arkit::column_component()
                    .width(Length::Fill)
                    .padding([
                        4.0,
                        shadcn::theme::spacing::LG,
                        8.0,
                        shadcn::theme::spacing::LG,
                    ])
                    .children(vec![v_stack(
                        vec![
                            shadcn::input("Components")
                                .value(search)
                                .on_change(move |value| on_search(value))
                                .width(Length::Fill)
                                .into(),
                            arkit::column_component()
                                .width(Length::Fill)
                                .children(list)
                                .into(),
                        ],
                        shadcn::theme::spacing::LG,
                    )])
                    .into(),
                512.0,
            )])
            .into(),
    ])
}
