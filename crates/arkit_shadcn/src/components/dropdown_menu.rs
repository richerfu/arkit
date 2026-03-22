use super::*;

fn menu_content(items: Vec<Element>) -> Element {
    shadow_sm(
        arkit::column_component()
            .width(224.0)
            .style(
                ArkUINodeAttributeType::Padding,
                vec![spacing::XXS, spacing::XXS, spacing::XXS, spacing::XXS],
            )
            .style(
                ArkUINodeAttributeType::BorderRadius,
                vec![radius::MD, radius::MD, radius::MD, radius::MD],
            )
            .style(
                ArkUINodeAttributeType::BorderWidth,
                vec![1.0, 1.0, 1.0, 1.0],
            )
            .style(ArkUINodeAttributeType::BorderColor, vec![color::BORDER])
            .style(ArkUINodeAttributeType::Clip, true)
            .background_color(color::POPOVER)
            .children(items),
    )
    .into()
}

pub fn dropdown_menu(trigger: Element, items: Vec<Element>, open: Signal<bool>) -> Element {
    if !open.get() {
        return trigger;
    }

    arkit::column_component()
        .percent_width(1.0)
        .children(vec![
            trigger,
            margin_top(
                arkit::row_component().children(vec![menu_content(items)]),
                spacing::XXS,
            )
            .into(),
        ])
        .into()
}

pub fn dropdown_item(title: impl Into<String>) -> Element {
    button(title, ButtonVariant::Ghost)
        .height(36.0)
        .style(ArkUINodeAttributeType::RowJustifyContent, FLEX_ALIGN_START)
        .style(
            ArkUINodeAttributeType::Padding,
            vec![8.0, spacing::SM, 8.0, spacing::SM],
        )
        .style(
            ArkUINodeAttributeType::BorderRadius,
            vec![radius::SM, radius::SM, radius::SM, radius::SM],
        )
        .style(ArkUINodeAttributeType::FontWeight, 3_i32)
        .style(ArkUINodeAttributeType::FontColor, color::POPOVER_FOREGROUND)
        .into()
}

pub fn dropdown_item_destructive(title: impl Into<String>) -> Element {
    button(title, ButtonVariant::Ghost)
        .height(36.0)
        .style(ArkUINodeAttributeType::RowJustifyContent, FLEX_ALIGN_START)
        .style(
            ArkUINodeAttributeType::Padding,
            vec![8.0, spacing::SM, 8.0, spacing::SM],
        )
        .style(
            ArkUINodeAttributeType::BorderRadius,
            vec![radius::SM, radius::SM, radius::SM, radius::SM],
        )
        .style(ArkUINodeAttributeType::FontWeight, 3_i32)
        .style(ArkUINodeAttributeType::FontColor, color::DESTRUCTIVE)
        .into()
}
