//! Skeleton — unstyled loading placeholder.

use crate::appearance::{SkeletonAppearance, SkeletonStyleInput};
use crate::style::use_style_kit;
use arkit_prelude::*;

/// Props for [`Skeleton`].
#[derive(Props, Clone, PartialEq)]
pub struct SkeletonProps {
    pub width: f32,
    pub height: f32,
    #[props(default)]
    pub appearance: Option<SkeletonAppearance>,
}

/// A loading placeholder shaped like its eventual content.
#[component]
pub fn Skeleton(props: SkeletonProps) -> Element {
    let kit = use_style_kit();
    let appearance = props.appearance.unwrap_or_else(|| {
        kit.skeleton(&SkeletonStyleInput {
            width: props.width,
            height: props.height,
        })
    });
    rsx! {
        row {
            width: props.width,
            height: props.height,
            background_color: appearance.fill,
            border_radius: appearance.radius,
            clip: true,
        }
    }
}
