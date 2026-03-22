use super::super::layout::fixed_width;
use super::shared::{top_center_canvas, DemoContext};
use arkit::prelude::*;
use arkit_shadcn as shadcn;

pub(crate) fn render(ctx: DemoContext) -> Element {
    top_center_canvas(
        fixed_width(
            shadcn::tabs(
                vec![String::from("Feedback"), String::from("Survey")],
                ctx.active_tab,
                vec![
                    shadcn::card(vec![
                        shadcn::card_header(
                            "Feedback",
                            "Share your thoughts with us. Click submit when you’re ready.",
                        ),
                        shadcn::card_content(vec![super::super::layout::v_stack(
                            vec![
                                super::super::layout::v_stack(
                                    vec![
                                        shadcn::label("Name").into(),
                                        shadcn::input("Michael Scott")
                                            .value("Michael Scott")
                                            .percent_width(1.0)
                                            .into(),
                                    ],
                                    12.0,
                                ),
                                super::super::layout::v_stack(
                                    vec![
                                        shadcn::label("Message").into(),
                                        shadcn::input("Where are the turtles?!")
                                            .value("Where are the turtles?!")
                                            .percent_width(1.0)
                                            .into(),
                                    ],
                                    12.0,
                                ),
                            ],
                            24.0,
                        )]),
                        shadcn::card_footer(vec![shadcn::button(
                            "Submit feedback",
                            shadcn::ButtonVariant::Default,
                        )
                        .into()]),
                    ]),
                    shadcn::card(vec![
                        shadcn::card_header(
                            "Quick Survey",
                            "Answer a few quick questions to help improve the demo.",
                        ),
                        shadcn::card_content(vec![super::super::layout::v_stack(
                            vec![
                                super::super::layout::v_stack(
                                    vec![
                                        shadcn::label("Job Title").into(),
                                        shadcn::input("Regional Manager")
                                            .value("Regional Manager")
                                            .percent_width(1.0)
                                            .into(),
                                    ],
                                    12.0,
                                ),
                                super::super::layout::v_stack(
                                    vec![
                                        shadcn::label("Favorite feature").into(),
                                        shadcn::input("CLI")
                                            .value("CLI")
                                            .percent_width(1.0)
                                            .into(),
                                    ],
                                    12.0,
                                ),
                            ],
                            24.0,
                        )]),
                        shadcn::card_footer(vec![shadcn::button(
                            "Submit survey",
                            shadcn::ButtonVariant::Default,
                        )
                        .into()]),
                    ]),
                ],
            ),
            384.0,
        ),
        [24.0, 24.0, 24.0, 24.0],
        true,
    )
}
