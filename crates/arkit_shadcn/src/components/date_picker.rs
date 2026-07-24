//! Date Picker — an outline trigger backed by a bottom-sheet calendar.
//!
//! React Native Reusables presents date selection as a compact outline button
//! with a calendar icon. Pressing it opens the shared month calendar in a
//! bottom sheet; pressing the selected day again clears the value. This keeps
//! the mobile interaction model instead of exposing ArkUI's inline wheel
//! picker, which is a different component.

use super::{BottomSheet, Button, ButtonSize, ButtonVariant, Calendar, CalendarLabels};
use crate::i18n::use_component_i18n;
use crate::icon::icon_placeholder;
use crate::theme::{spacing, typography, use_theme};
use arkit_prelude::*;

const DATE_PICKER_CONTENT_INSET: f32 = spacing::SM;

/// Props for [`DatePicker`].
#[derive(Props, Clone, PartialEq)]
pub struct DatePickerProps {
    /// Controlled selected date in `YYYY-MM-DD` form. When omitted, the picker
    /// owns its selection and starts from `default_selected`.
    pub selected: Option<String>,
    pub default_selected: Option<String>,
    /// Trigger text shown while no date is selected.
    pub placeholder: Option<String>,
    /// Label for the action that dismisses the calendar sheet.
    pub close_label: Option<String>,
    /// Localized month, weekday, and title text forwarded to [`Calendar`].
    /// When omitted, [`Calendar`] follows the active i18n locale.
    #[props(default)]
    pub calendar_labels: Option<CalendarLabels>,
    /// Controlled sheet state.
    pub open: Option<bool>,
    #[props(default)]
    pub default_open: bool,
    #[props(default)]
    pub disabled: bool,
    #[props(default)]
    pub on_change: EventHandler<Option<String>>,
    #[props(default)]
    pub on_open_change: EventHandler<bool>,
}

/// A mobile date picker matching the RNR outline-trigger/bottom-sheet flow.
#[component]
pub fn DatePicker(props: DatePickerProps) -> Element {
    let theme = use_theme();
    let i18n = use_component_i18n();
    let mut internal_selected = use_signal(|| props.default_selected.clone());
    let mut internal_open = use_signal(|| props.default_open);
    let open_controlled = props.open.is_some();
    let selected = props
        .selected
        .clone()
        .or_else(|| internal_selected.read().clone());
    let open = props.open.unwrap_or_else(|| *internal_open.read());
    let placeholder = props
        .placeholder
        .unwrap_or_else(|| i18n.date_picker_placeholder());
    let close_label = props
        .close_label
        .unwrap_or_else(|| i18n.date_picker_close());
    let calendar_labels = Some(
        props
            .calendar_labels
            .unwrap_or_else(|| CalendarLabels::localized(i18n)),
    );
    let label = selected.clone().unwrap_or(placeholder);
    let initial_month = selected
        .as_deref()
        .and_then(|date| date.get(..7))
        .map(ToOwned::to_owned);
    let disabled = props.disabled;
    let on_change = props.on_change;
    let on_open_change = props.on_open_change;

    let set_open = EventHandler::new(move |next: bool| {
        if !open_controlled {
            internal_open.set(next);
        }
        on_open_change.call(next);
    });

    let selected_for_press = selected.clone();
    let select_date = EventHandler::new(move |date: String| {
        let next = if selected_for_press.as_deref() == Some(date.as_str()) {
            None
        } else {
            Some(date)
        };
        // Keep the local mirror current even while externally controlled. If
        // a controller clears `selected` after a user press, falling back to
        // local state still reflects the same interaction rather than reviving
        // an older date.
        internal_selected.set(next.clone());
        on_change.call(next);
    });

    rsx! {
        Button {
            variant: ButtonVariant::Outline,
            disabled: Some(disabled),
            onclick: move |_| set_open.call(true),
            row {
                align_items: "center",
                justify_content: "center",
                {icon_placeholder("calendar", 21.0, theme.colors.foreground)}
                row { width: spacing::MD }
                text {
                    content: label,
                    font_size: typography::MD,
                    font_weight: 500_i32,
                    font_color: theme.colors.foreground,
                    line_height: 20.0,
                }
            }
        }
        BottomSheet {
            title: String::new(),
            open: Some(open),
            default_open: Some(false),
            show_header: Some(false),
            on_close: move |_| set_open.call(false),
            column {
                width: "100%",
                padding_right: DATE_PICKER_CONTENT_INSET,
                padding_left: DATE_PICKER_CONTENT_INSET,
                Calendar {
                    selected: selected.clone(),
                    initial_month,
                    labels: calendar_labels,
                    embedded: true,
                    on_day_press: move |date| select_date.call(date),
                }
                row { height: spacing::LG }
                Button {
                    size: ButtonSize::Sm,
                    width: "100%",
                    onclick: move |_| set_open.call(false),
                    {close_label}
                }
            }
        }
    }
}
