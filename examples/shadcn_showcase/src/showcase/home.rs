use crate::prelude::*;
use arkit_shadcn as shadcn;

use super::constants::SHOWCASE_COMPONENTS;
use super::layout::{component_list_cell, max_width, nav_bar, v_stack, ThemeMenuState};
use crate::{Message, ShowcaseState};

pub(crate) struct CatalogHome {
    search: String,
    theme: ThemeMenuState,
}

impl CatalogHome {
    pub(crate) fn new(state: &ShowcaseState) -> Self {
        Self {
            search: state.home_search.clone(),
            theme: ThemeMenuState::from(state),
        }
    }
}

impl arkit::advanced::Widget<crate::Message, arkit::Theme, arkit::Renderer> for CatalogHome {
    fn body(
        &self,
        _tree: &mut arkit::advanced::widget::Tree,
        _renderer: &arkit::Renderer,
    ) -> Option<Element> {
        let search = self.search.clone();
        let keyword = search.to_lowercase();
        let filtered = SHOWCASE_COMPONENTS
            .iter()
            .filter(|(_, name)| keyword.is_empty() || name.to_lowercase().contains(&keyword))
            .cloned()
            .collect::<Vec<_>>();

        let list = if filtered.is_empty() {
            vec![shadcn::Card::new(vec![
                shadcn::CardTitle::new("No component found").into(),
                shadcn::CardDescription::new("Try a different keyword").into(),
            ])
            .into()]
        } else {
            filtered
                .iter()
                .enumerate()
                .map(|(index, (slug, title))| {
                    component_list_cell(*slug, *title, index == 0, index + 1 == filtered.len())
                })
                .collect::<Vec<_>>()
        };

        Some(arkit::column(vec![
            nav_bar("Showcase", false, self.theme),
            arkit::scroll_component()
                .width(Length::Fill)
                .layout_weight(1.0_f32)
                .background_color(shadcn::theme::colors().background)
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
                                shadcn::Input::new("Components")
                                    .value(search)
                                    .on_input(Message::SetHomeSearch)
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
        ]))
    }
}
