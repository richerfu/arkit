use super::super::layout::{component_canvas, v_stack};
use super::shared::DemoContext;
use arkit::prelude::*;
use arkit_shadcn as shadcn;

pub(crate) fn render(ctx: DemoContext) -> Element {
    let toggle = ctx.toggle_state.clone();
    let cancel = ctx.toggle_state.clone();

    arkit::stack_component()
        .percent_width(1.0)
        .percent_height(1.0)
        .children(vec![
            component_canvas(
                shadcn::button("Open Dialog", shadcn::ButtonVariant::Outline)
                    .on_click(move || toggle.update(|open| *open = !*open))
                    .into(),
                true,
                24.0,
            ),
            shadcn::dialog(
                "Edit profile",
                ctx.toggle_state,
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
                                        .bind(ctx.query.clone())
                                        .percent_width(1.0)
                                        .into(),
                                ],
                                12.0,
                            ),
                            v_stack(
                                vec![
                                    shadcn::label("Username").into(),
                                    shadcn::input("@peduarte")
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
                            .on_click(move || cancel.set(false))
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
