use super::*;

pub fn sidebar(navigation: Vec<Element>, content: Element) -> Element {
    arkit::row_component()
        .percent_width(1.0)
        .children(vec![
            panel_surface(arkit::column_component().width(180.0).children(navigation)).into(),
            content,
        ])
        .into()
}

pub fn sidebar_item(title: impl Into<String>, active: bool) -> Element {
    let variant = if active {
        ButtonVariant::Secondary
    } else {
        ButtonVariant::Ghost
    };
    button(title).theme(variant).into()
}
