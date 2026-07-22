//! AspectRatio — shadcn-style aspect-ratio container.
//!
//! Migrated from the original Elm builder API to dioxus 0.7 `#[component]` +
//! `rsx!`. Wraps a single child in a `stack` with the given aspect ratio.

use arkit_prelude::*;

/// Props for [`AspectRatio`].
#[derive(Props, Clone, PartialEq)]
pub struct AspectRatioProps {
    pub ratio: f32,
    pub children: Element,
}

/// A container that forces a child into a fixed aspect ratio.
#[component]
pub fn AspectRatio(props: AspectRatioProps) -> Element {
    rsx! {
        stack {
            width: "100%",
            aspect_ratio: props.ratio,
            {props.children}
        }
    }
}
