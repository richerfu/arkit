use super::*;

pub fn navigation_menu(items: Vec<Element>) -> Element {
    shadow_sm(
        arkit::column_component::<(), arkit::Theme>()
            .style(
                ArkUINodeAttributeType::Padding,
                vec![spacing::SM, spacing::SM, spacing::SM, spacing::SM],
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
            .children(vec![arkit::row_component::<(), arkit::Theme>()
                .children(inline(items, spacing::XXS))
                .into()]),
    )
    .into()
}

pub fn navigation_item(title: impl Into<String>, active: bool) -> Element {
    let variant = if active {
        ButtonVariant::Secondary
    } else {
        ButtonVariant::Ghost
    };
    button(title, variant).into()
}
