use super::super::layout::component_canvas;
use super::shared::DemoContext;
use crate::prelude::*;
use crate::Message;
use arkit_shadcn as shadcn;

pub(crate) struct TooltipExample {
    ctx: DemoContext,
}

impl TooltipExample {
    pub(crate) fn new(ctx: DemoContext) -> Self {
        Self { ctx }
    }
}

impl arkit::advanced::Widget<crate::Message, arkit::Theme, arkit::Renderer> for TooltipExample {
    fn body(
        &self,
        _tree: &mut arkit::advanced::widget::Tree,
        _renderer: &arkit::Renderer,
    ) -> Option<Element> {
        let ctx = self.ctx.clone();
        Some({
            component_canvas(
                shadcn::Tooltip::new(
                    shadcn::Button::new("Press")
                        .theme(shadcn::ButtonVariant::Outline)
                        .on_press(Message::SetTooltipOpen(!ctx.tooltip_open))
                        .into(),
                    "Add to library",
                )
                .open(ctx.tooltip_open)
                .on_open_change(Message::SetTooltipOpen)
                .into(),
                true,
                24.0,
            )
        })
    }
}

// struct component render
