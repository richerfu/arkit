//! Date picker — form-group trigger + ColorUI bottom-modal calendar.

use arkit_component::components::{
    Calendar, CalendarDate, CalendarDayPlugin, CalendarLabels, CalendarYearRange,
};
use arkit_prelude::*;

use super::chrome::{colorui_bottom_portal, dialog_fill, provide_close, CuBarFooter, CuBarHeader};
use super::primitives::Button;
use crate::spec;
use crate::theme::{swatch, use_colorui_theme};
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
    let theme = use_colorui_theme();
    let dark = matches!(theme.mode, arkit_component::style::ThemeMode::Dark);
    let fill = swatch(theme.primary).fill;
    let mut internal_selected = use_signal(|| default_selected.clone());
    let mut internal_open = use_signal(|| default_open);
    let open_controlled = open.is_some();
    let current_selected = selected
        .clone()
        .or_else(|| internal_selected.read().clone());
    let current_open = open.unwrap_or_else(|| *internal_open.read());
    let placeholder = placeholder.unwrap_or_else(|| "请选择日期".into());
    let close_label = close_label.unwrap_or_else(|| "确定".into());
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

    let panel = provide_close(
        EventHandler::new(move |_: ()| set_open.call(false)),
        rsx! {
            column {
                width: "100%",
                background_color: dialog_fill(dark),
                CuBarHeader {
                    title: "选择日期".to_string(),
                    show_close: Some(true),
                }
                column {
                    width: "100%",
                    padding_left: spec::PADDING,
                    padding_right: spec::PADDING,
                    Calendar {
                        selected: current_selected.clone(),
                        initial_month,
                        month: calendar_month,
                        year_range: calendar_year_range,
                        today: calendar_today,
                        labels: calendar_labels,
                        embedded: true,
                        day_plugin: calendar_day_plugin,
                        selection_color: Some(fill),
                        today_color: Some(fill),
                        on_month_change: move |month| on_calendar_month_change.call(month),
                        on_day_press: move |date| select_date.call(date),
                    }
                }
                CuBarFooter {
                    Button {
                        variant: ButtonVariant::Default,
                        onclick: move |_| set_open.call(false),
                        "{close_label}"
                    }
                }
            }
        },
    );

    rsx! {
        row {
            width: "100%",
            min_height: spec::LIST_ITEM,
            align_items: "center",
            justify_content: "space-between",
            background_color: spec::BG_WHITE,
            padding_left: spec::PADDING,
            padding_right: spec::PADDING,
            opacity: if disabled { 0.6 } else { 1.0 },
            onclick: move |_| {
                if !disabled {
                    set_open.call(true);
                }
            },
            text {
                content: label,
                font_size: spec::TEXT_DF,
                font_color: if current_selected.is_some() {
                    spec::TEXT
                } else {
                    spec::TEXT_MUTED
                },
            }
            {arkit_icon::icon("chevron-right", 16.0, spec::TEXT_GREY)}
        }
        {colorui_bottom_portal(current_open, panel, EventHandler::new(move |_: ()| set_open.call(false)))}
    }
}
