//! Progress — shadcn-style progress bar.
//!
//! Migrated from the original Elm builder API to dioxus 0.7 `#[component]` +
//! `rsx!`. Preserves the original styling: `primary` progress color, 8px
//! height, `full` corner radius, clipped, `secondary` track background,
//! linear type.

use crate::theme::*;
use arkit_prelude::*;

/// Props for [`Progress`].
#[derive(Props, Clone, PartialEq)]
pub struct ProgressProps {
    pub value: f32,
    pub total: Option<f32>,
}

/// A horizontal progress bar.
#[component]
pub fn Progress(props: ProgressProps) -> Element {
    let theme = use_theme();
    let total = props.total.unwrap_or(100.0);
    rsx! {
        progress {
            progress_value: props.value,
            progress_total: total,
            progress_color: theme.colors.primary,
            progress_type: 0,
            height: 8.0,
            border_radius: theme.radii.full,
            clip: true,
            background_color: theme.colors.secondary,
        }
    }
}
