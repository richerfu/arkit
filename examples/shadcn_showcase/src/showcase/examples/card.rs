use super::super::layout::{component_canvas, fixed_width, v_stack};
use super::shared::DemoContext;
use crate::prelude::*;
use arkit_shadcn as shadcn;

pub(crate) struct CardExample {
    ctx: DemoContext,
}

impl CardExample {
    pub(crate) fn new(ctx: DemoContext) -> Self {
        Self { ctx }
    }
}

impl arkit::advanced::Widget<crate::Message, arkit::Theme, arkit::Renderer> for CardExample {
    fn body(
        &self,
        _tree: &mut arkit::advanced::widget::Tree,
        _renderer: &arkit::Renderer,
    ) -> Option<Element> {
        let ctx = self.ctx.clone();
        Some({
            let _ = ctx;
            component_canvas(
                fixed_width(
                    shadcn::Card::new(vec![
                        shadcn::CardHeader::new(
                            "Subscribe to our newsletter",
                            "Enter your details to receive updates and tips",
                        )
                        .into(),
                        shadcn::CardContent::new(vec![v_stack(
                            vec![
                                v_stack(
                                    vec![
                                        shadcn::Label::new("Email").into(),
                                        shadcn::Input::new("m@example.com")
                                            .percent_width(1.0)
                                            .into(),
                                    ],
                                    8.0,
                                ),
                                v_stack(
                                    vec![
                                        shadcn::Label::new("Name").into(),
                                        shadcn::Input::new("John Doe").percent_width(1.0).into(),
                                    ],
                                    8.0,
                                ),
                            ],
                            16.0,
                        )])
                        .into(),
                        shadcn::CardFooter::new(vec![v_stack(
                            vec![
                                shadcn::Button::new("Subscribe")
                                    .theme(shadcn::ButtonVariant::Default)
                                    .percent_width(1.0)
                                    .into(),
                                shadcn::Button::new("Later")
                                    .theme(shadcn::ButtonVariant::Outline)
                                    .percent_width(1.0)
                                    .into(),
                            ],
                            8.0,
                        )])
                        .into(),
                    ])
                    .into(),
                    384.0,
                ),
                true,
                24.0,
            )
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn into_any(self: Box<Self>) -> Box<dyn std::any::Any> {
        self
    }
}

// struct component render
