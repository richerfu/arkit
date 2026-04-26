use super::super::layout::{component_canvas, fixed_width};
use super::shared::DemoContext;
use crate::prelude::*;
use crate::Message;
use arkit_shadcn as shadcn;

pub(crate) struct RadioGroupExample {
    ctx: DemoContext,
}

impl RadioGroupExample {
    pub(crate) fn new(ctx: DemoContext) -> Self {
        Self { ctx }
    }
}

impl arkit::advanced::Widget<crate::Message, arkit::Theme, arkit::Renderer> for RadioGroupExample {
    fn body(
        &self,
        _tree: &mut arkit::advanced::widget::Tree,
        _renderer: &arkit::Renderer,
    ) -> Option<Element> {
        let ctx = self.ctx.clone();
        Some({
            component_canvas(
                fixed_width(
                    shadcn::RadioGroup::new(vec![
                        String::from("Default"),
                        String::from("Comfortable"),
                        String::from("Compact"),
                    ])
                    .selected(ctx.radio_choice)
                    .on_select(Message::SetRadioChoice)
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
