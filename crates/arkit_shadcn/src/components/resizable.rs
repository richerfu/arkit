//! Resizable — shadcn-style two-pane resizable container.
//!
//! Migrated from the original Elm builder API to dioxus 0.7 `#[component]` +
//! `rsx!`. Renders a left and right pane separated by a 1px vertical divider
//! (120px tall, `border` color) with `SM` gaps, mirroring the legacy
//! `resizable(left, right)` layout.

use crate::theme::*;
use arkit_prelude::*;

/// Props for [`Resizable`].
#[derive(Props, Clone, PartialEq)]
pub struct ResizableProps {
    /// Optional name for the split content region. Child panes retain their
    /// own semantics; the visual divider is excluded from the tree.
    #[props(default)]
    pub accessibility_label: Option<String>,
    pub left: Element,
    pub right: Element,
}

/// A two-pane container with a vertical divider.
#[component]
pub fn Resizable(props: ResizableProps) -> Element {
    let theme = use_theme();
    rsx! {
        row {
            accessibility_role: if props.accessibility_label.is_some() { "group" },
            accessibility_text: if let Some(label) = props.accessibility_label { label },
            accessibility_group: false,
            width: "100%",
            {props.left}
            row { width: spacing::SM }
            column {
                accessibility_mode: "disabled_for_descendants",
                width: 1.0,
                height: 120.0,
                background_color: theme.colors.border,
            }
            row { width: spacing::SM }
            {props.right}
        }
    }
}
