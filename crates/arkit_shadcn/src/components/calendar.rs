//! Calendar — shadcn-style calendar picker.
//!
//! Migrated from the original Elm builder API to dioxus 0.7 `#[component]` +
//! `rsx!`. Wraps the native ArkUI `CalendarPicker` in a panel surface
//! (`popover` fill, 1px `border`, `md` radius, `shadow-sm`) at 320px height,
//! matching the legacy `panel_surface(calendar_picker.height(320))`.

use crate::theme::*;
use arkit_prelude::*;

/// Props for [`Calendar`].
#[derive(Props, Clone, PartialEq)]
pub struct CalendarProps {
    pub selected: Option<String>,
}

/// A native calendar picker on a panel surface.
#[component]
pub fn Calendar(props: CalendarProps) -> Element {
    let theme = use_theme();
    rsx! {
        column {
            background_color: theme.colors.popover,
            border_width: 1.0,
            border_color: theme.colors.border,
            border_radius: theme.radii.md,
            shadow: 1,
            clip: true,
            calendar {
                calendar_selected: if let Some(s) = props.selected { s },
                height: 320.0,
            }
        }
    }
}
