//! Time picker — form-group trigger + ColorUI bottom list columns.

use arkit_component::components::{TimePickerFormat, TimePickerLabels, TimeValue};
use arkit_prelude::*;

use super::chrome::{colorui_bottom_portal, dialog_fill, provide_close, CuBarFooter, CuBarHeader};
use super::primitives::Button;
use crate::spec;
use crate::theme::use_colorui_theme;
use arkit_component::appearance::ButtonVariant;

#[component]
pub fn TimePicker(
    selected: Option<TimeValue>,
    default_selected: Option<TimeValue>,
    #[props(default)] format: TimePickerFormat,
    minute_step: Option<u8>,
    #[props(default)] labels: Option<TimePickerLabels>,
    open: Option<bool>,
    #[props(default)] default_open: bool,
    #[props(default)] disabled: bool,
    #[props(default)] on_change: EventHandler<Option<TimeValue>>,
    #[props(default)] on_open_change: EventHandler<bool>,
) -> Element {
    let theme = use_colorui_theme();
    let dark = matches!(theme.mode, arkit_component::style::ThemeMode::Dark);
    let mut internal_selected = use_signal(|| default_selected);
    let mut internal_open = use_signal(|| default_open);
    let open_controlled = open.is_some();
    let current = selected.or_else(|| *internal_selected.read());
    let current_open = open.unwrap_or_else(|| *internal_open.read());
    let labels = labels.unwrap_or_else(TimePickerLabels::english);
    let mut draft = use_signal(|| current.unwrap_or(TimeValue::MIDNIGHT));
    let step = minute_step.unwrap_or(1).clamp(1, 59);

    let set_open = EventHandler::new(move |next: bool| {
        if !open_controlled {
            internal_open.set(next);
        }
        on_open_change.call(next);
    });

    let trigger = current
        .map(|value| format!("{value}"))
        .unwrap_or_else(|| labels.placeholder.clone());

    let hours: Vec<u8> = match format {
        TimePickerFormat::TwentyFourHour => (0..24).collect(),
        TimePickerFormat::TwelveHour => (1..=12).collect(),
    };
    let minutes: Vec<u8> = (0..60).step_by(step as usize).map(|v| v as u8).collect();
    let draft_now = draft();
    let selected_hour = match format {
        TimePickerFormat::TwentyFourHour => draft_now.hour(),
        TimePickerFormat::TwelveHour => {
            let hour = draft_now.hour() % 12;
            if hour == 0 {
                12
            } else {
                hour
            }
        }
    };

    let hour_rows: Vec<Element> = hours
        .iter()
        .map(|hour| {
            let active = *hour == selected_hour;
            let value = *hour;
            rsx! {
                row {
                    width: "100%",
                    height: 40.0,
                    align_items: "center",
                    justify_content: "center",
                    background_color: if active { spec::LIGHT_GREEN } else { spec::BG_WHITE },
                    onclick: move |_| {
                        let current = draft();
                        let hour24 = match format {
                            TimePickerFormat::TwentyFourHour => value,
                            TimePickerFormat::TwelveHour => {
                                let pm = current.hour() >= 12;
                                let h = if value == 12 { 0 } else { value };
                                if pm { h + 12 } else { h }
                            }
                        };
                        if let Some(next) = TimeValue::new(hour24, current.minute()) {
                            draft.set(next);
                        }
                    },
                    text {
                        content: format!("{value:02}"),
                        font_size: spec::TEXT_DF,
                        font_color: if active { spec::BG_GREEN } else { spec::TEXT },
                    }
                }
            }
        })
        .collect();

    let minute_rows: Vec<Element> = minutes
        .iter()
        .map(|minute| {
            let active = *minute == draft_now.minute();
            let value = *minute;
            rsx! {
                row {
                    width: "100%",
                    height: 40.0,
                    align_items: "center",
                    justify_content: "center",
                    background_color: if active { spec::LIGHT_GREEN } else { spec::BG_WHITE },
                    onclick: move |_| {
                        let current = draft();
                        if let Some(next) = TimeValue::new(current.hour(), value) {
                            draft.set(next);
                        }
                    },
                    text {
                        content: format!("{value:02}"),
                        font_size: spec::TEXT_DF,
                        font_color: if active { spec::BG_GREEN } else { spec::TEXT },
                    }
                }
            }
        })
        .collect();

    let panel = provide_close(
        EventHandler::new(move |_: ()| set_open.call(false)),
        rsx! {
            column {
                width: "100%",
                background_color: dialog_fill(dark),
                CuBarHeader {
                    title: labels.title.clone(),
                    show_close: Some(true),
                }
                row {
                    width: "100%",
                    height: 220.0,
                    scroll {
                        layout_weight: 1.0,
                        height: 220.0,
                        column { {hour_rows.into_iter()} }
                    }
                    scroll {
                        layout_weight: 1.0,
                        height: 220.0,
                        column { {minute_rows.into_iter()} }
                    }
                }
                CuBarFooter {
                    Button {
                        variant: ButtonVariant::Outline,
                        onclick: move |_| {
                            set_open.call(false);
                            internal_selected.set(None);
                            on_change.call(None);
                        },
                        "{labels.clear}"
                    }
                    row { width: 8.0 }
                    Button {
                        onclick: move |_| {
                            let next = Some(draft());
                            set_open.call(false);
                            internal_selected.set(next);
                            on_change.call(next);
                        },
                        "{labels.confirm}"
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
                    draft.set(current.unwrap_or(TimeValue::MIDNIGHT));
                    set_open.call(true);
                }
            },
            text {
                content: trigger,
                font_size: spec::TEXT_DF,
                font_color: if current.is_some() { spec::TEXT } else { spec::TEXT_MUTED },
            }
            {arkit_icon::icon("chevron-right", 16.0, spec::TEXT_GREY)}
        }
        {colorui_bottom_portal(current_open, panel, EventHandler::new(move |_: ()| set_open.call(false)))}
    }
}
