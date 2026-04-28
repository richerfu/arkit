use crate::prelude::*;

use super::constants::component_title;
use super::examples::{DemoContext, ExampleRenderer};
use super::layout::{nav_bar, ThemeMenuState};

pub(crate) struct ComponentPage {
    name: String,
    ctx: DemoContext,
}

impl ComponentPage {
    pub(crate) fn new(name: String, ctx: DemoContext) -> Self {
        Self { name, ctx }
    }
}

impl arkit::advanced::Widget<crate::Message, arkit::Theme, arkit::Renderer> for ComponentPage {
    fn body(
        &self,
        _tree: &mut arkit::advanced::widget::Tree,
        _renderer: &arkit::Renderer,
    ) -> Option<Element> {
        let name = self.name.clone();
        let ctx = self.ctx.clone();
        let title = component_title(&name);
        let demo_name = name.clone();

        Some(arkit::column(vec![
            nav_bar(
                title,
                true,
                ThemeMenuState {
                    mode: ctx.theme_mode,
                    preset: ctx.theme_preset,
                    custom: ctx.custom_theme,
                    open: ctx.theme_menu_open,
                },
            ),
            Element::new(ExampleRenderer::new(demo_name, ctx)),
        ]))
    }
}
