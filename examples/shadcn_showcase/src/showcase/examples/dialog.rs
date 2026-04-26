use super::super::layout::{component_canvas, v_stack};
use super::shared::DemoContext;
use crate::prelude::*;
use crate::Message;
use arkit_shadcn as shadcn;

pub(crate) struct DialogExample {
    ctx: DemoContext,
}

impl DialogExample {
    pub(crate) fn new(ctx: DemoContext) -> Self {
        Self { ctx }
    }
}

impl arkit::advanced::Widget<crate::Message, arkit::Theme, arkit::Renderer> for DialogExample {
    fn body(
        &self,
        _tree: &mut arkit::advanced::widget::Tree,
        _renderer: &arkit::Renderer,
    ) -> Option<Element> {
        let ctx = self.ctx.clone();
        Some({
            arkit::stack_component()
                .percent_width(1.0)
                .percent_height(1.0)
                .children(vec![
                    component_canvas(
                        shadcn::Button::new("Open Dialog")
                            .theme(shadcn::ButtonVariant::Outline)
                            .on_press(Message::SetToggleState(!ctx.toggle_state))
                            .into(),
                        true,
                        24.0,
                    ),
                    shadcn::Dialog::new(
                        "Edit profile",
                        vec![
                            shadcn::DialogHeader::new(
                                "Edit profile",
                                "Make changes to your profile here. Click save when you’re done.",
                            )
                            .into(),
                            v_stack(
                                vec![
                                    v_stack(
                                        vec![
                                            shadcn::Label::new("Name").into(),
                                            shadcn::Input::new("Pedro Duarte")
                                                .value("Pedro Duarte")
                                                .percent_width(1.0)
                                                .into(),
                                        ],
                                        12.0,
                                    ),
                                    v_stack(
                                        vec![
                                            shadcn::Label::new("Username").into(),
                                            shadcn::Input::new("@peduarte")
                                                .value("@peduarte")
                                                .percent_width(1.0)
                                                .into(),
                                        ],
                                        12.0,
                                    ),
                                ],
                                16.0,
                            ),
                            shadcn::DialogFooter::new(vec![
                                shadcn::Button::new("Cancel")
                                    .theme(shadcn::ButtonVariant::Outline)
                                    .percent_width(1.0)
                                    .on_press(Message::SetToggleState(false))
                                    .into(),
                                shadcn::Button::new("Save changes")
                                    .theme(shadcn::ButtonVariant::Default)
                                    .percent_width(1.0)
                                    .into(),
                            ])
                            .into(),
                        ],
                    )
                    .open(ctx.toggle_state)
                    .on_open_change(Message::SetToggleState)
                    .into(),
                ])
                .into()
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
