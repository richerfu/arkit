use super::*;
use arkit_icon as lucide;

pub fn accordion(children: Vec<Element>) -> Element {
    arkit::column_component()
        .percent_width(1.0)
        .children(children)
        .into()
}

pub fn accordion_item(
    title: impl Into<String>,
    open: Signal<bool>,
    content: Vec<Element>,
) -> Element {
    let click = open.clone();
    let mut children = vec![arkit::row_component()
        .percent_width(1.0)
        .align_items_top()
        .style(
            ArkUINodeAttributeType::RowJustifyContent,
            FLEX_ALIGN_SPACE_BETWEEN,
        )
        .style(
            ArkUINodeAttributeType::Padding,
            vec![spacing::LG, 0.0, spacing::LG, 0.0],
        )
        .on_click(move || click.update(|value| *value = !*value))
        .children(vec![
            body_text(title)
                .style(ArkUINodeAttributeType::FontColor, color::FOREGROUND)
                .into(),
            lucide::icon(if open.get() {
                "chevron-up"
            } else {
                "chevron-down"
            })
            .size(16.0)
            .color(color::MUTED_FOREGROUND)
            .render(),
        ])
        .into()];

    if open.get() {
        children.push(
            arkit::column_component()
                .percent_width(1.0)
                .style(
                    ArkUINodeAttributeType::Padding,
                    vec![0.0, 0.0, spacing::LG, 0.0],
                )
                .children(content)
                .into(),
        );
    }

    arkit::column_component()
        .percent_width(1.0)
        .style(
            ArkUINodeAttributeType::BorderWidth,
            vec![0.0, 0.0, 1.0, 0.0],
        )
        .style(ArkUINodeAttributeType::BorderColor, vec![color::BORDER])
        .children(children)
        .into()
}
