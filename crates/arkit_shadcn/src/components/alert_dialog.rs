use super::*;
use std::rc::Rc;

pub fn alert_dialog(
    title: impl Into<String>,
    description: impl Into<String>,
    actions: Vec<Element>,
) -> Element {
    alert_dialog_with_message::<()>(title, description, actions)
}

pub fn alert_dialog_with_message<Message: 'static>(
    title: impl Into<String>,
    description: impl Into<String>,
    actions: Vec<Element<Message>>,
) -> Element<Message> {
    shadow_sm(
        arkit::column_component::<Message, arkit::Theme>()
            .percent_width(1.0)
            .max_width_constraint(super::dialog::DIALOG_MAX_WIDTH)
            .padding([spacing::XXL, spacing::XXL, spacing::XXL, spacing::XXL])
            .border_radius([radius::LG, radius::LG, radius::LG, radius::LG])
            .border_width([1.0, 1.0, 1.0, 1.0])
            .border_color(color::BORDER)
            .background_color(color::BACKGROUND)
            .children(vec![stack(
                vec![
                    super::dialog::dialog_header(title, description),
                    super::dialog::dialog_footer(actions),
                ],
                spacing::LG,
            )]),
    )
    .into()
}

pub fn alert_dialog_modal_message<Message>(
    open: bool,
    on_open_change: impl Fn(bool) -> Message + 'static,
    title: impl Into<String>,
    description: impl Into<String>,
    actions: Vec<Element<Message>>,
) -> Element<Message>
where
    Message: Send + 'static,
{
    super::dialog::modal_overlay(
        open,
        alert_dialog_with_message(title, description, actions),
        Some(Rc::new(move || dispatch_message(on_open_change(false)))),
    )
}

pub fn alert_dialog_actions<Message: 'static>(actions: Vec<Element<Message>>) -> Element<Message> {
    super::dialog::dialog_footer(actions)
}
