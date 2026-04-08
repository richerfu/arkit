use super::super::layout::{max_width, v_stack};
use super::shared::{top_start_canvas, DemoContext};
use crate::prelude::*;
use arkit_shadcn as shadcn;

pub(crate) fn render(_ctx: DemoContext) -> Element {
    top_start_canvas(
        max_width(
            v_stack(
                vec![
                    shadcn::alert_root(
                        "circle-check",
                        shadcn::AlertVariant::Default,
                        vec![
                            shadcn::alert_title(
                                "Success! Your changes have been saved",
                                shadcn::AlertVariant::Default,
                            )
                            .into(),
                            shadcn::alert_description(
                                "This is an alert with icon, title and description.",
                                shadcn::AlertVariant::Default,
                            )
                            .into(),
                        ],
                    ),
                    shadcn::alert_root(
                        "terminal",
                        shadcn::AlertVariant::Default,
                        vec![shadcn::alert_title(
                            "This Alert has no description.",
                            shadcn::AlertVariant::Default,
                        )
                        .into()],
                    ),
                    shadcn::alert_root(
                        "circle-alert",
                        shadcn::AlertVariant::Destructive,
                        vec![
                            shadcn::alert_title(
                                "Unable to process your payment.",
                                shadcn::AlertVariant::Destructive,
                            )
                            .into(),
                            shadcn::alert_description(
                                "Please verify your billing information and try again.",
                                shadcn::AlertVariant::Destructive,
                            )
                            .into(),
                            shadcn::alert_list(
                                vec![
                                    "Check your card details",
                                    "Ensure sufficient funds",
                                    "Verify billing address",
                                ],
                                shadcn::AlertVariant::Destructive,
                            ),
                        ],
                    ),
                ],
                shadcn::theme::spacing::LG,
            ),
            576.0,
        ),
        24.0,
    )
}
