use super::shared::{no_padding_center_canvas, select_carousel, DemoContext};
use crate::prelude::*;

pub(crate) struct SelectExample {
    ctx: DemoContext,
}

impl SelectExample {
    pub(crate) fn new(ctx: DemoContext) -> Self {
        Self { ctx }
    }
}

impl arkit::advanced::Widget<crate::Message, arkit::Theme, arkit::Renderer> for SelectExample {
    fn body(
        &self,
        _tree: &mut arkit::advanced::widget::Tree,
        _renderer: &arkit::Renderer,
    ) -> Option<Element> {
        let ctx = self.ctx.clone();
        Some({
            no_padding_center_canvas(select_carousel(
                ctx.page,
                ctx.select_choice,
                ctx.select_open,
            ))
        })
    }
}

// struct component render
