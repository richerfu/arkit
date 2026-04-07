use super::super::layout::component_canvas;
use super::shared::DemoContext;
use arkit::prelude::*;
use arkit_shadcn as shadcn;

pub(crate) fn render(ctx: DemoContext) -> Element {
    let toggle = ctx.toggle_state.clone();
    let close_cancel = ctx.toggle_state.clone();
    let close_continue = ctx.toggle_state.clone();

    arkit::stack_component()
        .percent_width(1.0)
        .percent_height(1.0)
        .children(vec![
            component_canvas(
                shadcn::button("Show Alert Dialog", shadcn::ButtonVariant::Outline)
                    .on_click(move || toggle.update(|open| *open = !*open))
                    .into(),
                true,
                24.0,
            ),
            shadcn::alert_dialog_modal(
                ctx.toggle_state,
                "Are you absolutely sure?",
                "This action cannot be undone. This will permanently delete your account and remove your data from our servers.",
                vec![
                    shadcn::button("Cancel", shadcn::ButtonVariant::Outline)
                        .percent_width(1.0)
                        .on_click(move || close_cancel.set(false))
                        .into(),
                    shadcn::button("Continue", shadcn::ButtonVariant::Default)
                        .percent_width(1.0)
                        .on_click(move || close_continue.set(false))
                        .into(),
                ],
            ),
        ])
        .into()
}
