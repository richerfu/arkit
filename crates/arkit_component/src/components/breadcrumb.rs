//! Breadcrumb — shadcn-style breadcrumb trail.
//!
//! Migrated from the original Elm builder API to dioxus 0.7 `#[component]` +
//! `rsx!`. [`Breadcrumb`] renders a row of items separated by `/`; non-final
//! items are muted, the final item is foreground-colored. [`BreadcrumbItem`]
//! renders a standalone muted item.

use crate::style::*;
use arkit_prelude::*;

/// Props for [`Breadcrumb`].
#[derive(Props, Clone, PartialEq)]
pub struct BreadcrumbProps {
    pub items: Vec<String>,
}

/// A breadcrumb trail.
#[component]
pub fn Breadcrumb(props: BreadcrumbProps) -> Element {
    let theme = use_theme();
    let total = props.items.len();
    let rows: Vec<Element> = props
        .items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let is_last = index + 1 == total;
            let content = item.clone();
            if is_last {
                rsx! {
                    text {
                        content: content,
                        font_size: typography::MD,
                        font_color: theme.colors.foreground,
                        line_height: 20.0,
                    }
                }
            } else {
                rsx! {
                    text {
                        content: content,
                        font_size: typography::SM,
                        font_color: theme.colors.muted_foreground,
                        line_height: 20.0,
                    }
                    text {
                        content: "/".to_string(),
                        font_size: typography::SM,
                        font_color: theme.colors.muted_foreground,
                    }
                }
            }
        })
        .collect();

    rsx! {
        row {
            width: "100%",
            align_items: "center",
            {rows.into_iter()}
        }
    }
}

/// Props for [`BreadcrumbItem`].
#[derive(Props, Clone, PartialEq)]
pub struct BreadcrumbItemProps {
    pub content: String,
}

/// A standalone muted breadcrumb item.
#[component]
pub fn BreadcrumbItem(props: BreadcrumbItemProps) -> Element {
    let theme = use_theme();
    rsx! {
        text {
            content: props.content.clone(),
            font_size: typography::SM,
            font_color: theme.colors.muted_foreground,
            line_height: 20.0,
        }
    }
}
