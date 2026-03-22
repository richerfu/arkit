use super::super::layout::{component_canvas, fixed_width, v_stack, FLEX_ALIGN_CENTER};
use super::shared::DemoContext;
use arkit::prelude::*;
use arkit_shadcn as shadcn;

fn form_row(label_text: &str, value: Element) -> Element {
    arkit::row_component()
        .percent_width(1.0)
        .style(ArkUINodeAttributeType::RowAlignItems, FLEX_ALIGN_CENTER)
        .children(vec![
            arkit::row_component()
                .width(96.0)
                .children(vec![shadcn::label(label_text).into()])
                .into(),
            arkit::row_component()
                .style(
                    ArkUINodeAttributeType::Margin,
                    vec![0.0, 0.0, 0.0, shadcn::theme::spacing::LG],
                )
                .style(ArkUINodeAttributeType::LayoutWeight, 1.0_f32)
                .children(vec![value])
                .into(),
        ])
        .into()
}

pub(crate) fn render(ctx: DemoContext) -> Element {
    let toggle = ctx.toggle_state.clone();
    component_canvas(
        fixed_width(
            shadcn::popover(
                shadcn::button("Open popover", shadcn::ButtonVariant::Outline)
                    .on_click(move || toggle.update(|open| *open = !*open))
                    .into(),
                vec![
                    v_stack(
                        vec![
                            arkit::text("Dimensions")
                                .font_size(shadcn::theme::typography::MD)
                                .style(ArkUINodeAttributeType::FontWeight, 4_i32)
                                .style(
                                    ArkUINodeAttributeType::FontColor,
                                    shadcn::theme::color::FOREGROUND,
                                )
                                .style(ArkUINodeAttributeType::TextLineHeight, 16.0)
                                .into(),
                            shadcn::text_with_variant(
                                "Set the dimensions for the layer.",
                                shadcn::TextVariant::Muted,
                            ),
                        ],
                        8.0,
                    ),
                    v_stack(
                        vec![
                            form_row(
                                "Width",
                                shadcn::input("100%")
                                    .height(32.0)
                                    .bind(ctx.query.clone())
                                    .percent_width(1.0)
                                    .into(),
                            ),
                            form_row(
                                "Max. width",
                                shadcn::input("300px")
                                    .height(32.0)
                                    .percent_width(1.0)
                                    .into(),
                            ),
                            form_row(
                                "Height",
                                shadcn::input("25px")
                                    .height(32.0)
                                    .percent_width(1.0)
                                    .into(),
                            ),
                            form_row(
                                "Max. height",
                                shadcn::input("none")
                                    .height(32.0)
                                    .percent_width(1.0)
                                    .into(),
                            ),
                        ],
                        8.0,
                    ),
                ],
                ctx.toggle_state,
            ),
            384.0,
        ),
        true,
        24.0,
    )
}
