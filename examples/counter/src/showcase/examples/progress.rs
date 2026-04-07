use super::super::layout::{component_canvas, fixed_width};
use super::shared::DemoContext;
use arkit::prelude::*;
use arkit_shadcn as shadcn;

#[component]
pub(crate) fn render(_ctx: DemoContext) -> Element {
    component_canvas(
        fixed_width(shadcn::progress(66.0, 100.0).into(), 288.0),
        true,
        24.0,
    )
}
