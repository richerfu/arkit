//! Skeleton — shadcn-style loading placeholder.
//!
//! Migrated from the original Elm builder API to dioxus 0.7 `#[component]` +
//! `rsx!`. Geometry: square ≥ 40vp uses full radius (circle), otherwise `md`.
//! Fill matches modern shadcn (`bg-primary/10`) so the block stays visible on
//! both pure white and the showcase `surface=secondary` canvas.

use crate::theme::*;
use arkit_prelude::*;

/// ~10% opacity primary — shadcn `bg-primary/10`.
const SKELETON_PRIMARY_ALPHA: u8 = 0x1A;

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
    // Do not use `accent`/`secondary`/`muted`: in this theme they are identical,
    // and the showcase remaps `surface` to `secondary`, so accent-on-surface
    // painted nothing visible.
    let fill = with_alpha(theme.colors.primary, SKELETON_PRIMARY_ALPHA);
    rsx! {
        row {
            width: props.width,
            height: props.height,
            background_color: fill,
            border_radius: radius,
            clip: true,
        }
    }
}
