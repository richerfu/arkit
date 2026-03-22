use super::*;

pub fn collapsible(title: impl Into<String>, open: Signal<bool>, content: Vec<Element>) -> Element {
    let click = open.clone();
    let mut items = content.into_iter();
    let first = items.next();
    let rest = items.collect::<Vec<_>>();

    let mut children = vec![
        arkit::row_component()
            .percent_width(1.0)
            .style(ArkUINodeAttributeType::RowAlignItems, FLEX_ALIGN_CENTER)
            .style(
                ArkUINodeAttributeType::RowJustifyContent,
                FLEX_ALIGN_SPACE_BETWEEN,
            )
            .style(ArkUINodeAttributeType::Padding, vec![0.0, spacing::LG, 0.0, spacing::LG])
            .children(vec![
                body_text(title)
                    .style(ArkUINodeAttributeType::FontWeight, 5_i32)
                    .style(ArkUINodeAttributeType::FontColor, color::FOREGROUND)
                    .into(),
                icon_button_with_variant("chevrons-up-down", ButtonVariant::Ghost)
                    .width(32.0)
                    .height(32.0)
                    .style(ArkUINodeAttributeType::Padding, vec![0.0, 0.0, 0.0, 0.0])
                    .on_click(move || click.update(|value| *value = !*value))
                    .into(),
            ])
            .into(),
    ];

    if let Some(first) = first {
        children.push(
            arkit::row_component()
                .style(ArkUINodeAttributeType::Margin, vec![spacing::SM, 0.0, 0.0, 0.0])
                .children(vec![first])
                .into(),
        );
    }

    if open.get() {
        children.extend(rest.into_iter().map(|child| {
            arkit::row_component()
                .style(ArkUINodeAttributeType::Margin, vec![spacing::SM, 0.0, 0.0, 0.0])
                .children(vec![child])
                .into()
        }));
    }

    arkit::column_component()
        .percent_width(1.0)
        .children(children)
        .into()
}
