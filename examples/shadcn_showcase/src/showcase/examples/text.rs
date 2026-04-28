use super::shared::{no_padding_center_canvas, text_carousel, DemoContext};
use crate::prelude::*;

pub(crate) struct TextExample {
    ctx: DemoContext,
}

impl TextExample {
    pub(crate) fn new(ctx: DemoContext) -> Self {
        Self { ctx }
    }
}

impl arkit::advanced::Widget<crate::Message, arkit::Theme, arkit::Renderer> for TextExample {
    fn body(
        &self,
        _tree: &mut arkit::advanced::widget::Tree,
        _renderer: &arkit::Renderer,
    ) -> Option<Element> {
        let ctx = self.ctx.clone();
        Some(no_padding_center_canvas(text_carousel(ctx.page)))
    }
}

// struct component render
