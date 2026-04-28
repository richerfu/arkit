use super::super::layout::{component_canvas, h_stack};
use super::shared::DemoContext;
use crate::prelude::*;
use arkit_shadcn as shadcn;

pub(crate) struct SwitchExample {
    ctx: DemoContext,
}

impl SwitchExample {
    pub(crate) fn new(ctx: DemoContext) -> Self {
        Self { ctx }
    }
}

impl arkit::advanced::Widget<crate::Message, arkit::Theme, arkit::Renderer> for SwitchExample {
    fn body(
        &self,
        _tree: &mut arkit::advanced::widget::Tree,
        _renderer: &arkit::Renderer,
    ) -> Option<Element> {
        let _ctx = self.ctx.clone();
        Some({
            component_canvas(
                h_stack(
                    vec![
                        shadcn::Switch::new(false).default_checked(false).into(),
                        arkit::row_component()
                            .children(vec![shadcn::Label::new("Airplane Mode").into()])
                            .into(),
                    ],
                    shadcn::theme::spacing::SM,
                ),
                true,
                24.0,
            )
        })
    }
}

// struct component render
