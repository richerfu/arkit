use super::super::layout::{component_canvas, fixed_width};
use super::shared::DemoContext;
use arkit::prelude::*;
use arkit_shadcn as shadcn;

pub(crate) fn render(ctx: DemoContext) -> Element {
    component_canvas(
        fixed_width(
            shadcn::input("Email")
                .value(ctx.query)
                .on_change(move |value| (ctx.on_query)(value))
                .percent_width(1.0)
                .into(),
            384.0,
        ),
        true,
        24.0,
    )
}
