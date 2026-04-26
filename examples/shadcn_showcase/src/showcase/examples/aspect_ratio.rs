use super::super::layout::component_canvas_with;
use super::shared::DemoContext;
use crate::prelude::*;
use arkit_shadcn as shadcn;

pub(crate) struct AspectRatioExample {
    ctx: DemoContext,
}

impl AspectRatioExample {
    pub(crate) fn new(ctx: DemoContext) -> Self {
        Self { ctx }
    }
}

impl arkit::advanced::Widget<crate::Message, arkit::Theme, arkit::Renderer> for AspectRatioExample {
    fn body(
        &self,
        _tree: &mut arkit::advanced::widget::Tree,
        _renderer: &arkit::Renderer,
    ) -> Option<Element> {
        let _ctx = self.ctx.clone();
        Some({
            component_canvas_with(
                arkit::stack_component()
                    .percent_width(1.0)
                    .aspect_ratio(16.0 / 9.0)
                    .children(vec![arkit::row_component()
                        .percent_width(1.0)
                        .percent_height(1.0)
                        .clip(true)
                        .border_radius([
                            shadcn::theme::radii().md,
                            shadcn::theme::radii().md,
                            shadcn::theme::radii().md,
                            shadcn::theme::radii().md,
                        ])
                        .children(vec![arkit::image(
                            "https://images.unsplash.com/photo-1672758247442-82df22f5899e",
                        )
                        .percent_width(1.0)
                        .percent_height(1.0)
                        .image_object_fit(ObjectFit::Cover)
                        .into()])
                        .into()])
                    .into(),
                false,
                true,
                true,
                [0.0, 24.0, 0.0, 24.0],
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
