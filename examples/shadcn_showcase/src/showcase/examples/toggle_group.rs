use super::super::layout::component_canvas;
use super::shared::DemoContext;
use crate::prelude::*;
use crate::Message;
use arkit_shadcn as shadcn;

pub(crate) fn render(ctx: DemoContext) -> Element {
    component_canvas(
        shadcn::toggle_group_icons_multi(
            vec![
                String::from("bold"),
                String::from("italic"),
                String::from("underline"),
            ],
            ctx.toggle_group_values,
            Message::SetToggleGroupValues,
        ),
        true,
        24.0,
    )
}
