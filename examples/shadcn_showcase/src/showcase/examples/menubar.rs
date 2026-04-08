use super::shared::{top_center_canvas, DemoContext};
use crate::prelude::*;
use arkit_shadcn as shadcn;

pub(crate) fn render(_ctx: DemoContext) -> Element {
    top_center_canvas(
        shadcn::menubar(vec![
            shadcn::menubar_item("File"),
            shadcn::menubar_item("Edit"),
            shadcn::menubar_item("View"),
            shadcn::menubar_item("Profiles"),
        ]),
        [16.0, 16.0, 16.0, 16.0],
        true,
    )
}
