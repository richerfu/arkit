use super::super::layout::component_canvas;
use super::shared::DemoContext;
use arkit::prelude::*;
use arkit_shadcn as shadcn;

pub(crate) fn render(ctx: DemoContext) -> Element {
    let on_toggle = ctx.on_toggle_state.clone();
    let on_close_cancel = ctx.on_toggle_state.clone();
    let on_close_continue = ctx.on_toggle_state.clone();
    let on_modal_toggle = ctx.on_toggle_state.clone();
    let dialog_open = ctx.toggle_state;

    arkit::stack_component()
        .percent_width(1.0)
        .percent_height(1.0)
        .children(vec![
            component_canvas(
                shadcn::button("Show Alert Dialog", shadcn::ButtonVariant::Outline)
                    .on_click(move || on_toggle(!dialog_open))
                    .into(),
                true,
                24.0,
            ),
            shadcn::alert_dialog_modal(
                ctx.toggle_state,
                move |value| on_modal_toggle(value),
                "Are you absolutely sure?",
                "This action cannot be undone. This will permanently delete your account and remove your data from our servers.",
                vec![
                    shadcn::button("Cancel", shadcn::ButtonVariant::Outline)
                        .percent_width(1.0)
                        .on_click(move || on_close_cancel(false))
                        .into(),
                    shadcn::button("Continue", shadcn::ButtonVariant::Default)
                        .percent_width(1.0)
                        .on_click(move || on_close_continue(false))
                        .into(),
                ],
            ),
        ])
        .into()
}
