//! Skeleton — shadcn-style loading placeholder.
//!
//! Migrated from the original Elm builder API to dioxus 0.7 `#[component]` +
//! `rsx!`. Preserves the original logic: a `accent`-colored box whose radius
//! becomes `full` for square shapes ≥ 40px, otherwise `md`.

use crate::theme::*;
use arkit_prelude::*;

/// Props for [`Skeleton`].
#[derive(Props, Clone, PartialEq)]
pub struct SkeletonProps {
    pub width: f32,
    pub height: f32,
}

/// A loading placeholder shaped like its eventual content.
#[component]
pub fn Skeleton(props: SkeletonProps) -> Element {
    let theme = use_theme();
    let radius = if (props.width - props.height).abs() < f32::EPSILON && props.width >= 40.0 {
        theme.radii.full
    } else {
        theme.radii.md
    };
    rsx! {
        row {
            width: props.width,
            height: props.height,
            background_color: theme.colors.accent,
            border_radius: radius,
        }
    }
}
