use super::super::layout::{component_canvas, fixed_width};
use super::shared::DemoContext;
use arkit::prelude::*;
use arkit::queue_after_mount;
use arkit_shadcn as shadcn;

#[component]
pub(crate) fn render(_ctx: DemoContext) -> Element {
    let value = create_signal(13.0_f32);
    let progress = value.clone();
    queue_after_mount(move || progress.set(66.0));

    component_canvas(
        fixed_width(shadcn::progress(value.get(), 100.0).into(), 288.0),
        true,
        24.0,
    )
}
