use super::super::layout::{component_canvas, fixed_width};
use super::shared::DemoContext;
use crate::prelude::*;
use crate::Message;
use arkit_shadcn as shadcn;

pub(crate) struct InputExample {
    ctx: DemoContext,
}

impl InputExample {
    pub(crate) fn new(ctx: DemoContext) -> Self {
        Self { ctx }
    }
}

impl arkit::advanced::Widget<crate::Message, arkit::Theme, arkit::Renderer> for InputExample {
    fn body(
        &self,
        _tree: &mut arkit::advanced::widget::Tree,
        _renderer: &arkit::Renderer,
    ) -> Option<Element> {
        let ctx = self.ctx.clone();
        Some({
            component_canvas(
                fixed_width(
                    shadcn::Input::new("Email")
                        .value(ctx.query)
                        .on_input(Message::SetQuery)
                        .percent_width(1.0)
                        .into(),
                    384.0,
                ),
                true,
                24.0,
            )
        })
    }
}

// struct component render
