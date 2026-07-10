//! ScrollArea — shadcn-style scroll container.
//!
//! Migrated from the original Elm builder API to dioxus 0.7 `#[component]` +
//! `rsx!`. Wraps the native ArkUI `Scroll` in a panel surface (`popover` fill,
//! 1px `border`, `md` radius) at full width, matching the legacy
//! `panel_surface(scroll_component(...))` styling. The `shadow-sm` from the
//! legacy `panel_surface` is not applied because the `Scroll` element does not
//! expose a `shadow` attribute in the dioxus bindings.

use crate::theme::*;
use arkit_prelude::*;

/// Props for [`ScrollArea`].
#[derive(Props, Clone, PartialEq)]
pub struct ScrollAreaProps {
    pub children: Element,
}

/// A scrollable panel surface.
#[component]
pub fn ScrollArea(props: ScrollAreaProps) -> Element {
    let theme = use_theme();
    rsx! {
        scroll {
            percent_width: 1.0,
            background_color: theme.colors.popover,
            border_width: 1.0,
            border_color: theme.colors.border,
            border_radius: theme.radii.md,
            {props.children}
        }
    }
}
