//! Separator — shadcn-style horizontal/vertical divider.
//!
//! Migrated from the original Elm builder API to dioxus 0.7 `#[component]` +
//! `rsx!`. Horizontal renders a 1px full-width row; vertical renders a 1px
//! column of the given height. Both use the theme `border` color.

use crate::theme::*;
use arkit_prelude::*;

/// Props for [`Separator`].
#[derive(Props, Clone, PartialEq)]
pub struct SeparatorProps {
    /// When `Some`, renders a vertical separator of the given height. When
    /// `None`, renders a horizontal full-width separator.
    pub vertical_height: Option<f32>,
}

/// A horizontal or vertical divider.
#[component]
pub fn Separator(props: SeparatorProps) -> Element {
    let theme = use_theme();
    match props.vertical_height {
        Some(height) => rsx! {
            column {
                width: 1.0,
                height: height,
                background_color: theme.colors.border,
            }
        },
        None => rsx! {
            row {
                height: 1.0,
                width: "100%",
                background_color: theme.colors.border,
            }
        },
    }
}
