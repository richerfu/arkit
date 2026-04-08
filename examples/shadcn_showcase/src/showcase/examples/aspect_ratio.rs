use super::super::layout::component_canvas_with;
use super::shared::DemoContext;
use crate::prelude::*;
use arkit_shadcn as shadcn;

pub(crate) fn render(_ctx: DemoContext) -> Element {
    component_canvas_with(
        arkit::stack_component()
            .percent_width(1.0)
            .style(ArkUINodeAttributeType::AspectRatio, 16.0 / 9.0)
            .children(vec![arkit::row_component()
                .percent_width(1.0)
                .percent_height(1.0)
                .style(ArkUINodeAttributeType::Clip, true)
                .style(
                    ArkUINodeAttributeType::BorderRadius,
                    vec![
                        shadcn::theme::radius::MD,
                        shadcn::theme::radius::MD,
                        shadcn::theme::radius::MD,
                        shadcn::theme::radius::MD,
                    ],
                )
                .children(vec![arkit::image_component()
                    .attr(
                        ArkUINodeAttributeType::ImageSrc,
                        "https://images.unsplash.com/photo-1672758247442-82df22f5899e",
                    )
                    .percent_width(1.0)
                    .percent_height(1.0)
                    .style(ArkUINodeAttributeType::ImageObjectFit, 1_i32)
                    .into()])
                .into()])
            .into(),
        false,
        true,
        true,
        [0.0, 24.0, 0.0, 24.0],
    )
}
