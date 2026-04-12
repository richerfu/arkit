use super::super::layout::component_canvas;
use super::shared::DemoContext;
use crate::prelude::*;
use crate::Message;
use arkit_shadcn as shadcn;

pub(crate) fn render(ctx: DemoContext) -> Element {
    arkit::stack_component()
        .percent_width(1.0)
        .percent_height(1.0)
        .children(vec![
            component_canvas(
                shadcn::button("Show Alert Dialog")
                    .theme(shadcn::ButtonVariant::Outline)
                    .on_press(Message::SetToggleState(!ctx.toggle_state))
                    .into(),
                true,
                24.0,
            ),
            shadcn::alert_dialog_modal(
                ctx.toggle_state,
                Message::SetToggleState,
                "Are you absolutely sure?",
                "This action cannot be undone. This will permanently delete your account and remove your data from our servers.",
                vec![
                    shadcn::button("Cancel")
                        .theme(shadcn::ButtonVariant::Outline)
                        .percent_width(1.0)
                        .on_press(Message::SetToggleState(false))
                        .into(),
                    shadcn::button("Continue")
                        .theme(shadcn::ButtonVariant::Default)
                        .percent_width(1.0)
                        .on_press(Message::SetToggleState(false))
                        .into(),
                ],
            ),
        ])
        .into()
}
