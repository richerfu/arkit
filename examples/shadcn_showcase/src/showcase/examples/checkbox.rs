use super::super::layout::{component_canvas, fixed_width, h_stack, v_stack};
use super::shared::DemoContext;
use crate::prelude::*;
use crate::Message;
use arkit_shadcn as shadcn;

pub(crate) fn render(ctx: DemoContext) -> Element {
    let card_checked = ctx.checkbox_card;
    component_canvas(
        fixed_width(
            v_stack(
                vec![
                    h_stack(
                        vec![
                            shadcn::checkbox(
                                "",
                                ctx.checkbox_first,
                                Message::SetCheckboxFirst,
                            ),
                            shadcn::label("Accept terms and conditions").into(),
                        ],
                        12.0,
                    ),
                    arkit::row_component()
                        .align_items_top()
                        .children(vec![
                            shadcn::checkbox(
                                "",
                                ctx.checkbox_second,
                                Message::SetCheckboxSecond,
                            ),
                            arkit::row_component()
                                .margin([0.0, 0.0, 0.0, 12.0])
                                .layout_weight(1.0_f32)
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
                                .margin([0.0, 0.0, 0.0, 12.0])
                                .children(vec![shadcn::label("Enable notifications")
                                    .opacity(0.5_f32)
                                    .into()])
                                .into(),
                        ])
                        .into(),
                    arkit::column_component()
                        .percent_width(1.0)
                        .padding([12.0, 12.0, 12.0, 12.0])
                        .border_width([1.0, 1.0, 1.0, 1.0])
                        .border_color(if ctx.checkbox_card {
                                0xFF2563EB
                            } else {
                                shadcn::theme::colors().border
                            })
                        .border_radius([
                                shadcn::theme::radii().lg,
                                shadcn::theme::radii().lg,
                                shadcn::theme::radii().lg,
                                shadcn::theme::radii().lg,
                            ])
                        .background_color(if ctx.checkbox_card {
                            0xFFEFF6FF
                        } else {
                            shadcn::theme::colors().background
                        })
                        .on_press(Message::SetCheckboxCard(!card_checked))
                        .children(vec![arkit::row_component()
                            .align_items_top()
                            .children(vec![
                                shadcn::checkbox_with_checked_color(
                                    "",
                                    card_checked,
                                    Message::SetCheckboxCard,
                                    0xFF2563EB,
                                ),
                                arkit::row_component()
                                    .margin([0.0, 0.0, 0.0, 12.0])
                                    .layout_weight(1.0_f32)
                                .children(vec![arkit::column_component()
                                        .percent_width(1.0)
                                        .align_items_start()
                                        .children(vec![
                                            arkit::text("Enable notifications")
                                                .font_size(shadcn::theme::typography::SM)
                                                .font_weight(FontWeight::W500,)
                                                .font_color(shadcn::theme::colors().foreground,)
                                                .line_height(14.0,)
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
