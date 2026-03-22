use super::super::layout::{component_canvas_with, fixed_width};
use super::shared::DemoContext;
use arkit::prelude::*;
use arkit_shadcn as shadcn;

pub(crate) fn render(_ctx: DemoContext) -> Element {
    component_canvas_with(
        fixed_width(
            shadcn::aspect_ratio(
                16.0 / 9.0,
                arkit::row_component()
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
                    .children(vec![
                        arkit::image_component()
                            .attr(
                                ArkUINodeAttributeType::ImageSrc,
                                "https://images.unsplash.com/photo-1672758247442-82df22f5899e",
                            )
                            .percent_width(1.0)
                            .percent_height(1.0)
                            // object-cover
                            .style(ArkUINodeAttributeType::ImageObjectFit, 1_i32)
                            .into(),
                    ])
                    .into(),
            ),
            320.0,
        ),
        true,
        true,
        true,
        [0.0, 24.0, 0.0, 24.0],
    )
}
