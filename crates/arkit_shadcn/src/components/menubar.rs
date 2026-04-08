use super::*;

pub fn menubar<Message: 'static>(items: Vec<Element<Message>>) -> Element<Message> {
    shadow_sm(rounded_menubar_surface(
        arkit::row_component::<Message, arkit::Theme>().children(inline(items, spacing::XXS)),
    ))
    .into()
}

pub fn menubar_item<Message: Send + 'static>(title: impl Into<String>) -> Element<Message> {
    button::<Message>(title, ButtonVariant::Ghost)
        .height(32.0)
        .style(
            ArkUINodeAttributeType::Padding,
            vec![6.0, spacing::SM, 6.0, spacing::SM],
        )
        .into()
}

pub fn menubar_item_active<Message: Send + 'static>(title: impl Into<String>) -> Element<Message> {
    button::<Message>(title, ButtonVariant::Secondary)
        .height(32.0)
        .style(
            ArkUINodeAttributeType::Padding,
            vec![6.0, spacing::SM, 6.0, spacing::SM],
        )
        .into()
}
