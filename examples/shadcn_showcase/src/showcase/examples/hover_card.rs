use super::super::layout::{component_canvas, fixed_width, v_stack};
use super::shared::DemoContext;
use crate::prelude::*;
use crate::Message;
use arkit_shadcn as shadcn;

pub(crate) fn render(ctx: DemoContext) -> Element {
    component_canvas(
        fixed_width(
            shadcn::hover_card_with_width(
                shadcn::button("@expo")
                    .theme(shadcn::ButtonVariant::Link)
                    .on_press(Message::SetToggleState(!ctx.toggle_state))
                    .into(),
                vec![arkit::row_component()
                    .percent_width(1.0)
                    .align_items_top()
                    .children(vec![
                        shadcn::avatar(Some(String::from("https://github.com/expo.png")), "E"),
                        arkit::row_component()
                            .layout_weight(1.0_f32)
                            .margin([0.0, 0.0, 0.0, 16.0])
                            .children(vec![v_stack(
                                vec![
                                    arkit::text("@expo")
                                        .font_size(shadcn::theme::typography::SM)
                                        .font_weight(FontWeight::W600)
                                        .font_color(shadcn::theme::color::FOREGROUND)
                                        .line_height(20.0)
                                        .into(),
                                    arkit::text(
                                        "Framework and tools for creating native apps with React.",
                                    )
                                    .font_size(shadcn::theme::typography::SM)
                                    .font_color(shadcn::theme::color::FOREGROUND)
                                    .line_height(20.0)
                                    .into(),
                                    arkit::text("Joined December 2021")
                                        .font_size(shadcn::theme::typography::XS)
                                        .font_color(shadcn::theme::color::MUTED_FOREGROUND)
                                        .line_height(16.0)
                                        .into(),
                                ],
                                4.0,
                            )])
                            .into(),
                    ])
                    .into()],
                ctx.toggle_state,
                Message::SetToggleState,
                320.0,
            ),
            320.0,
        ),
        true,
        24.0,
    )
}
