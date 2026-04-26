use super::super::layout::component_canvas;
use super::shared::DemoContext;
use crate::prelude::*;
use crate::Message;
use arkit_shadcn as shadcn;

pub(crate) struct ToggleGroupExample {
    ctx: DemoContext,
}

impl ToggleGroupExample {
    pub(crate) fn new(ctx: DemoContext) -> Self {
        Self { ctx }
    }
}

impl arkit::advanced::Widget<crate::Message, arkit::Theme, arkit::Renderer> for ToggleGroupExample {
    fn body(
        &self,
        _tree: &mut arkit::advanced::widget::Tree,
        _renderer: &arkit::Renderer,
    ) -> Option<Element> {
        let ctx = self.ctx.clone();
        Some({
            component_canvas(
                shadcn::ToggleGroup::new(vec![
                    String::from("bold"),
                    String::from("italic"),
                    String::from("underline"),
                ])
                .icons(true)
                .multi(true)
                .selected(ctx.toggle_group_values)
                .on_change(Message::SetToggleGroupValues)
                .into(),
                true,
                24.0,
            )
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn into_any(self: Box<Self>) -> Box<dyn std::any::Any> {
        self
    }
}

// struct component render
