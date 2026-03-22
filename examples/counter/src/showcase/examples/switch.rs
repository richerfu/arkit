use super::super::layout::{component_canvas, h_stack};
use super::shared::DemoContext;
use arkit::prelude::*;
use arkit_shadcn as shadcn;

pub(crate) fn render(ctx: DemoContext) -> Element {
    let toggle = ctx.toggle_state.clone();
    component_canvas(
        h_stack(
            vec![
                shadcn::switch(ctx.toggle_state.clone()).into(),
                arkit::row_component()
                    .on_click(move || toggle.update(|value| *value = !*value))
                    .children(vec![shadcn::label("Airplane Mode").into()])
                    .into(),
            ],
            shadcn::theme::spacing::SM,
        ),
        true,
        24.0,
    )
}
