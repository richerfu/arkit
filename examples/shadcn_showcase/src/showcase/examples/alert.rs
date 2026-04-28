use super::super::layout::{max_width, v_stack};
use super::shared::{top_start_canvas, DemoContext};
use crate::prelude::*;
use arkit_shadcn as shadcn;

pub(crate) struct AlertExample {
    ctx: DemoContext,
}

impl AlertExample {
    pub(crate) fn new(ctx: DemoContext) -> Self {
        Self { ctx }
    }
}

impl arkit::advanced::Widget<crate::Message, arkit::Theme, arkit::Renderer> for AlertExample {
    fn body(
        &self,
        _tree: &mut arkit::advanced::widget::Tree,
        _renderer: &arkit::Renderer,
    ) -> Option<Element> {
        let _ctx = self.ctx.clone();
        Some({
            top_start_canvas(
                max_width(
                    v_stack(
                        vec![
                            shadcn::Alert::new(
                                "circle-check",
                                shadcn::AlertVariant::Default,
                                vec![
                                    shadcn::AlertTitle::new(
                                        "Success! Your changes have been saved",
                                        shadcn::AlertVariant::Default,
                                    )
                                    .into(),
                                    shadcn::AlertDescription::new(
                                        "This is an alert with icon, title and description.",
                                        shadcn::AlertVariant::Default,
                                    )
                                    .into(),
                                ],
                            )
                            .into(),
                            shadcn::Alert::new(
                                "terminal",
                                shadcn::AlertVariant::Default,
                                vec![shadcn::AlertTitle::new(
                                    "This Alert has no description.",
                                    shadcn::AlertVariant::Default,
                                )
                                .into()],
                            )
                            .into(),
                            shadcn::Alert::new(
                                "circle-alert",
                                shadcn::AlertVariant::Destructive,
                                vec![
                                    shadcn::AlertTitle::new(
                                        "Unable to process your payment.",
                                        shadcn::AlertVariant::Destructive,
                                    )
                                    .into(),
                                    shadcn::AlertDescription::new(
                                        "Please verify your billing information and try again.",
                                        shadcn::AlertVariant::Destructive,
                                    )
                                    .into(),
                                    shadcn::AlertList::new(
                                        vec![
                                            "Check your card details",
                                            "Ensure sufficient funds",
                                            "Verify billing address",
                                        ],
                                        shadcn::AlertVariant::Destructive,
                                    )
                                    .into(),
                                ],
                            )
                            .into(),
                        ],
                        shadcn::theme::spacing::LG,
                    ),
                    576.0,
                ),
                24.0,
            )
        })
    }
}

// struct component render
