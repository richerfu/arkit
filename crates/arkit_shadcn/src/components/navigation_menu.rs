use super::*;

pub fn navigation_menu(items: Vec<Element>) -> Element {
    shadow_sm(
        arkit::column_component::<(), arkit::Theme>()
            .padding(spacing::SM)
            .border_radius(radius::LG)
            .border_width(1.0)
            .border_color(color::BORDER)
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
    button(title).theme(variant).into()
}
