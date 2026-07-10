//! DatePicker — shadcn-style date picker.
//!
//! Migrated from the original Elm builder API to dioxus 0.7 `#[component]` +
//! `rsx!`. Wraps the native ArkUI `DatePicker` in an input surface
//! (`background` fill, 1px `input` border, `md` radius, `[0, 12, 0, 12]`
//! padding), matching the legacy `input_surface(date_picker)`. The surface is
//! applied to a wrapping row because the native `DatePicker` element does not
//! expose per-side padding.

use super::ARKUI_BORDER_STYLE_SOLID;
use crate::theme::*;
use arkit_prelude::*;

/// Props for [`DatePicker`].
#[derive(Props, Clone, PartialEq)]
pub struct DatePickerProps {
    pub selected: Option<String>,
}

/// A native date picker on an input surface.
#[component]
pub fn DatePicker(props: DatePickerProps) -> Element {
    let theme = use_theme();
    rsx! {
        row {
            background_color: theme.colors.background,
            border_style: ARKUI_BORDER_STYLE_SOLID,
            border_width: 1.0,
            border_color: theme.colors.input,
            border_radius: theme.radii.md,
            padding_top: 0.0,
            padding_right: 12.0,
            padding_bottom: 0.0,
            padding_left: 12.0,
            datepicker {
                datepicker_selected: if let Some(s) = props.selected { s },
            }
        }
    }
}
