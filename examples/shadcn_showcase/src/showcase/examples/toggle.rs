use super::super::layout::component_canvas;
use super::shared::DemoContext;
use crate::prelude::*;
use crate::Message;
use arkit_shadcn as shadcn;

pub(crate) fn render(ctx: DemoContext) -> Element {
    component_canvas(
        shadcn::toggle_icon("bold", ctx.toggle_state, Message::SetToggleState).into(),
        true,
        24.0,
    )
}
