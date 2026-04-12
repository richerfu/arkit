use super::super::layout::component_canvas_with;
use super::shared::DemoContext;
use crate::prelude::*;
use arkit_shadcn as shadcn;

pub(crate) fn render(_ctx: DemoContext) -> Element {
    component_canvas_with(
        arkit::stack_component()
            .percent_width(1.0)
            .aspect_ratio(16.0 / 9.0)
            .children(vec![arkit::row_component()
                .percent_width(1.0)
                .percent_height(1.0)
                .clip(true)
                .border_radius([
                    shadcn::theme::radius::MD,
                    shadcn::theme::radius::MD,
                    shadcn::theme::radius::MD,
                    shadcn::theme::radius::MD,
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
}
