use super::super::layout::component_canvas;
use super::shared::DemoContext;
use arkit::prelude::*;
use arkit_shadcn as shadcn;

pub(crate) fn render(ctx: DemoContext) -> Element {
    component_canvas(
        shadcn::toggle_icon("bold", ctx.toggle_state, move |value| (ctx.on_toggle_state)(value))
            .into(),
        true,
        24.0,
    )
}
