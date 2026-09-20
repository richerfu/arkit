//! AspectRatio — shadcn-style aspect-ratio container.
//!
//! Migrated from the original Elm builder API to dioxus 0.7 `#[component]` +
//! `rsx!`. Wraps a single child in a `stack` with the given aspect ratio.

use arkit_prelude::*;

/// Props for [`AspectRatio`].
#[derive(Props, Clone, PartialEq)]
pub struct AspectRatioProps {
    pub ratio: f32,
    /// Optional name for the visual region. Descendant controls remain
    /// independently reachable because the container is not collapsed.
    #[props(default)]
    pub accessibility_label: Option<String>,
    #[props(default)]
    pub accessibility_description: Option<String>,
    pub children: Element,
}

/// A container that forces a child into a fixed aspect ratio.
#[component]
pub fn AspectRatio(props: AspectRatioProps) -> Element {
    rsx! {
        stack {
            accessibility_role: if props.accessibility_label.is_some() { "group" },
            accessibility_text: if let Some(label) = props.accessibility_label { label },
            accessibility_description: if let Some(description) = props.accessibility_description { description },
            accessibility_group: false,
            width: "100%",
            aspect_ratio: props.ratio,
            {props.children}
        }
    }
}
