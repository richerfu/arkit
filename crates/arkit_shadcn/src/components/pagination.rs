use super::*;
use std::rc::Rc;

pub fn pagination<Message>(
    page: i32,
    total_pages: i32,
    on_page_change: impl Fn(i32) -> Message + 'static,
) -> Element<Message>
where
    Message: Clone + Send + 'static,
{
    let total_pages = total_pages.max(1);
    let current = page.clamp(1, total_pages);
    let on_page_change = Rc::new(on_page_change) as Rc<dyn Fn(i32) -> Message>;
    let mut page_numbers = vec![1_i32, total_pages, current - 1, current, current + 1]
        .into_iter()
        .filter(|value| *value >= 1 && *value <= total_pages)
        .collect::<Vec<_>>();
    page_numbers.sort_unstable();
    page_numbers.dedup();

    let mut items: Vec<Element<Message>> = vec![button("Prev")
        .theme(ButtonVariant::Ghost)
        .height(36.0)
        .padding([10.0, 0.0])
        .on_press((on_page_change.as_ref())((current - 1).max(1)))
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
        button("Next")
            .theme(ButtonVariant::Ghost)
            .height(36.0)
            .padding([10.0, 0.0])
            .on_press((on_page_change.as_ref())((current + 1).min(total_pages)))
            .into(),
    );

    arkit::row_component::<Message, arkit::Theme>()
        .percent_width(1.0)
        .align_items_center()
        .children(inline(items, spacing::XXS))
        .into()
}

pub fn pagination_item<Message>(
    page_num: i32,
    current: i32,
    on_page_change: Rc<dyn Fn(i32) -> Message>,
) -> Element<Message>
where
    Message: Clone + Send + 'static,
{
    let variant = if current == page_num {
        ButtonVariant::Outline
    } else {
        ButtonVariant::Ghost
    };
    button(page_num.to_string())
        .theme(variant)
        .width(36.0)
        .height(36.0)
        .padding(arkit::Padding::ZERO)
        .on_press((on_page_change.as_ref())(page_num))
        .into()
}

fn pagination_ellipsis<Message: 'static>() -> Element<Message> {
    arkit::row_component()
        .width(36.0)
        .height(36.0)
        .align_items_center()
        .justify_content_center()
        .children(vec![arkit::text::<Message, arkit::Theme>("...")
            .font_size(typography::SM)
            .font_color(colors().muted_foreground)
            .line_height(20.0)
            .into()])
        .into()
}
