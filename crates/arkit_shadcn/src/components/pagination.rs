//! Pagination — shadcn-style page navigation.
//!
//! Migrated from the original Elm builder API to dioxus 0.7 `#[component]` +
//! `rsx!`. Preserves the original Prev/Next ghost buttons, the numbered page
//! items (outline when active, ghost otherwise), the `...` ellipsis for gaps,
//! and the `XXS` inline spacing. Button variant styling (ghost/outline) is
//! inlined so this component is self-contained.

use crate::theme::*;
use arkit_prelude::*;

use super::{ARKUI_BORDER_STYLE_SOLID, ARKUI_BUTTON_TYPE_NORMAL};

const TRANSPARENT: u32 = 0x00000000;

/// Props for [`Pagination`].
#[derive(Props, Clone, PartialEq)]
pub struct PaginationProps {
    pub page: i32,
    pub total_pages: i32,
    #[props(default)]
    pub on_page_change: EventHandler<i32>,
}

/// A pagination bar — Prev, numbered pages (with ellipses for gaps), Next.
#[component]
pub fn Pagination(props: PaginationProps) -> Element {
    let theme = use_theme();
    let total_pages = props.total_pages.max(1);
    let current = props.page.clamp(1, total_pages);
    let on_page_change = props.on_page_change;

    let mut page_numbers: Vec<i32> = vec![1, total_pages, current - 1, current, current + 1]
        .into_iter()
        .filter(|value| *value >= 1 && *value <= total_pages)
        .collect();
    page_numbers.sort_unstable();
    page_numbers.dedup();

    let mut items: Vec<Element> = Vec::new();

    // Prev button (ghost).
    let prev_target = (current - 1).max(1);
    let on_prev = on_page_change;
    items.push(rsx! {
        button {
            button_type: ARKUI_BUTTON_TYPE_NORMAL,
            focusable: false,
            focus_on_touch: false,
            background_color: TRANSPARENT,
            border_style: ARKUI_BORDER_STYLE_SOLID,
            border_width: 0.0,
            border_color: TRANSPARENT,
            height: 36.0,
            padding_top: 10.0,
            padding_right: 0.0,
            padding_bottom: 10.0,
            padding_left: 0.0,
            alignment: 4,
            onclick: move |_| on_prev.call(prev_target),
            text {
                content: "Prev",
                font_size: typography::SM,
                font_weight: 500,
                font_color: theme.colors.foreground,
                line_height: 20.0,
            }
        }
    });

    let mut previous_number: Option<i32> = None;
    for number in page_numbers {
        if let Some(last) = previous_number {
            if number - last > 1 {
                items.push(rsx! {
                    row {
                        width: 36.0,
                        height: 36.0,
                        align_items: "center",
                        justify_content: "center",
                        text {
                            content: "...",
                            font_size: typography::SM,
                            font_color: theme.colors.muted_foreground,
                            line_height: 20.0,
                        }
                    }
                });
            }
        }

        let is_active = current == number;
        let background = if is_active {
            theme.colors.background
        } else {
            TRANSPARENT
        };
        let border_width = if is_active { 1.0 } else { 0.0 };
        let border_color = if is_active {
            theme.colors.border
        } else {
            TRANSPARENT
        };
        let shadow = if is_active { 1 } else { 0 };
        let on_page = on_page_change;
        items.push(rsx! {
            button {
                button_type: ARKUI_BUTTON_TYPE_NORMAL,
                focusable: false,
                focus_on_touch: false,
                width: 36.0,
                height: 36.0,
                padding_top: 0.0,
                padding_right: 0.0,
                padding_bottom: 0.0,
                padding_left: 0.0,
                background_color: background,
                border_style: ARKUI_BORDER_STYLE_SOLID,
                border_width: border_width,
                border_color: border_color,
                border_radius: theme.radii.md,
                alignment: 4,
                shadow: shadow,
                onclick: move |_| on_page.call(number),
                text {
                    content: number.to_string(),
                    font_size: typography::SM,
                    font_weight: 500,
                    font_color: theme.colors.foreground,
                    line_height: 20.0,
                }
            }
        });
        previous_number = Some(number);
    }

    // Next button (ghost).
    let next_target = (current + 1).min(total_pages);
    let on_next = on_page_change;
    items.push(rsx! {
        button {
            button_type: ARKUI_BUTTON_TYPE_NORMAL,
            focusable: false,
            focus_on_touch: false,
            background_color: TRANSPARENT,
            border_style: ARKUI_BORDER_STYLE_SOLID,
            border_width: 0.0,
            border_color: TRANSPARENT,
            height: 36.0,
            padding_top: 10.0,
            padding_right: 0.0,
            padding_bottom: 10.0,
            padding_left: 0.0,
            alignment: 4,
            onclick: move |_| on_next.call(next_target),
            text {
                content: "Next",
                font_size: typography::SM,
                font_weight: 500,
                font_color: theme.colors.foreground,
                line_height: 20.0,
            }
        }
    });

    // Inline the items with `XXS` left margin between them.
    let inlined: Vec<Element> = items
        .into_iter()
        .enumerate()
        .map(|(i, child)| {
            if i == 0 {
                child
            } else {
                rsx! { row { margin_left: spacing::XXS, {child} } }
            }
        })
        .collect();

    rsx! {
        row {
            percent_width: 1.0,
            align_items: "center",
            {inlined.into_iter()}
        }
    }
}
