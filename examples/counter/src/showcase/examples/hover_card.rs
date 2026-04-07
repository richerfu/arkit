use super::super::layout::{component_canvas, fixed_width, v_stack};
use super::shared::DemoContext;
use arkit::prelude::*;
use arkit_shadcn as shadcn;

pub(crate) fn render(ctx: DemoContext) -> Element {
    let on_toggle = ctx.on_toggle_state.clone();
    let on_card_toggle = ctx.on_toggle_state.clone();
    let is_open = ctx.toggle_state;
    component_canvas(
        fixed_width(
            shadcn::hover_card_with_width(
                shadcn::button("@expo", shadcn::ButtonVariant::Link)
                    .on_click(move || on_toggle(!is_open))
                    .into(),
                vec![arkit::row_component()
                    .percent_width(1.0)
                    .align_items_top()
                    .children(vec![
                        shadcn::avatar(Some(String::from("https://github.com/expo.png")), "E"),
                        arkit::row_component()
                            .style(ArkUINodeAttributeType::LayoutWeight, 1.0_f32)
                            .style(ArkUINodeAttributeType::Margin, vec![0.0, 0.0, 0.0, 16.0])
                            .children(vec![v_stack(
                                vec![
                                    arkit::text("@expo")
                                        .font_size(shadcn::theme::typography::SM)
                                        .style(ArkUINodeAttributeType::FontWeight, 5_i32)
                                        .style(
                                            ArkUINodeAttributeType::FontColor,
                                            shadcn::theme::color::FOREGROUND,
                                        )
                                        .style(ArkUINodeAttributeType::TextLineHeight, 20.0)
                                        .into(),
                                    arkit::text(
                                        "Framework and tools for creating native apps with React.",
                                    )
                                    .font_size(shadcn::theme::typography::SM)
                                    .style(
                                        ArkUINodeAttributeType::FontColor,
                                        shadcn::theme::color::FOREGROUND,
                                    )
                                    .style(ArkUINodeAttributeType::TextLineHeight, 20.0)
                                    .into(),
                                    arkit::text("Joined December 2021")
                                        .font_size(shadcn::theme::typography::XS)
                                        .style(
                                            ArkUINodeAttributeType::FontColor,
                                            shadcn::theme::color::MUTED_FOREGROUND,
                                        )
                                        .style(ArkUINodeAttributeType::TextLineHeight, 16.0)
                                        .into(),
                                ],
                                4.0,
                            )])
                            .into(),
                    ])
                    .into()],
                ctx.toggle_state,
                move |value| on_card_toggle(value),
                320.0,
            ),
            320.0,
        ),
        true,
        24.0,
    )
}
