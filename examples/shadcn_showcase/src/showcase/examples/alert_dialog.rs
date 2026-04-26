use super::super::layout::component_canvas;
use super::shared::DemoContext;
use crate::prelude::*;
use crate::Message;
use arkit_shadcn as shadcn;

pub(crate) struct AlertDialogExample {
    ctx: DemoContext,
}

impl AlertDialogExample {
    pub(crate) fn new(ctx: DemoContext) -> Self {
        Self { ctx }
    }
}

impl arkit::advanced::Widget<crate::Message, arkit::Theme, arkit::Renderer> for AlertDialogExample {
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
                shadcn::Button::new("Show Alert Dialog")
                    .theme(shadcn::ButtonVariant::Outline)
                    .on_press(Message::SetToggleState(!ctx.toggle_state))
                    .into(),
                true,
                24.0,
            ),
            shadcn::AlertDialog::new(
                "Are you absolutely sure?",
                "This action cannot be undone. This will permanently delete your account and remove your data from our servers.",
                vec![
                    shadcn::Button::new("Cancel")
                        .theme(shadcn::ButtonVariant::Outline)
                        .percent_width(1.0)
                        .on_press(Message::SetToggleState(false))
                        .into(),
                    shadcn::Button::new("Continue")
                        .theme(shadcn::ButtonVariant::Default)
                        .percent_width(1.0)
                        .on_press(Message::SetToggleState(false))
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
