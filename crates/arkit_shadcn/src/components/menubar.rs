use super::*;

pub fn menubar(items: Vec<Element>) -> Element {
    shadow_sm(rounded_menubar_surface(
        arkit::row_component().children(inline(items, spacing::XXS)),
    ))
    .into()
}

pub fn menubar_item(title: impl Into<String>) -> Element {
    button(title, ButtonVariant::Ghost)
        .height(32.0)
        .style(
            ArkUINodeAttributeType::Padding,
            vec![6.0, spacing::SM, 6.0, spacing::SM],
        )
        .into()
}

pub fn menubar_item_active(title: impl Into<String>) -> Element {
    button(title, ButtonVariant::Ghost)
        .height(32.0)
        .style(
            ArkUINodeAttributeType::Padding,
            vec![6.0, spacing::SM, 6.0, spacing::SM],
        )
        .background_color(color::ACCENT)
        .style(ArkUINodeAttributeType::FontColor, color::ACCENT_FOREGROUND)
        .into()
}
