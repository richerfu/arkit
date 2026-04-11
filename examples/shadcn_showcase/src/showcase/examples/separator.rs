use super::super::layout::{component_canvas, fixed_width, h_stack, v_stack};
use super::shared::DemoContext;
use crate::prelude::*;
use arkit_shadcn as shadcn;

pub(crate) fn render(_ctx: DemoContext) -> Element {
    component_canvas(
        fixed_width(
            v_stack(
                vec![
                    v_stack(
                        vec![
                            shadcn::text_sm_medium("Radix Primitives"),
                            shadcn::text_with_variant(
                                "An open-source UI component library.",
                                shadcn::TextVariant::Muted,
                            ),
                        ],
                        4.0,
                    ),
                    shadcn::separator(),
                    h_stack(
                        vec![
                            shadcn::text_sm("Blog"),
                            shadcn::separator_vertical(20.0),
                            shadcn::text_sm("Docs"),
                            shadcn::separator_vertical(20.0),
                            shadcn::text_sm("Source"),
                        ],
                        16.0,
                    ),
                ],
                16.0,
            ),
            320.0,
        ),
        true,
        24.0,
    )
}
