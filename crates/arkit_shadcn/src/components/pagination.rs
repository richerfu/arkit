use super::*;

pub fn pagination(page: i32, total_pages: i32, on_page_change: impl Fn(i32) + 'static) -> Element {
    let total_pages = total_pages.max(1);
    let current = page.clamp(1, total_pages);
    let on_page_change = std::rc::Rc::new(on_page_change);
    let mut page_numbers = vec![1_i32, total_pages, current - 1, current, current + 1]
        .into_iter()
        .filter(|value| *value >= 1 && *value <= total_pages)
        .collect::<Vec<_>>();
    page_numbers.sort_unstable();
    page_numbers.dedup();

    let mut items = vec![button("Prev", ButtonVariant::Ghost)
        .height(36.0)
        .style(ArkUINodeAttributeType::Padding, vec![0.0, 10.0, 0.0, 10.0])
        .on_click({
            let on_page_change = on_page_change.clone();
            move || on_page_change((current - 1).max(1))
        })
        .into()];

    let mut previous_number = None;
    for number in page_numbers {
        if let Some(last) = previous_number {
            if number - last > 1 {
                items.push(pagination_ellipsis());
            }
        }
        items.push(pagination_item(number, current, on_page_change.clone()));
        previous_number = Some(number);
    }

    items.push(
        button("Next", ButtonVariant::Ghost)
            .height(36.0)
            .style(ArkUINodeAttributeType::Padding, vec![0.0, 10.0, 0.0, 10.0])
            .on_click(move || on_page_change((current + 1).min(total_pages)))
            .into(),
    );

    arkit::row_component()
        .percent_width(1.0)
        .align_items_center()
        .children(inline(items, spacing::XXS))
        .into()
}

pub fn pagination_item(
    page_num: i32,
    current: i32,
    on_page_change: std::rc::Rc<dyn Fn(i32)>,
) -> Element {
    let variant = if current == page_num {
        ButtonVariant::Outline
    } else {
        ButtonVariant::Ghost
    };
    button(page_num.to_string(), variant)
        .width(36.0)
        .height(36.0)
        .style(ArkUINodeAttributeType::Padding, vec![0.0, 0.0, 0.0, 0.0])
        .on_click(move || on_page_change(page_num))
        .into()
}

fn pagination_ellipsis() -> Element {
    arkit::row_component()
        .width(36.0)
        .height(36.0)
        .align_items_center()
        .style(ArkUINodeAttributeType::RowJustifyContent, FLEX_ALIGN_CENTER)
        .children(vec![arkit::text("...")
            .font_size(typography::SM)
            .style(ArkUINodeAttributeType::FontColor, color::MUTED_FOREGROUND)
            .style(ArkUINodeAttributeType::TextLineHeight, 20.0)
            .into()])
        .into()
}
