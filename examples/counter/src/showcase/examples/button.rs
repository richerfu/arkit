use super::shared::{button_carousel, no_padding_center_canvas, DemoContext};
use arkit::prelude::*;

pub(crate) fn render(ctx: DemoContext) -> Element {
    no_padding_center_canvas(button_carousel(ctx.page, ctx.on_page))
}
