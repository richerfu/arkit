use super::*;

pub fn accordion(children: Vec<Element>) -> Element {
    arkit::column_component()
        .percent_width(1.0)
        .style(
            ArkUINodeAttributeType::BorderWidth,
            vec![1.0, 1.0, 1.0, 1.0],
        )
        .style(ArkUINodeAttributeType::BorderColor, vec![color::BORDER])
        .style(
            ArkUINodeAttributeType::BorderRadius,
            vec![radius::MD, radius::MD, radius::MD, radius::MD],
        )
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
        .style(ArkUINodeAttributeType::RowAlignItems, FLEX_ALIGN_CENTER)
        .style(
            ArkUINodeAttributeType::RowJustifyContent,
            FLEX_ALIGN_SPACE_BETWEEN,
        )
        .style(
            ArkUINodeAttributeType::Padding,
            vec![spacing::MD, spacing::MD, spacing::MD, spacing::MD],
        )
        .on_click(move || click.update(|v| *v = !*v))
        .children(vec![
            body_text(title)
                .style(ArkUINodeAttributeType::FontColor, color::FOREGROUND)
                .into(),
            muted_text(if open.get() { "⌃" } else { "⌄" }).into(),
        ])
        .into()];

    if open.get() {
        children.push(
            arkit::column_component()
                .percent_width(1.0)
                .style(
                    ArkUINodeAttributeType::Padding,
                    vec![0.0, spacing::MD, spacing::MD, spacing::MD],
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

pub fn collapsible(title: impl Into<String>, open: Signal<bool>, content: Vec<Element>) -> Element {
    accordion_item(title, open, content)
}

pub fn dialog(title: impl Into<String>, open: Signal<bool>, content: Vec<Element>) -> Element {
    if !open.get() {
        return arkit::row_component().into();
    }

    arkit::column_component()
        .percent_width(1.0)
        .percent_height(1.0)
        .background_color(0x7F000000)
        .style(
            ArkUINodeAttributeType::ColumnJustifyContent,
            FLEX_ALIGN_CENTER,
        )
        .style(ArkUINodeAttributeType::ColumnAlignItems, FLEX_ALIGN_CENTER)
        .style(
            ArkUINodeAttributeType::Padding,
            vec![spacing::XXL, spacing::XXL, spacing::XXL, spacing::XXL],
        )
        .children(vec![panel_surface(
            arkit::column_component()
                .percent_width(1.0)
                .style(
                    ArkUINodeAttributeType::Padding,
                    vec![spacing::LG, spacing::LG, spacing::LG, spacing::LG],
                )
                .children(
                    std::iter::once(dialog_header(title, ""))
                        .chain(content)
                        .collect::<Vec<_>>(),
                ),
        )
        .into()])
        .into()
}

pub fn alert_dialog(
    title: impl Into<String>,
    description: impl Into<String>,
    actions: Vec<Element>,
) -> Element {
    card(vec![
        dialog_header(title, description),
        dialog_footer(actions),
    ])
}

pub fn alert_dialog_actions(actions: Vec<Element>) -> Element {
    dialog_footer(actions)
}

pub fn popover(trigger: Element, content: Vec<Element>, open: Signal<bool>) -> Element {
    if !open.get() {
        return trigger;
    }

    arkit::column_component()
        .percent_width(1.0)
        .children(vec![
            trigger,
            margin_top(
                panel_surface(
                    arkit::column_component()
                        .width(288.0)
                        .style(
                            ArkUINodeAttributeType::Padding,
                            vec![spacing::MD, spacing::MD, spacing::MD, spacing::MD],
                        )
                        .children(vec![stack(content, spacing::SM)]),
                ),
                spacing::XXS,
            )
            .into(),
        ])
        .into()
}

pub fn hover_card(trigger: Element, content: Vec<Element>, show: bool) -> Element {
    if show {
        arkit::column_component()
            .percent_width(1.0)
            .children(vec![
                trigger,
                arkit::row_component()
                    .style(
                        ArkUINodeAttributeType::Margin,
                        vec![spacing::SM, 0.0, 0.0, 0.0],
                    )
                    .children(vec![card(content)])
                    .into(),
            ])
            .into()
    } else {
        trigger
    }
}

pub fn dropdown_menu(trigger: Element, items: Vec<Element>, open: Signal<bool>) -> Element {
    popover(trigger, items, open)
}

pub fn context_menu(trigger: Element, items: Vec<Element>, open: Signal<bool>) -> Element {
    popover(trigger, items, open)
}

pub fn sheet(title: impl Into<String>, open: Signal<bool>, content: Vec<Element>) -> Element {
    dialog(title, open, content)
}

pub fn drawer(title: impl Into<String>, open: Signal<bool>, content: Vec<Element>) -> Element {
    dialog(title, open, content)
}

pub fn dialog_footer(actions: Vec<Element>) -> Element {
    arkit::row_component()
        .percent_width(1.0)
        .style(ArkUINodeAttributeType::RowAlignItems, FLEX_ALIGN_CENTER)
        .style(ArkUINodeAttributeType::RowJustifyContent, FLEX_ALIGN_END)
        .children(inline(actions, spacing::XS))
        .into()
}

pub fn dialog_header(title: impl Into<String>, description: impl Into<String>) -> Element {
    let title = title.into();
    let description = description.into();
    let mut children = vec![title_text(title).into()];
    if !description.is_empty() {
        children.push(margin_top(muted_text(description), spacing::XXS).into());
    }
    arkit::column_component()
        .percent_width(1.0)
        .children(children)
        .into()
}
