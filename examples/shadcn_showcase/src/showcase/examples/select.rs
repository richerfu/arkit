use super::shared::{no_padding_center_canvas, select_carousel, DemoContext};
use crate::prelude::*;

pub(crate) fn render(ctx: DemoContext) -> Element {
    no_padding_center_canvas(select_carousel(
        ctx.page,
        ctx.select_choice,
        ctx.select_open,
    ))
}
