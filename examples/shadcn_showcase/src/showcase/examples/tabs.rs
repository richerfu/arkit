use super::shared::{top_start_canvas, DemoContext};
use crate::prelude::*;
use arkit_shadcn as shadcn;

pub(crate) struct TabsExample {
    ctx: DemoContext,
}

impl TabsExample {
    pub(crate) fn new(ctx: DemoContext) -> Self {
        Self { ctx }
    }
}

impl arkit::advanced::Widget<crate::Message, arkit::Theme, arkit::Renderer> for TabsExample {
    fn body(
        &self,
        _tree: &mut arkit::advanced::widget::Tree,
        _renderer: &arkit::Renderer,
    ) -> Option<Element> {
        let _ctx = self.ctx.clone();
        Some({
            top_start_canvas(
                arkit::row_component()
                    .percent_width(1.0)
                    .max_width_constraint(384.0)
                    .children(vec![shadcn::Tabs::new(
                        vec![String::from("Feedback"), String::from("Survey")],
                        vec![
                            shadcn::Card::new(vec![
                                shadcn::CardHeader::new(
                                    "Feedback",
                                    "Share your thoughts with us. Click submit when you’re ready.",
                                )
                                .into(),
                                shadcn::CardContent::new(vec![super::super::layout::v_stack(
                                    vec![
                                        super::super::layout::v_stack(
                                            vec![
                                                shadcn::Label::new("Name").into(),
                                                shadcn::Input::new("Michael Scott")
                                                    .value("Michael Scott")
                                                    .percent_width(1.0)
                                                    .into(),
                                            ],
                                            12.0,
                                        ),
                                        super::super::layout::v_stack(
                                            vec![
                                                shadcn::Label::new("Message").into(),
                                                shadcn::Input::new("Where are the turtles?!")
                                                    .value("Where are the turtles?!")
                                                    .percent_width(1.0)
                                                    .into(),
                                            ],
                                            12.0,
                                        ),
                                    ],
                                    24.0,
                                )])
                                .into(),
                                shadcn::CardFooter::new(vec![shadcn::Button::new(
                                    "Submit feedback",
                                )
                                .theme(shadcn::ButtonVariant::Default)
                                .into()])
                                .into(),
                            ])
                            .into(),
                            shadcn::Card::new(vec![
                                shadcn::CardHeader::new(
                                    "Quick Survey",
                                    "Answer a few quick questions to help improve the demo.",
                                )
                                .into(),
                                shadcn::CardContent::new(vec![super::super::layout::v_stack(
                                    vec![
                                        super::super::layout::v_stack(
                                            vec![
                                                shadcn::Label::new("Job Title").into(),
                                                shadcn::Input::new("Regional Manager")
                                                    .value("Regional Manager")
                                                    .percent_width(1.0)
                                                    .into(),
                                            ],
                                            12.0,
                                        ),
                                        super::super::layout::v_stack(
                                            vec![
                                                shadcn::Label::new("Favorite feature").into(),
                                                shadcn::Input::new("CLI")
                                                    .value("CLI")
                                                    .percent_width(1.0)
                                                    .into(),
                                            ],
                                            12.0,
                                        ),
                                    ],
                                    24.0,
                                )])
                                .into(),
                                shadcn::CardFooter::new(vec![shadcn::Button::new("Submit survey")
                                    .theme(shadcn::ButtonVariant::Default)
                                    .into()])
                                .into(),
                            ])
                            .into(),
                        ],
                    )
                    .default_active(0)
                    .into()])
                    .into(),
                24.0,
            )
        })
    }
}

// struct component render
