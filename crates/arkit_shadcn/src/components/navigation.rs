use super::*;

pub fn menubar(items: Vec<Element>) -> Element {
    rounded_menubar_surface(arkit::row_component().children(inline(items, spacing::XXS))).into()
}

pub fn navigation_menu(items: Vec<Element>) -> Element {
    card(vec![muted_text("Navigation").into(), menubar(items)])
}

pub fn breadcrumb(items: Vec<String>) -> Element {
    let mut children = Vec::new();
    let total = items.len();
    for (index, item) in items.into_iter().enumerate() {
        if index > 0 {
            children.push(
                arkit::text("/")
                    .font_size(typography::SM)
                    .style(ArkUINodeAttributeType::FontColor, color::MUTED_FOREGROUND)
                    .into(),
            );
        }
        if index + 1 == total {
            children.push(
                body_text_regular(item)
                    .style(ArkUINodeAttributeType::FontColor, color::FOREGROUND)
                    .into(),
            );
        } else {
            children.push(muted_text(item).into());
        }
    }
    arkit::row_component()
        .percent_width(1.0)
        .style(ArkUINodeAttributeType::RowAlignItems, FLEX_ALIGN_CENTER)
        .children(children)
        .into()
}

pub fn pagination(page: Signal<i32>, total_pages: i32) -> Element {
    let prev_page = page.clone();
    let next_page = page.clone();

    arkit::row_component()
        .percent_width(1.0)
        .style(ArkUINodeAttributeType::RowAlignItems, FLEX_ALIGN_CENTER)
        .children(inline(
            vec![
                button("Prev", ButtonVariant::Outline)
                    .on_click(move || {
                        prev_page.update(|p| {
                            if *p > 1 {
                                *p -= 1;
                            }
                        });
                    })
                    .into(),
                muted_text(format!("{}/{}", page.get(), total_pages)).into(),
                button("Next", ButtonVariant::Outline)
                    .on_click(move || {
                        next_page.update(|p| {
                            if *p < total_pages {
                                *p += 1;
                            }
                        });
                    })
                    .into(),
            ],
            spacing::XXS,
        ))
        .into()
}

pub fn tabs(tab_labels: Vec<String>, active: Signal<usize>, panels: Vec<Element>) -> Element {
    arkit::column_component()
        .percent_width(1.0)
        .children(vec![
            tabs_list(tab_labels, active.clone()),
            arkit::row_component()
                .style(
                    ArkUINodeAttributeType::Margin,
                    vec![spacing::SM, 0.0, 0.0, 0.0],
                )
                .children(vec![tabs_content(panels, active)])
                .into(),
        ])
        .into()
}

pub fn resizable(left: Element, right: Element) -> Element {
    arkit::row_component()
        .percent_width(1.0)
        .children(inline(
            vec![left, separator_vertical(120.0), right],
            spacing::SM,
        ))
        .into()
}

pub fn scroll_area(children: Vec<Element>) -> ScrollElement {
    panel_surface(
        arkit::scroll_component()
            .percent_width(1.0)
            .children(children),
    )
}

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
    button(title, variant).into()
}

pub fn dropdown_item(title: impl Into<String>) -> Element {
    button(title, ButtonVariant::Ghost).into()
}

pub fn breadcrumb_item(title: impl Into<String>) -> Element {
    muted_text(title).into()
}

pub fn navigation_item(title: impl Into<String>, active: bool) -> Element {
    let variant = if active {
        ButtonVariant::Secondary
    } else {
        ButtonVariant::Ghost
    };
    button(title, variant).into()
}

pub fn menubar_item(title: impl Into<String>) -> Element {
    button(title, ButtonVariant::Ghost)
        .height(30.0)
        .style(
            ArkUINodeAttributeType::Padding,
            vec![spacing::XXS, spacing::SM, spacing::XXS, spacing::SM],
        )
        .into()
}

pub fn tabs_list(tab_labels: Vec<String>, active: Signal<usize>) -> Element {
    let children = tab_labels
        .into_iter()
        .enumerate()
        .map(|(idx, label)| {
            let click = active.clone();
            let variant = if active.get() == idx {
                ButtonVariant::Outline
            } else {
                ButtonVariant::Ghost
            };
            button(label, variant)
                .height(30.0)
                .style(
                    ArkUINodeAttributeType::Padding,
                    vec![spacing::XXS, spacing::SM, spacing::XXS, spacing::SM],
                )
                .on_click(move || click.set(idx))
                .into()
        })
        .collect::<Vec<_>>();
    rounded_tabs_list_surface(
        arkit::row_component()
            .style(ArkUINodeAttributeType::RowAlignItems, FLEX_ALIGN_CENTER)
            .children(inline(children, spacing::XXS)),
    )
    .into()
}

pub fn tabs_content(panels: Vec<Element>, active: Signal<usize>) -> Element {
    panels
        .into_iter()
        .nth(active.get())
        .unwrap_or_else(|| arkit::column(vec![]))
}

pub fn pagination_item(page_num: i32, current: Signal<i32>) -> Element {
    let variant = if current.get() == page_num {
        ButtonVariant::Default
    } else {
        ButtonVariant::Outline
    };
    let click = current.clone();
    button(page_num.to_string(), variant)
        .on_click(move || click.set(page_num))
        .into()
}
