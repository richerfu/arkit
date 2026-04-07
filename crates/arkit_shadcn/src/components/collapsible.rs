use super::*;

pub fn collapsible(title: impl Into<String>, open: Signal<bool>, content: Vec<Element>) -> Element {
    let click = open.clone();
    let mut items = content.into_iter();
    let first = items.next();
    let rest: Vec<Element> = items
        .map(|child| {
            arkit::row_component()
                .style(
                    ArkUINodeAttributeType::Margin,
                    vec![spacing::SM, 0.0, 0.0, 0.0],
                )
                .children(vec![child])
                .into()
        })
        .collect();

    let mut children: Vec<Element> = vec![arkit::row_component()
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
        .on_click(move || click.update(|value| *value = !*value))
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
            arkit::row_component()
                .style(
                    ArkUINodeAttributeType::Margin,
                    vec![spacing::SM, 0.0, 0.0, 0.0],
                )
                .children(vec![first])
                .into(),
        );
    }

    // Wrap rest children in a visibility-toggled container.
    // `watch_signal` reactively toggles visibility when the `open` signal changes,
    // avoiding the stale static `if open.get()` that only evaluated once at mount.
    if !rest.is_empty() {
        children.push(
            visibility_gate(arkit::column_component().percent_width(1.0), open)
                .children(rest)
                .into(),
        );
    }

    arkit::column_component()
        .percent_width(1.0)
        .children(children)
        .into()
}
