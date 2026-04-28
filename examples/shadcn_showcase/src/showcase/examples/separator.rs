use super::super::layout::{component_canvas, fixed_width, h_stack, v_stack};
use super::shared::DemoContext;
use crate::prelude::*;
use arkit_shadcn as shadcn;

pub(crate) struct SeparatorExample {
    ctx: DemoContext,
}

impl SeparatorExample {
    pub(crate) fn new(ctx: DemoContext) -> Self {
        Self { ctx }
    }
}

impl arkit::advanced::Widget<crate::Message, arkit::Theme, arkit::Renderer> for SeparatorExample {
    fn body(
        &self,
        _tree: &mut arkit::advanced::widget::Tree,
        _renderer: &arkit::Renderer,
    ) -> Option<Element> {
        let _ctx = self.ctx.clone();
        Some({
            component_canvas(
                fixed_width(
                    v_stack(
                        vec![
                            v_stack(
                                vec![
                                    shadcn::Text::small_medium("Radix Primitives").into(),
                                    shadcn::Text::with_variant(
                                        "An open-source UI component library.",
                                        shadcn::TextVariant::Muted,
                                    )
                                    .into(),
                                ],
                                4.0,
                            ),
                            shadcn::Separator::new().into(),
                            h_stack(
                                vec![
                                    shadcn::Text::small("Blog").into(),
                                    shadcn::Separator::vertical(20.0).into(),
                                    shadcn::Text::small("Docs").into(),
                                    shadcn::Separator::vertical(20.0).into(),
                                    shadcn::Text::small("Source").into(),
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
        })
    }
}

// struct component render
