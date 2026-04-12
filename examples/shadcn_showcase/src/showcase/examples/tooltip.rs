use super::super::layout::component_canvas;
use super::shared::DemoContext;
use crate::prelude::*;
use crate::Message;
use arkit_shadcn as shadcn;

pub(crate) fn render(ctx: DemoContext) -> Element {
    component_canvas(
        shadcn::tooltip(
            shadcn::button("Press")
                .theme(shadcn::ButtonVariant::Outline)
                .on_press(Message::SetTooltipOpen(!ctx.tooltip_open))
                .into(),
            "Add to library",
            ctx.tooltip_open,
            Message::SetTooltipOpen,
        ),
        true,
        24.0,
    )
}
