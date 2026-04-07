use super::*;

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
        .align_items_center()
        .children(inline(items, spacing::XXS))
        .into()
}

pub fn pagination_item(page_num: i32, current: Signal<i32>) -> Element {
    let click = current.clone();
    arkit::dynamic(move || {
        let variant = if current.get() == page_num {
            ButtonVariant::Outline
        } else {
            ButtonVariant::Ghost
        };
        let click = click.clone();
        button(page_num.to_string(), variant)
            .width(36.0)
            .height(36.0)
            .style(ArkUINodeAttributeType::Padding, vec![0.0, 0.0, 0.0, 0.0])
            .on_click(move || click.set(page_num))
            .into()
    })
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
