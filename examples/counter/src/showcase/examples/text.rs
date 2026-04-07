use super::shared::{no_padding_center_canvas, text_carousel, DemoContext};
use arkit::prelude::*;

pub(crate) fn render(ctx: DemoContext) -> Element {
    no_padding_center_canvas(text_carousel(ctx.page, ctx.on_page))
}
