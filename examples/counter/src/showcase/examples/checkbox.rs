use super::super::layout::{component_canvas, fixed_width, h_stack, v_stack};
use super::shared::DemoContext;
use arkit::prelude::*;
use arkit_shadcn as shadcn;

#[component]
pub(crate) fn render(ctx: DemoContext) -> Element {
    let on_first = ctx.on_checkbox_first.clone();
    let on_second = ctx.on_checkbox_second.clone();
    let on_card_click = ctx.on_checkbox_card.clone();
    let on_card_checkbox = ctx.on_checkbox_card.clone();
    let card_checked = ctx.checkbox_card;
    component_canvas(
        fixed_width(
            v_stack(
                vec![
                    h_stack(
                        vec![
                            shadcn::checkbox("", ctx.checkbox_first, move |value| on_first(value)),
                            shadcn::label("Accept terms and conditions").into(),
                        ],
                        12.0,
                    ),
                    arkit::row_component()
                        .align_items_top()
                        .children(vec![
                            shadcn::checkbox("", ctx.checkbox_second, move |value| on_second(value)),
                            arkit::row_component()
                                .style(
                                    ArkUINodeAttributeType::Margin,
                                    vec![0.0, 0.0, 0.0, 12.0],
                                )
                                .style(ArkUINodeAttributeType::LayoutWeight, 1.0_f32)
                                .children(vec![v_stack(
                                    vec![
                                        shadcn::label("Accept terms and conditions").into(),
                                        shadcn::text_variant(
                                            "By clicking this checkbox, you agree to the terms and conditions.",
                                            true,
                                        ),
                                    ],
                                    8.0,
                                )])
                                .into(),
                        ])
                        .into(),
                    arkit::row_component()
                        .align_items_top()
                        .children(vec![
                            shadcn::disabled_checkbox("", false),
                            arkit::row_component()
                                .style(
                                    ArkUINodeAttributeType::Margin,
                                    vec![0.0, 0.0, 0.0, 12.0],
                                )
                                .children(vec![shadcn::label("Enable notifications")
                                    .style(ArkUINodeAttributeType::Opacity, 0.5_f32)
                                    .into()])
                                .into(),
                        ])
                        .into(),
                    arkit::column_component()
                        .percent_width(1.0)
                        .style(ArkUINodeAttributeType::Padding, vec![12.0, 12.0, 12.0, 12.0])
                        .style(ArkUINodeAttributeType::BorderWidth, vec![1.0, 1.0, 1.0, 1.0])
                        .style(
                            ArkUINodeAttributeType::BorderColor,
                            vec![if ctx.checkbox_card {
                                0xFF2563EB
                            } else {
                                shadcn::theme::color::BORDER
                            }],
                        )
                        .style(
                            ArkUINodeAttributeType::BorderRadius,
                            vec![
                                shadcn::theme::radius::LG,
                                shadcn::theme::radius::LG,
                                shadcn::theme::radius::LG,
                                shadcn::theme::radius::LG,
                            ],
                        )
                        .background_color(if ctx.checkbox_card {
                            0xFFEFF6FF
                        } else {
                            shadcn::theme::color::BACKGROUND
                        })
                        .on_click(move || on_card_click(!card_checked))
                        .children(vec![arkit::row_component()
                            .align_items_top()
                            .children(vec![
                                shadcn::checkbox_with_checked_color(
                                    "",
                                    card_checked,
                                    move |value| on_card_checkbox(value),
                                    0xFF2563EB,
                                ),
                                arkit::row_component()
                                    .style(
                                        ArkUINodeAttributeType::Margin,
                                        vec![0.0, 0.0, 0.0, 12.0],
                                    )
                                    .style(ArkUINodeAttributeType::LayoutWeight, 1.0_f32)
                                .children(vec![arkit::column_component()
                                        .percent_width(1.0)
                                        .align_items_start()
                                        .children(vec![
                                            arkit::text("Enable notifications")
                                                .font_size(shadcn::theme::typography::SM)
                                                .style(
                                                    ArkUINodeAttributeType::FontWeight,
                                                    4_i32,
                                                )
                                                .style(
                                                    ArkUINodeAttributeType::FontColor,
                                                    shadcn::theme::color::FOREGROUND,
                                                )
                                                .style(
                                                    ArkUINodeAttributeType::TextLineHeight,
                                                    14.0,
                                                )
                                                .into(),
                                            shadcn::text_variant(
                                                "You can enable or disable notifications at any time.",
                                                true,
                                            ),
                                        ])
                                        .into()])
                                    .into(),
                            ])
                            .into()])
                        .into(),
                ],
                24.0,
            ),
            384.0,
        ),
        true,
        32.0,
    )
}
