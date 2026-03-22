use super::*;

pub fn alert_dialog(
    title: impl Into<String>,
    description: impl Into<String>,
    actions: Vec<Element>,
) -> Element {
    shadow_sm(
        arkit::column_component()
            .percent_width(1.0)
            .max_width_constraint(425.0)
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

pub fn alert_dialog_actions(actions: Vec<Element>) -> Element {
    super::dialog::dialog_footer(actions)
}
