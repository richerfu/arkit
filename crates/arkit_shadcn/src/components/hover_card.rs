use super::*;

const HOVER_CARD_DEFAULT_WIDTH: f32 = 256.0; // Tailwind `w-64`

pub fn hover_card(trigger: Element, content: Vec<Element>, show: bool) -> Element {
    hover_card_with_width(trigger, content, show, HOVER_CARD_DEFAULT_WIDTH)
}

pub fn hover_card_with_width(
    trigger: Element,
    content: Vec<Element>,
    show: bool,
    width: f32,
) -> Element {
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
                            .width(width)
                            .style(
                                ArkUINodeAttributeType::Padding,
                                vec![spacing::LG, spacing::LG, spacing::LG, spacing::LG],
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
