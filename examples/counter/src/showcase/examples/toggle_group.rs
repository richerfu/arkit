use super::super::layout::component_canvas;
use super::shared::DemoContext;
use arkit::prelude::*;
use arkit_shadcn as shadcn;

#[component]
pub(crate) fn render(ctx: DemoContext) -> Element {
    component_canvas(
        shadcn::toggle_group_icons_multi(
            vec![
                String::from("bold"),
                String::from("italic"),
                String::from("underline"),
            ],
            ctx.toggle_group_values,
            {
                let on_change = ctx.on_toggle_group_values.clone();
                move |values| on_change(values)
            },
        ),
        true,
        24.0,
    )
}
