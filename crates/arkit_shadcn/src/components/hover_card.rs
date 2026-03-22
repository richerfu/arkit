use super::*;

pub fn hover_card(trigger: Element, content: Vec<Element>, show: bool) -> Element {
    if show {
        arkit::column_component()
            .percent_width(1.0)
            .children(vec![
                trigger,
                arkit::row_component()
                    .style(
                        ArkUINodeAttributeType::Margin,
                        vec![spacing::XXS, 0.0, 0.0, 0.0],
                    )
                    .children(vec![panel_surface(
                        arkit::column_component()
                            .width(320.0)
                            .style(
                                ArkUINodeAttributeType::Padding,
                                vec![spacing::MD, spacing::MD, spacing::MD, spacing::MD],
                            )
                            .children(vec![stack(content, spacing::MD)]),
                    )
                    .into()])
                    .into(),
            ])
            .into()
    } else {
        trigger
    }
}
