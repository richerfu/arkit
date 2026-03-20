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
    let total_pages = total_pages.max(1);
    let current = page.get().clamp(1, total_pages);
    let prev_page = page.clone();
    let next_page = page.clone();
    let mut page_numbers = vec![1_i32, total_pages, current - 1, current, current + 1]
        .into_iter()
        .filter(|value| *value >= 1 && *value <= total_pages)
        .collect::<Vec<_>>();
    page_numbers.sort_unstable();
    page_numbers.dedup();

    let mut items = vec![button("Prev", ButtonVariant::Ghost)
        .height(36.0)
        .style(ArkUINodeAttributeType::Padding, vec![0.0, 10.0, 0.0, 10.0])
        .on_click(move || {
            prev_page.update(|p| {
                if *p > 1 {
                    *p -= 1;
                }
            });
        })
        .into()];

    let mut previous_number = None;
    for number in page_numbers {
        if let Some(last) = previous_number {
            if number - last > 1 {
                items.push(pagination_ellipsis());
            }
        }
        items.push(pagination_item(number, page.clone()));
        previous_number = Some(number);
    }

    items.push(
        button("Next", ButtonVariant::Ghost)
            .height(36.0)
            .style(ArkUINodeAttributeType::Padding, vec![0.0, 10.0, 0.0, 10.0])
            .on_click(move || {
                next_page.update(|p| {
                    if *p < total_pages {
                        *p += 1;
                    }
                });
            })
            .into(),
    );

    arkit::row_component()
        .percent_width(1.0)
        .style(ArkUINodeAttributeType::RowAlignItems, FLEX_ALIGN_CENTER)
        .children(inline(items, spacing::XXS))
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
    button(title, ButtonVariant::Ghost)
        .height(32.0)
        .style(ArkUINodeAttributeType::RowJustifyContent, FLEX_ALIGN_START)
        .style(
            ArkUINodeAttributeType::Padding,
            vec![6.0, spacing::SM, 6.0, spacing::SM],
        )
        .style(
            ArkUINodeAttributeType::BorderRadius,
            vec![radius::SM, radius::SM, radius::SM, radius::SM],
        )
        .style(ArkUINodeAttributeType::FontWeight, 3_i32)
        .style(ArkUINodeAttributeType::FontColor, color::POPOVER_FOREGROUND)
        .into()
}

pub fn dropdown_item_destructive(title: impl Into<String>) -> Element {
    button(title, ButtonVariant::Ghost)
        .height(32.0)
        .style(ArkUINodeAttributeType::RowJustifyContent, FLEX_ALIGN_START)
        .style(
            ArkUINodeAttributeType::Padding,
            vec![6.0, spacing::SM, 6.0, spacing::SM],
        )
        .style(
            ArkUINodeAttributeType::BorderRadius,
            vec![radius::SM, radius::SM, radius::SM, radius::SM],
        )
        .style(ArkUINodeAttributeType::FontWeight, 3_i32)
        .style(ArkUINodeAttributeType::FontColor, color::DESTRUCTIVE)
        .into()
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

pub fn tabs_list(tab_labels: Vec<String>, active: Signal<usize>) -> Element {
    let children = tab_labels
        .into_iter()
        .enumerate()
        .map(|(idx, label)| {
            let click = active.clone();
            let is_active = active.get() == idx;
            arkit::row_component()
                .height(29.0)
                .style(ArkUINodeAttributeType::RowAlignItems, FLEX_ALIGN_CENTER)
                .style(ArkUINodeAttributeType::RowJustifyContent, FLEX_ALIGN_CENTER)
                .style(
                    ArkUINodeAttributeType::Padding,
                    vec![spacing::XXS, spacing::SM, spacing::XXS, spacing::SM],
                )
                .style(
                    ArkUINodeAttributeType::BorderRadius,
                    vec![radius::MD, radius::MD, radius::MD, radius::MD],
                )
                .style(
                    ArkUINodeAttributeType::BorderWidth,
                    vec![1.0, 1.0, 1.0, 1.0],
                )
                .style(ArkUINodeAttributeType::BorderColor, vec![0x00000000])
                .background_color(if is_active {
                    color::BACKGROUND
                } else {
                    0x00000000
                })
                .on_click(move || click.set(idx))
                .children(vec![body_text(label)
                    .style(ArkUINodeAttributeType::FontColor, color::FOREGROUND)
                    .into()])
                .into()
        })
        .collect::<Vec<_>>();
    rounded_tabs_list_surface(
        arkit::row_component()
            .style(ArkUINodeAttributeType::RowAlignItems, FLEX_ALIGN_CENTER)
            .children(children),
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
        ButtonVariant::Outline
    } else {
        ButtonVariant::Ghost
    };
    let click = current.clone();
    button(page_num.to_string(), variant)
        .width(36.0)
        .height(36.0)
        .style(ArkUINodeAttributeType::Padding, vec![0.0, 0.0, 0.0, 0.0])
        .on_click(move || click.set(page_num))
        .into()
}

fn pagination_ellipsis() -> Element {
    arkit::row_component()
        .width(36.0)
        .height(36.0)
        .style(ArkUINodeAttributeType::RowAlignItems, FLEX_ALIGN_CENTER)
        .style(ArkUINodeAttributeType::RowJustifyContent, FLEX_ALIGN_CENTER)
        .children(vec![arkit::text("...")
            .font_size(typography::SM)
            .style(ArkUINodeAttributeType::FontColor, color::MUTED_FOREGROUND)
            .style(ArkUINodeAttributeType::TextLineHeight, 20.0)
            .into()])
        .into()
}
