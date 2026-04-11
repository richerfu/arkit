use super::*;

pub fn collapsible<Message: Send + 'static>(
    title: impl Into<String>,
    open: bool,
    on_open_change: impl Fn(bool) + 'static,
    content: Vec<Element<Message>>,
) -> Element<Message> {
    let mut items = content.into_iter();
    let first = items.next();
    let rest: Vec<Element<Message>> = items
        .map(|child| {
            arkit::row_component::<Message, arkit::Theme>()
                .style(
                    ArkUINodeAttributeType::Margin,
                    vec![spacing::SM, 0.0, 0.0, 0.0],
                )
                .children(vec![child])
                .into()
        })
        .collect();

    let mut children: Vec<Element<Message>> = vec![arkit::row_component::<Message, arkit::Theme>()
        .percent_width(1.0)
        .align_items_center()
        .style(
            ArkUINodeAttributeType::RowJustifyContent,
            FLEX_ALIGN_SPACE_BETWEEN,
        )
        .style(
            ArkUINodeAttributeType::Padding,
            vec![0.0, spacing::LG, 0.0, spacing::LG],
        )
        .on_click(move || on_open_change(!open))
        .children(vec![
            body_text(title)
                .style(ArkUINodeAttributeType::FontWeight, 5_i32)
                .style(ArkUINodeAttributeType::FontColor, color::FOREGROUND)
                .into(),
            icon_button_with_variant("chevrons-up-down", ButtonVariant::Ghost)
                .width(32.0)
                .height(32.0)
                .style(ArkUINodeAttributeType::Padding, vec![0.0, 0.0, 0.0, 0.0])
                .into(),
        ])
        .into()];

    if let Some(first) = first {
        children.push(
            arkit::row_component::<Message, arkit::Theme>()
                .style(
                    ArkUINodeAttributeType::Margin,
                    vec![spacing::SM, 0.0, 0.0, 0.0],
                )
                .children(vec![first])
                .into(),
        );
    }

    // Keep the body mounted and let normal patching update visibility so layout
    // and interaction remain stable across explicit runtime rerenders.
    if !rest.is_empty() {
        children.push(
            visibility_gate(
                arkit::column_component::<Message, arkit::Theme>().percent_width(1.0),
                open,
            )
            .children(rest)
            .into(),
        );
    }

    arkit::column_component::<Message, arkit::Theme>()
        .percent_width(1.0)
        .children(children)
        .into()
}

pub fn collapsible_message<Message>(
    title: impl Into<String>,
    open: bool,
    on_open_change: impl Fn(bool) -> Message + 'static,
    content: Vec<Element<Message>>,
) -> Element<Message>
where
    Message: Send + 'static,
{
    collapsible(
        title,
        open,
        move |value| dispatch_message(on_open_change(value)),
        content,
    )
}
