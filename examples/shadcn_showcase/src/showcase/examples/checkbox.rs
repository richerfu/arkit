use super::super::layout::{component_canvas, fixed_width, h_stack, v_stack};
use super::shared::DemoContext;
use crate::prelude::*;
use crate::Message;
use arkit_shadcn as shadcn;

pub(crate) struct CheckboxExample {
    ctx: DemoContext,
}

impl CheckboxExample {
    pub(crate) fn new(ctx: DemoContext) -> Self {
        Self { ctx }
    }
}

impl arkit::advanced::Widget<crate::Message, arkit::Theme, arkit::Renderer> for CheckboxExample {
    fn body(
        &self,
        _tree: &mut arkit::advanced::widget::Tree,
        _renderer: &arkit::Renderer,
    ) -> Option<Element> {
        let ctx = self.ctx.clone();
        Some({
            let card_checked = ctx.checkbox_card;
            component_canvas(
        fixed_width(
            v_stack(
                vec![
                    h_stack(
                        vec![
                            shadcn::Checkbox::new("")
                                .checked(ctx.checkbox_first)
                                .on_change(Message::SetCheckboxFirst)
                                .into(),
                            shadcn::Label::new("Accept terms and conditions").into(),
                        ],
                        12.0,
                    ),
                    arkit::row_component()
                        .align_items_top()
                        .children(vec![
                            shadcn::Checkbox::new("")
                                .checked(ctx.checkbox_second)
                                .on_change(Message::SetCheckboxSecond)
                                .into(),
                            arkit::row_component()
                                .margin([0.0, 0.0, 0.0, 12.0])
                                .layout_weight(1.0_f32)
                                .children(vec![v_stack(
                                    vec![
                                        shadcn::Label::new("Accept terms and conditions").into(),
                                        shadcn::Text::muted(
                                            "By clicking this checkbox, you agree to the terms and conditions.",
                                        ).into(),
                                    ],
                                    8.0,
                                )])
                                .into(),
                        ])
                        .into(),
                    arkit::row_component()
                        .align_items_top()
                        .children(vec![
                            shadcn::Checkbox::new("").disabled(true).into(),
                            arkit::row_component()
                                .margin([0.0, 0.0, 0.0, 12.0])
                                .children(vec![arkit::text("Enable notifications")
                                    .font_size(shadcn::theme::typography::SM)
                                    .font_color(shadcn::theme::colors().foreground)
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
                                shadcn::Checkbox::new("")
                                    .checked(card_checked)
                                    .on_change(Message::SetCheckboxCard)
                                    .checked_color(0xFF2563EB)
                                    .into(),
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
                                            shadcn::Text::muted(
                                                "You can enable or disable notifications at any time.",
                                            ).into(),
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
