use super::super::layout::{component_canvas, fixed_width};
use super::shared::DemoContext;
use crate::prelude::*;
use crate::Message;
use arkit_shadcn as shadcn;

pub(crate) fn render(ctx: DemoContext) -> Element {
    component_canvas(
        fixed_width(
            shadcn::radio_group(
                vec![
                    String::from("Default"),
                    String::from("Comfortable"),
                    String::from("Compact"),
                ],
                ctx.radio_choice,
                Message::SetRadioChoice,
            ),
            384.0,
        ),
        true,
        24.0,
    )
}
