use crate::prelude::*;

use super::constants::component_title;
use super::examples::{self, DemoContext};
use super::layout::nav_bar;

pub(crate) fn component_page(name: String, ctx: DemoContext) -> Element {
    let title = component_title(&name);
    let demo_name = name.clone();

    arkit::column(vec![
        nav_bar(title, true),
        examples::render(&demo_name, ctx),
    ])
}
