use super::super::layout::component_canvas;
use super::shared::{icon_showcase, DemoContext};
use crate::prelude::*;

pub(crate) fn render(_ctx: DemoContext) -> Element {
    component_canvas(icon_showcase(), true, 24.0)
}
