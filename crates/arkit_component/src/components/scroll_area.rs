//! ScrollArea — shadcn-style scroll container.
//!
//! Migrated from the original Elm builder API to dioxus 0.7 `#[component]` +
//! `rsx!`. Wraps the native ArkUI `Scroll` in a panel surface (`popover` fill,
//! 1px `border`, `md` radius) at full width, matching the legacy
//! `panel_surface(scroll_component(...))` styling. The `shadow-sm` from the
//! legacy `panel_surface` is not applied because the `Scroll` element does not
//! expose a `shadow` attribute in the dioxus bindings.

use crate::style::*;
use arkit_prelude::*;

/// Props for [`ScrollArea`].
#[derive(Props, Clone, PartialEq)]
pub struct ScrollAreaProps {
    pub children: Element,
    /// Scrollbar visibility. Maps to ArkUI `ScrollBarDisplayMode`:
    /// - `false` / omitted hide policy defaults to `"auto"` when `None`
    /// - Prefer explicit: `Some(false)` hide, `Some(true)` always on
    ///
    /// For full control (including auto), set `scroll_bar_mode` instead.
    #[props(default)]
    pub show_scroll_bar: Option<bool>,
    /// Explicit mode string forwarded to the `scroll` element: `"off"`,
    /// `"auto"`, or `"on"`. Takes precedence over [`Self::show_scroll_bar`].
    #[props(default)]
    pub scroll_bar_mode: Option<String>,
}

/// A scrollable panel surface.
#[component]
pub fn ScrollArea(props: ScrollAreaProps) -> Element {
    let theme = use_theme();
    let scroll_bar = props
        .scroll_bar_mode
        .clone()
        .or_else(|| {
            props
                .show_scroll_bar
                .map(|show| if show { "on" } else { "off" }.to_string())
        })
        .unwrap_or_else(|| "auto".to_string());

    rsx! {
        scroll {
            width: "100%",
            background_color: theme.colors.popover,
            border_width: 1.0,
            border_color: theme.colors.border,
            border_radius: theme.radii.md,
            scroll_bar: scroll_bar,
            {props.children}
        }
    }
}
