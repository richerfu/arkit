use super::super::layout::{component_canvas, fixed_width, v_stack};
use super::shared::DemoContext;
use arkit::prelude::*;
use arkit_shadcn as shadcn;

fn form_row(label_text: &str, value: Element) -> Element {
    arkit::row_component()
        .percent_width(1.0)
        .align_items_center()
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
    let on_toggle = ctx.on_toggle_state.clone();
    let on_popover_toggle = ctx.on_toggle_state.clone();
    let is_open = ctx.toggle_state;
    component_canvas(
        fixed_width(
            shadcn::popover_with_width(
                shadcn::button("Open popover", shadcn::ButtonVariant::Outline)
                    .on_click(move || on_toggle(!is_open))
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
                                    .value("100%")
                                    .percent_width(1.0)
                                    .into(),
                            ),
                            form_row(
                                "Max. width",
                                shadcn::input("300px")
                                    .height(32.0)
                                    .value("300px")
                                    .percent_width(1.0)
                                    .into(),
                            ),
                            form_row(
                                "Height",
                                shadcn::input("25px")
                                    .height(32.0)
                                    .value("25px")
                                    .percent_width(1.0)
                                    .into(),
                            ),
                            form_row(
                                "Max. height",
                                shadcn::input("none")
                                    .height(32.0)
                                    .value("none")
                                    .percent_width(1.0)
                                    .into(),
                            ),
                        ],
                        8.0,
                    ),
                ],
                ctx.toggle_state,
                move |value| on_popover_toggle(value),
                320.0,
            ),
            384.0,
        ),
        true,
        24.0,
    )
}
