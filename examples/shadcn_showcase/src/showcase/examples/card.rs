use super::super::layout::{component_canvas, fixed_width, v_stack};
use super::shared::DemoContext;
use crate::prelude::*;
use arkit_shadcn as shadcn;

pub(crate) fn render(ctx: DemoContext) -> Element {
    let _ = ctx;
    component_canvas(
        fixed_width(
            shadcn::card(vec![
                shadcn::card_header(
                    "Subscribe to our newsletter",
                    "Enter your details to receive updates and tips",
                ),
                shadcn::card_content(vec![v_stack(
                    vec![
                        v_stack(
                            vec![
                                shadcn::label("Email").into(),
                                shadcn::input("m@example.com").percent_width(1.0).into(),
                            ],
                            8.0,
                        ),
                        v_stack(
                            vec![
                                shadcn::label("Name").into(),
                                shadcn::input("John Doe").percent_width(1.0).into(),
                            ],
                            8.0,
                        ),
                    ],
                    16.0,
                )]),
                shadcn::card_footer(vec![v_stack(
                    vec![
                        shadcn::button("Subscribe")
                            .theme(shadcn::ButtonVariant::Default)
                            .percent_width(1.0)
                            .into(),
                        shadcn::button("Later")
                            .theme(shadcn::ButtonVariant::Outline)
                            .percent_width(1.0)
                            .into(),
                    ],
                    8.0,
                )]),
            ]),
            384.0,
        ),
        true,
        24.0,
    )
}
