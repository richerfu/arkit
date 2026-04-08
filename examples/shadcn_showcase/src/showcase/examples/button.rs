use super::shared::{button_carousel, no_padding_center_canvas, DemoContext};
use crate::prelude::*;

pub(crate) fn render(ctx: DemoContext) -> Element {
    no_padding_center_canvas(button_carousel(ctx.page))
}
