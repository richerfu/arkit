use super::*;

pub fn popover(trigger: Element, content: Vec<Element>, open: Signal<bool>) -> Element {
    if !open.get() {
        return trigger;
    }

    arkit::column_component()
        .percent_width(1.0)
        .style(ArkUINodeAttributeType::ColumnAlignItems, FLEX_ALIGN_CENTER)
        .children(vec![
            panel_surface(
                arkit::column_component()
                    .width(320.0)
                    .style(
                        ArkUINodeAttributeType::Padding,
                        vec![spacing::LG, spacing::LG, spacing::LG, spacing::LG],
                    )
                    .children(vec![stack(content, spacing::LG)]),
            )
            .into(),
            arkit::row_component()
                .style(
                    ArkUINodeAttributeType::Margin,
                    vec![spacing::XXS, 0.0, 0.0, 0.0],
                )
                .children(vec![trigger])
                .into(),
        ])
        .into()
}

pub fn popover_card(title: impl Into<String>, body: impl Into<String>) -> Element {
    card(vec![title_text(title).into(), muted_text(body).into()])
}
