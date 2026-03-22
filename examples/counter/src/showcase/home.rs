use arkit::prelude::*;
use arkit_shadcn as shadcn;

use super::constants::SHOWCASE_COMPONENTS;
use super::layout::{component_list_cell, max_width, nav_bar, v_stack};

#[component]
pub(crate) fn catalog_home() -> Element {
    arkit::scope(|| {
        let search = use_signal(String::new);
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
            arkit::scroll_component()
                .percent_width(1.0)
                .style(ArkUINodeAttributeType::LayoutWeight, 1.0_f32)
                .background_color(shadcn::theme::color::BACKGROUND)
                .children(vec![max_width(
                    arkit::column_component()
                        .percent_width(1.0)
                        .style(
                            ArkUINodeAttributeType::Padding,
                            vec![
                                0.0,
                                shadcn::theme::spacing::LG,
                                8.0,
                                shadcn::theme::spacing::LG,
                            ],
                        )
                        .children(vec![v_stack(
                            vec![
                                shadcn::input("Components")
                                    .bind(search)
                                    .percent_width(1.0)
                                    .into(),
                                arkit::column_component()
                                    .percent_width(1.0)
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
    })
}
