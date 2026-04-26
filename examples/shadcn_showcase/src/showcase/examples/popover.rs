use super::super::layout::{component_canvas, fixed_width, v_stack};
use super::shared::DemoContext;
use crate::prelude::*;
use crate::Message;
use arkit_shadcn as shadcn;

fn form_row(label_text: &str, value: Element) -> Element {
    arkit::row_component()
        .percent_width(1.0)
        .align_items_center()
        .children(vec![
            arkit::row_component()
                .width(96.0)
                .children(vec![shadcn::Label::new(label_text).into()])
                .into(),
            arkit::row_component()
                .margin([0.0, 0.0, 0.0, shadcn::theme::spacing::LG])
                .layout_weight(1.0_f32)
                .children(vec![value])
                .into(),
        ])
        .into()
}

pub(crate) struct PopoverExample {
    ctx: DemoContext,
}

impl PopoverExample {
    pub(crate) fn new(ctx: DemoContext) -> Self {
        Self { ctx }
    }
}

impl arkit::advanced::Widget<crate::Message, arkit::Theme, arkit::Renderer> for PopoverExample {
    fn body(
        &self,
        _tree: &mut arkit::advanced::widget::Tree,
        _renderer: &arkit::Renderer,
    ) -> Option<Element> {
        let ctx = self.ctx.clone();
        Some({
            component_canvas(
                fixed_width(
                    shadcn::Popover::new(
                        shadcn::Button::new("Open popover")
                            .theme(shadcn::ButtonVariant::Outline)
                            .on_press(Message::SetPopoverOpen(!ctx.popover_open))
                            .into(),
                        vec![
                            v_stack(
                                vec![
                                    arkit::text("Dimensions")
                                        .font_size(shadcn::theme::typography::MD)
                                        .font_weight(FontWeight::W500)
                                        .font_color(shadcn::theme::colors().foreground)
                                        .line_height(16.0)
                                        .into(),
                                    arkit::text("Set the dimensions for the layer.")
                                        .font_size(shadcn::theme::typography::SM)
                                        .font_color(shadcn::theme::colors().muted_foreground)
                                        .line_height(20.0)
                                        .into(),
                                ],
                                8.0,
                            ),
                            v_stack(
                                vec![
                                    form_row(
                                        "Width",
                                        shadcn::Input::new("100%")
                                            .height(32.0)
                                            .value("100%")
                                            .percent_width(1.0)
                                            .into(),
                                    ),
                                    form_row(
                                        "Max. width",
                                        shadcn::Input::new("300px")
                                            .height(32.0)
                                            .value("300px")
                                            .percent_width(1.0)
                                            .into(),
                                    ),
                                    form_row(
                                        "Height",
                                        shadcn::Input::new("25px")
                                            .height(32.0)
                                            .value("25px")
                                            .percent_width(1.0)
                                            .into(),
                                    ),
                                    form_row(
                                        "Max. height",
                                        shadcn::Input::new("none")
                                            .height(32.0)
                                            .value("none")
                                            .percent_width(1.0)
                                            .into(),
                                    ),
                                ],
                                8.0,
                            ),
                        ],
                    )
                    .open(ctx.popover_open)
                    .on_open_change(Message::SetPopoverOpen)
                    .width(320.0)
                    .into(),
                    384.0,
                ),
                true,
                24.0,
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
