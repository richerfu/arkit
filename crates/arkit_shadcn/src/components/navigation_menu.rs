use super::button::button;
use super::*;

fn navigation_menu(items: Vec<Element>) -> Element {
    shadow_sm(
        arkit::column_component::<(), arkit::Theme>()
            .padding(spacing::SM)
            .border_radius(radii().lg)
            .border_width(1.0)
            .border_color(colors().border)
            .background_color(colors().background)
            .children(vec![arkit::row_component::<(), arkit::Theme>()
                .children(inline(items, spacing::XXS))
                .into()]),
    )
    .into()
}

fn navigation_item(title: impl Into<String>, active: bool) -> Element {
    let variant = if active {
        ButtonVariant::Secondary
    } else {
        ButtonVariant::Ghost
    };
    button(title).theme(variant).into()
}
