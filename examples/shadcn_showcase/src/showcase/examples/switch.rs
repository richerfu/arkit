use super::super::layout::{component_canvas, h_stack};
use super::shared::DemoContext;
use crate::prelude::*;
use crate::Message;
use arkit_shadcn as shadcn;

pub(crate) fn render(ctx: DemoContext) -> Element {
    let checked = ctx.toggle_state;
    component_canvas(
        h_stack(
            vec![
                shadcn::switch(ctx.toggle_state)
                    .on_toggle(Message::SetToggleState)
                    .into(),
                arkit::row_component()
                    .on_press(Message::SetToggleState(!checked))
                    .children(vec![shadcn::label("Airplane Mode").into()])
                    .into(),
            ],
            shadcn::theme::spacing::SM,
        ),
        true,
        24.0,
    )
}
