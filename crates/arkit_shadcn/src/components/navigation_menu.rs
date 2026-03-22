use super::*;

pub fn navigation_menu(items: Vec<Element>) -> Element {
    card(vec![muted_text("Navigation").into(), super::menubar::menubar(items)])
}

pub fn navigation_item(title: impl Into<String>, active: bool) -> Element {
    let variant = if active {
        ButtonVariant::Secondary
    } else {
        ButtonVariant::Ghost
    };
    button(title, variant).into()
}
