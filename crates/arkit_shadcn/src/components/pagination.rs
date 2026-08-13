//! Pagination — official ghost prev/next + outline current page.

use arkit_prelude::*;

use crate::spec;
use crate::theme::use_theme;

#[component]
pub fn Pagination(
    page: i32,
    total_pages: i32,
    previous_label: Option<String>,
    next_label: Option<String>,
    #[props(default)] on_page_change: EventHandler<i32>,
) -> Element {
    let theme = use_theme();
    let total = total_pages.max(1);
    let current = page.clamp(1, total);
    let prev_l = previous_label.unwrap_or_else(|| "Previous".into());
    let next_l = next_label.unwrap_or_else(|| "Next".into());
    let mut nums: Vec<i32> = vec![1, total, current - 1, current, current + 1]
        .into_iter()
        .filter(|n| *n >= 1 && *n <= total)
        .collect();
    nums.sort_unstable();
    nums.dedup();
    let mut items = Vec::new();
    let prev_t = (current - 1).max(1);
    items.push(ghost_btn(prev_l, theme.colors.foreground, move || {
        on_page_change.call(prev_t)
    }));
    let mut last = None;
    for number in nums {
        if let Some(prev) = last {
            if number - prev > 1 {
                items.push(rsx! {
                    text {
                        content: "...",
                        font_size: spec::TEXT_SM,
                        font_color: theme.colors.muted_foreground,
                        margin_left: 4.0,
                        margin_right: 4.0,
                    }
                });
            }
        }
        let active = number == current;
        items.push(page_btn(
            number.to_string(),
            active,
            theme.colors.background,
            theme.colors.border,
            theme.colors.foreground,
            move || on_page_change.call(number),
        ));
        last = Some(number);
    }
    let next_t = (current + 1).min(total);
    items.push(ghost_btn(next_l, theme.colors.foreground, move || {
        on_page_change.call(next_t)
    }));
    rsx! {
        row { align_items: "center", {items.into_iter()} }
    }
}

fn ghost_btn(label: String, fg: u32, on_click: impl FnMut() + 'static) -> Element {
    let mut on_click = on_click;
    rsx! {
        button {
            button_type: "normal",
            height: 36.0,
            padding_left: 8.0,
            padding_right: 8.0,
            background_color: 0x00000000u32,
            border_width: 0.0,
            focusable: false,
            onclick: move |_| on_click(),
            text { content: label, font_size: spec::TEXT_SM, font_weight: 500, font_color: fg }
        }
    }
}

fn page_btn(
    label: String,
    active: bool,
    bg: u32,
    border: u32,
    fg: u32,
    on_click: impl FnMut() + 'static,
) -> Element {
    let mut on_click = on_click;
    rsx! {
        button {
            button_type: "normal",
            width: 36.0,
            height: 36.0,
            margin_left: 4.0,
            margin_right: 4.0,
            background_color: if active { bg } else { 0x00000000u32 },
            border_width: if active { 1.0 } else { 0.0 },
            border_color: border,
            border_radius: spec::RADIUS_MD,
            shadow: if active { "sm" },
            focusable: false,
            alignment: "center",
            onclick: move |_| on_click(),
            text { content: label, font_size: spec::TEXT_SM, font_weight: 500, font_color: fg }
        }
    }
}
