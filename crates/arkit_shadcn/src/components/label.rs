//! Label — shadcn-style form label.
//!
//! Migrated from the original Elm builder API to dioxus 0.7 `#[component]` +
//! `rsx!`. Preserves the original styling: `text-sm` (14), `font-medium`
//! (`W500`), foreground color, full-width start alignment.

use crate::theme::*;
use arkit_prelude::*;

/// Props for [`Label`].
#[derive(Props, Clone, PartialEq)]
pub struct LabelProps {
    pub content: String,
}

/// A form label — small, medium-weight, foreground-colored text.
#[component]
pub fn Label(props: LabelProps) -> Element {
    let theme = use_theme();
    rsx! {
        text {
            content: props.content.clone(),
            percent_width: 1.0,
            font_size: typography::SM,
            font_weight: 500,
            font_color: theme.colors.foreground,
            line_height: 14.0,
            text_align: 0,
        }
    }
}
