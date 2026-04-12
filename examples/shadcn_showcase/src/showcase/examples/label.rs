use super::super::layout::component_canvas;
use super::shared::DemoContext;
use crate::prelude::*;
use crate::Message;
use arkit_shadcn as shadcn;

pub(crate) fn render(ctx: DemoContext) -> Element {
    let checked = ctx.toggle_state;
    component_canvas(
        arkit::row_component()
            .align_items_center()
            .on_press(Message::SetToggleState(!checked))
            .children(vec![
                shadcn::checkbox("", checked, Message::SetToggleState),
                arkit::row_component()
                    .margin([0.0, 0.0, 0.0, shadcn::theme::spacing::SM])
                    .children(vec![shadcn::label("Accept terms and conditions").into()])
                    .into(),
            ])
            .into(),
        true,
        24.0,
    )
}
