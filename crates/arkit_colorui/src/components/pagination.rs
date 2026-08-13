//! ColorUI pagination — `cu-btn sm` pages, filled current page.

use arkit_prelude::*;

use crate::theme::{swatch, use_colorui_theme};

#[component]
pub fn Pagination(
    page: i32,
    total_pages: i32,
    previous_label: Option<String>,
    next_label: Option<String>,
    #[props(default)] on_page_change: EventHandler<i32>,
) -> Element {
    let theme = use_colorui_theme();
    let tone = swatch(theme.primary);
    let tokens = theme.tokens();
    let total_pages = total_pages.max(1);
    let current = page.clamp(1, total_pages);
    let on_page_change = on_page_change;
    let previous_label = previous_label.unwrap_or_else(|| "上一页".into());
    let next_label = next_label.unwrap_or_else(|| "下一页".into());

    let mut page_numbers: Vec<i32> = vec![1, total_pages, current - 1, current, current + 1]
        .into_iter()
        .filter(|value| *value >= 1 && *value <= total_pages)
        .collect();
    page_numbers.sort_unstable();
    page_numbers.dedup();

    let mut items: Vec<Element> = Vec::new();
    let prev_target = (current - 1).max(1);
    let on_prev = on_page_change;
    items.push(page_btn(
        previous_label,
        false,
        tone.fill,
        tokens.colors.foreground,
        move || on_prev.call(prev_target),
    ));

    let mut previous_number: Option<i32> = None;
    for number in page_numbers {
        if let Some(prev) = previous_number {
            if number - prev > 1 {
                items.push(rsx! {
                    text {
                        content: "...",
                        font_size: 12.0,
                        font_color: tokens.colors.muted_foreground,
                        margin_left: 4.0,
                        margin_right: 4.0,
                    }
                });
            }
        }
        let active = number == current;
        let on_page = on_page_change;
        items.push(page_btn(
            number.to_string(),
            active,
            tone.fill,
            tokens.colors.foreground,
            move || on_page.call(number),
        ));
        previous_number = Some(number);
    }

    let next_target = (current + 1).min(total_pages);
    let on_next = on_page_change;
    items.push(page_btn(
        next_label,
        false,
        tone.fill,
        tokens.colors.foreground,
        move || on_next.call(next_target),
    ));

    rsx! {
        row {
            align_items: "center",
            {items.into_iter()}
        }
    }
}

fn page_btn(
    label: String,
    active: bool,
    fill: u32,
    muted: u32,
    on_click: impl FnMut() + 'static,
) -> Element {
    let mut on_click = on_click;
    rsx! {
        button {
            button_type: "normal",
            height: 24.0,
            min_width: 24.0,
            margin_left: 4.0,
            margin_right: 4.0,
            padding_left: 10.0,
            padding_right: 10.0,
            background_color: if active { fill } else { 0x00000000u32 },
            border_width: if active { 0.0 } else { 1.0 },
            border_color: if active { 0x00000000u32 } else { fill },
            border_radius: 6.0,
            focusable: false,
            focus_on_touch: false,
            alignment: "center",
            onclick: move |_| on_click(),
            text {
                content: label,
                font_size: 12.0,
                font_color: if active { 0xFFFFFFFFu32 } else { muted },
            }
        }
    }
}
