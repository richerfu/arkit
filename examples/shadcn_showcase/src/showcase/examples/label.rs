use super::super::layout::component_canvas;
use super::shared::DemoContext;
use crate::prelude::*;
use crate::Message;
use arkit_shadcn as shadcn;

pub(crate) struct LabelExample {
    ctx: DemoContext,
}

impl LabelExample {
    pub(crate) fn new(ctx: DemoContext) -> Self {
        Self { ctx }
    }
}

impl arkit::advanced::Widget<crate::Message, arkit::Theme, arkit::Renderer> for LabelExample {
    fn body(
        &self,
        _tree: &mut arkit::advanced::widget::Tree,
        _renderer: &arkit::Renderer,
    ) -> Option<Element> {
        let ctx = self.ctx.clone();
        Some({
            let checked = ctx.toggle_state;
            component_canvas(
                arkit::row_component()
                    .align_items_center()
                    .on_press(Message::SetToggleState(!checked))
                    .children(vec![
                        shadcn::Checkbox::new("")
                            .checked(checked)
                            .on_change(Message::SetToggleState)
                            .into(),
                        arkit::row_component()
                            .margin([0.0, 0.0, 0.0, shadcn::theme::spacing::SM])
                            .children(vec![
                                shadcn::Label::new("Accept terms and conditions").into()
                            ])
                            .into(),
                    ])
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
