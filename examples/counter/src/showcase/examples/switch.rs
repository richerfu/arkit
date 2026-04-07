use super::super::layout::{component_canvas, h_stack};
use super::shared::DemoContext;
use arkit::prelude::*;
use arkit_shadcn as shadcn;

pub(crate) fn render(ctx: DemoContext) -> Element {
    let on_switch_change = ctx.on_toggle_state.clone();
    let on_row_toggle = ctx.on_toggle_state.clone();
    let checked = ctx.toggle_state;
    component_canvas(
        h_stack(
            vec![
                shadcn::switch(ctx.toggle_state)
                    .on_change(move |value| on_switch_change(value))
                    .into(),
                arkit::row_component()
                    .on_click(move || on_row_toggle(!checked))
                    .children(vec![shadcn::label("Airplane Mode").into()])
                    .into(),
            ],
            shadcn::theme::spacing::SM,
        ),
        true,
        24.0,
    )
}
