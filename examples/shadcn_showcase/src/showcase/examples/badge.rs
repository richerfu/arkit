use super::super::layout::{component_canvas, fixed_width, h_stack, v_stack};
use super::shared::DemoContext;
use crate::prelude::*;
use arkit_shadcn as shadcn;

pub(crate) fn render(_ctx: DemoContext) -> Element {
    component_canvas(
        fixed_width(
            v_stack(
                vec![
                    h_stack(
                        vec![
                            shadcn::badge("Badge"),
                            shadcn::badge_with_variant(
                                "Secondary",
                                shadcn::BadgeVariant::Secondary,
                            ),
                            shadcn::badge_with_variant(
                                "Destructive",
                                shadcn::BadgeVariant::Destructive,
                            ),
                            shadcn::badge_with_variant("Outline", shadcn::BadgeVariant::Outline),
                        ],
                        shadcn::theme::spacing::SM,
                    ),
                    h_stack(
                        vec![
                            shadcn::badge_with_icon_colors(
                                "Verified",
                                "badge-check",
                                0xFF3B82F6,
                                0xFFFFFFFF,
                            ),
                            shadcn::pill_badge_with_variant("8", shadcn::BadgeVariant::Default),
                            shadcn::pill_badge_with_variant(
                                "99",
                                shadcn::BadgeVariant::Destructive,
                            ),
                            shadcn::pill_badge_with_variant("20+", shadcn::BadgeVariant::Outline),
                        ],
                        shadcn::theme::spacing::SM,
                    ),
                ],
                shadcn::theme::spacing::SM,
            ),
            384.0,
        ),
        true,
        24.0,
    )
}
