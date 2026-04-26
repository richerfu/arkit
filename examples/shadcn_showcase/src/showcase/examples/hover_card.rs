use super::super::layout::{component_canvas, fixed_width, v_stack};
use super::shared::DemoContext;
use crate::prelude::*;
use crate::Message;
use arkit_shadcn as shadcn;

pub(crate) struct HoverCardExample {
    ctx: DemoContext,
}

impl HoverCardExample {
    pub(crate) fn new(ctx: DemoContext) -> Self {
        Self { ctx }
    }
}

impl arkit::advanced::Widget<crate::Message, arkit::Theme, arkit::Renderer> for HoverCardExample {
    fn body(
        &self,
        _tree: &mut arkit::advanced::widget::Tree,
        _renderer: &arkit::Renderer,
    ) -> Option<Element> {
        let ctx = self.ctx.clone();
        Some({
            component_canvas(
                fixed_width(
                    shadcn::HoverCard::new(
                        shadcn::Button::new("@expo")
                            .theme(shadcn::ButtonVariant::Link)
                            .on_press(Message::SetToggleState(!ctx.toggle_state))
                            .into(),
                        vec![arkit::row_component()
                            .percent_width(1.0)
                            .align_items_top()
                            .children(vec![
                                shadcn::Avatar::new(
                                    Some(String::from("https://github.com/expo.png")),
                                    "E",
                                )
                                .into(),
                                arkit::row_component()
                                    .layout_weight(1.0_f32)
                                    .margin([0.0, 0.0, 0.0, 16.0])
                                    .children(vec![v_stack(
                                        vec![
                                    arkit::text("@expo")
                                        .font_size(shadcn::theme::typography::SM)
                                        .font_weight(FontWeight::W600)
                                        .font_color(shadcn::theme::colors().foreground)
                                        .line_height(20.0)
                                        .into(),
                                    arkit::text(
                                        "Framework and tools for creating native apps with React.",
                                    )
                                    .font_size(shadcn::theme::typography::SM)
                                    .font_color(shadcn::theme::colors().foreground)
                                    .line_height(20.0)
                                    .into(),
                                    arkit::text("Joined December 2021")
                                        .font_size(shadcn::theme::typography::XS)
                                        .font_color(shadcn::theme::colors().muted_foreground)
                                        .line_height(16.0)
                                        .into(),
                                ],
                                        4.0,
                                    )])
                                    .into(),
                            ])
                            .into()],
                    )
                    .open(ctx.toggle_state)
                    .on_open_change(Message::SetToggleState)
                    .width(320.0)
                    .into(),
                    320.0,
                ),
                true,
                24.0,
            )
        })
    }
}

// struct component render
