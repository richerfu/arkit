use super::super::layout::{component_canvas, fixed_width, h_stack, v_stack};
use super::shared::DemoContext;
use crate::prelude::*;
use arkit_shadcn as shadcn;

pub(crate) struct SkeletonExample {
    ctx: DemoContext,
}

impl SkeletonExample {
    pub(crate) fn new(ctx: DemoContext) -> Self {
        Self { ctx }
    }
}

impl arkit::advanced::Widget<crate::Message, arkit::Theme, arkit::Renderer> for SkeletonExample {
    fn body(
        &self,
        _tree: &mut arkit::advanced::widget::Tree,
        _renderer: &arkit::Renderer,
    ) -> Option<Element> {
        let _ctx = self.ctx.clone();
        Some({
            component_canvas(
                fixed_width(
                    h_stack(
                        vec![
                            shadcn::Skeleton::new(48.0, 48.0).into(),
                            v_stack(
                                vec![
                                    shadcn::Skeleton::new(250.0, 16.0).into(),
                                    shadcn::Skeleton::new(200.0, 16.0).into(),
                                ],
                                shadcn::theme::spacing::SM,
                            ),
                        ],
                        shadcn::theme::spacing::LG,
                    ),
                    320.0,
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
