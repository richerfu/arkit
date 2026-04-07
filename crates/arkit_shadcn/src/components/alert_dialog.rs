use super::*;
use std::rc::Rc;

pub fn alert_dialog(
    title: impl Into<String>,
    description: impl Into<String>,
    actions: Vec<Element>,
) -> Element {
    shadow_sm(
        arkit::column_component()
            .percent_width(1.0)
            .max_width_constraint(super::dialog::DIALOG_MAX_WIDTH)
            .style(
                ArkUINodeAttributeType::Padding,
                vec![spacing::XXL, spacing::XXL, spacing::XXL, spacing::XXL],
            )
            .style(
                ArkUINodeAttributeType::BorderRadius,
                vec![radius::LG, radius::LG, radius::LG, radius::LG],
            )
            .style(
                ArkUINodeAttributeType::BorderWidth,
                vec![1.0, 1.0, 1.0, 1.0],
            )
            .style(ArkUINodeAttributeType::BorderColor, vec![color::BORDER])
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

pub fn alert_dialog_modal(
    open: bool,
    on_open_change: impl Fn(bool) + 'static,
    title: impl Into<String>,
    description: impl Into<String>,
    actions: Vec<Element>,
) -> Element {
    let dismiss = Rc::new(move || on_open_change(false));
    super::dialog::modal_overlay(
        open,
        alert_dialog(title, description, actions),
        Some(dismiss),
    )
}

pub fn alert_dialog_actions(actions: Vec<Element>) -> Element {
    super::dialog::dialog_footer(actions)
}
