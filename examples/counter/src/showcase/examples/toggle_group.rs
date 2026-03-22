use super::super::layout::component_canvas;
use super::shared::DemoContext;
use arkit::prelude::*;
use arkit_shadcn as shadcn;

#[component]
pub(crate) fn render(_ctx: DemoContext) -> Element {
    let values = use_signal(|| Vec::<String>::new());

    component_canvas(
        shadcn::toggle_group_icons_multi(
            vec![
                String::from("bold"),
                String::from("italic"),
                String::from("underline"),
            ],
            values,
        ),
        true,
        24.0,
    )
}
