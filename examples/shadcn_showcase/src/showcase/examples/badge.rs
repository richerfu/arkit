use super::super::layout::{component_canvas, fixed_width, h_stack, v_stack};
use super::shared::DemoContext;
use crate::prelude::*;
use arkit_shadcn as shadcn;

pub(crate) struct BadgeExample {
    ctx: DemoContext,
}

impl BadgeExample {
    pub(crate) fn new(ctx: DemoContext) -> Self {
        Self { ctx }
    }
}

impl arkit::advanced::Widget<crate::Message, arkit::Theme, arkit::Renderer> for BadgeExample {
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
                            h_stack(
                                vec![
                                    shadcn::Badge::new("Badge").into(),
                                    shadcn::Badge::new("Secondary")
                                        .variant(shadcn::BadgeVariant::Secondary)
                                        .into(),
                                    shadcn::Badge::new("Destructive")
                                        .variant(shadcn::BadgeVariant::Destructive)
                                        .into(),
                                    shadcn::Badge::new("Outline")
                                        .variant(shadcn::BadgeVariant::Outline)
                                        .into(),
                                ],
                                shadcn::theme::spacing::SM,
                            ),
                            h_stack(
                                vec![
                                    shadcn::Badge::new("Verified")
                                        .icon_colors("badge-check", 0xFF3B82F6, 0xFFFFFFFF)
                                        .into(),
                                    shadcn::Badge::new("8")
                                        .variant(shadcn::BadgeVariant::Default)
                                        .pill(true)
                                        .into(),
                                    shadcn::Badge::new("99")
                                        .variant(shadcn::BadgeVariant::Destructive)
                                        .pill(true)
                                        .into(),
                                    shadcn::Badge::new("20+")
                                        .variant(shadcn::BadgeVariant::Outline)
                                        .pill(true)
                                        .into(),
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
