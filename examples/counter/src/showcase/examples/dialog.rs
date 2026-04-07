use super::super::layout::{component_canvas, v_stack};
use super::shared::DemoContext;
use arkit::prelude::*;
use arkit_shadcn as shadcn;

pub(crate) fn render(ctx: DemoContext) -> Element {
    let on_toggle = ctx.on_toggle_state.clone();
    let on_cancel = ctx.on_toggle_state.clone();
    let on_dialog_toggle = ctx.on_toggle_state.clone();
    let dialog_open = ctx.toggle_state;

    arkit::stack_component()
        .percent_width(1.0)
        .percent_height(1.0)
        .children(vec![
            component_canvas(
                shadcn::button("Open Dialog", shadcn::ButtonVariant::Outline)
                    .on_click(move || on_toggle(!dialog_open))
                    .into(),
                true,
                24.0,
            ),
            shadcn::dialog(
                "Edit profile",
                ctx.toggle_state,
                move |value| on_dialog_toggle(value),
                vec![
                    shadcn::dialog_header(
                        "Edit profile",
                        "Make changes to your profile here. Click save when you’re done.",
                    ),
                    v_stack(
                        vec![
                            v_stack(
                                vec![
                                    shadcn::label("Name").into(),
                                    shadcn::input("Pedro Duarte")
                                        .value("Pedro Duarte")
                                        .percent_width(1.0)
                                        .into(),
                                ],
                                12.0,
                            ),
                            v_stack(
                                vec![
                                    shadcn::label("Username").into(),
                                    shadcn::input("@peduarte")
                                        .value("@peduarte")
                                        .percent_width(1.0)
                                        .into(),
                                ],
                                12.0,
                            ),
                        ],
                        16.0,
                    ),
                    shadcn::dialog_footer(vec![
                        shadcn::button("Cancel", shadcn::ButtonVariant::Outline)
                            .percent_width(1.0)
                            .on_click(move || on_cancel(false))
                            .into(),
                        shadcn::button("Save changes", shadcn::ButtonVariant::Default)
                            .percent_width(1.0)
                            .into(),
                    ]),
                ],
            ),
        ])
        .into()
}
