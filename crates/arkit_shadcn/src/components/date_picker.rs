use arkit_component::components::{
    Calendar, CalendarDate, CalendarDayPlugin, CalendarLabels, CalendarYearRange,
};
use arkit_prelude::*;

use super::overlays::BottomSheet;
use super::primitives::Button;
use crate::spec;
use crate::theme::use_theme;
use arkit_component::appearance::ButtonVariant;

#[component]
pub fn DatePicker(
    selected: Option<String>,
    default_selected: Option<String>,
    placeholder: Option<String>,
    close_label: Option<String>,
    #[props(default)] calendar_labels: Option<CalendarLabels>,
    #[props(default)] calendar_month: Option<String>,
    #[props(default)] calendar_year_range: Option<CalendarYearRange>,
    #[props(default)] calendar_today: Option<CalendarDate>,
    #[props(default)] calendar_day_plugin: Option<CalendarDayPlugin>,
    open: Option<bool>,
    #[props(default)] default_open: bool,
    #[props(default)] disabled: bool,
    #[props(default)] on_change: EventHandler<Option<String>>,
    #[props(default)] on_open_change: EventHandler<bool>,
    #[props(default)] on_calendar_month_change: EventHandler<String>,
) -> Element {
    let theme = use_theme();
    let mut internal_selected = use_signal(|| default_selected.clone());
    let mut internal_open = use_signal(|| default_open);
    let open_controlled = open.is_some();
    let current_selected = selected
        .clone()
        .or_else(|| internal_selected.read().clone());
    let current_open = open.unwrap_or_else(|| *internal_open.read());
    let placeholder = placeholder.unwrap_or_else(|| "Pick a date".into());
    let close_label = close_label.unwrap_or_else(|| "Done".into());
    let label = current_selected.clone().unwrap_or(placeholder);
    let initial_month = current_selected
        .as_deref()
        .and_then(|date| date.get(..7))
        .map(ToOwned::to_owned);
    let set_open = EventHandler::new(move |next: bool| {
        if !open_controlled {
            internal_open.set(next);
        }
        on_open_change.call(next);
    });
    let selected_for_press = current_selected.clone();
    let select_date = EventHandler::new(move |date: String| {
        let next = if selected_for_press.as_deref() == Some(date.as_str()) {
            None
        } else {
            Some(date)
        };
        internal_selected.set(next.clone());
        on_change.call(next);
    });
    rsx! {
        Button {
            variant: ButtonVariant::Outline,
            disabled: Some(disabled),
            onclick: move |_| {
                if !disabled {
                    set_open.call(true);
                }
            },
            row {
                align_items: "center",
                {arkit_icon::icon("calendar", 16.0, theme.colors.foreground)}
                row { width: 8.0 }
                text {
                    content: label,
                    font_size: spec::TEXT_SM,
                    font_weight: spec::FONT_MEDIUM,
                    font_color: theme.colors.foreground,
                }
            }
        }
        BottomSheet {
            title: "Date".to_string(),
            open: Some(current_open),
            default_open: Some(false),
            on_close: move |_| set_open.call(false),
            Calendar {
                selected: current_selected.clone(),
                initial_month,
                month: calendar_month,
                year_range: calendar_year_range,
                today: calendar_today,
                labels: calendar_labels,
                embedded: true,
                day_plugin: calendar_day_plugin,
                selection_color: Some(theme.colors.primary),
                today_color: Some(theme.colors.primary),
                on_month_change: move |month| on_calendar_month_change.call(month),
                on_day_press: move |date| select_date.call(date),
            }
            row { height: 16.0 }
            Button {
                width: "100%",
                onclick: move |_| set_open.call(false),
                "{close_label}"
            }
        }
    }
}
