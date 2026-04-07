use arkit::prelude::*;
use std::rc::Rc;

use super::constants::component_title;
use super::examples::{self, DemoContext};
use super::layout::nav_bar;

#[component]
pub(crate) fn component_page(name: String, ctx: DemoContext, on_back: Rc<dyn Fn()>) -> Element {
    let title = component_title(&name);
    let demo_name = name.clone();

    arkit::column(vec![
        nav_bar(title, true, Some(on_back)),
        examples::render(&demo_name, ctx),
    ])
}
