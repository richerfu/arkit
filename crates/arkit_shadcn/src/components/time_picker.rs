//! Time Picker — an outline trigger backed by a bottom-sheet time selector.
//!
//! The component keeps the same mobile interaction model as [`super::DatePicker`]:
//! a compact trigger opens a bottom sheet, while the picker itself offers
//! independently scrollable hour and minute columns plus an optional AM/PM
//! column.

use std::fmt;

use super::{BottomSheet, Button, ButtonSize, ButtonVariant, ARKUI_BORDER_STYLE_SOLID};
use crate::i18n::{use_component_i18n, ComponentI18n};
use crate::icon::icon_placeholder;
use crate::theme::{spacing, typography, use_theme, Theme};
use arkit_prelude::*;

const TIME_PICKER_COLUMN_HEIGHT: f32 = 220.0;
const TIME_PICKER_OPTION_HEIGHT: f32 = 44.0;
const TIME_PICKER_CONTENT_INSET: f32 = spacing::SM;
const TIME_PICKER_DEFAULT_MINUTE_STEP: u8 = 1;

/// A validated wall-clock time.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TimeValue {
    hour: u8,
    minute: u8,
}

impl TimeValue {
    /// Midnight (`00:00`), used as the draft value when the picker is empty.
    pub const MIDNIGHT: Self = Self { hour: 0, minute: 0 };

    /// Creates a time when `hour < 24` and `minute < 60`.
    pub const fn new(hour: u8, minute: u8) -> Option<Self> {
        if hour < 24 && minute < 60 {
            Some(Self { hour, minute })
        } else {
            None
        }
    }

    /// Parses a strict `HH:mm` value.
    pub fn parse(value: &str) -> Option<Self> {
        if value.len() != 5 || value.as_bytes().get(2) != Some(&b':') {
            return None;
        }
        let hour = value.get(..2)?.parse().ok()?;
        let minute = value.get(3..)?.parse().ok()?;
        Self::new(hour, minute)
    }

    pub const fn hour(self) -> u8 {
        self.hour
    }

    pub const fn minute(self) -> u8 {
        self.minute
    }
}

impl fmt::Display for TimeValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:02}:{:02}", self.hour, self.minute)
    }
}

/// Clock notation used by [`TimePicker`].
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum TimePickerFormat {
    /// Displays `00:00` through `23:59`.
    #[default]
    TwentyFourHour,
    /// Displays `12:00 AM` through `11:59 PM`.
    TwelveHour,
}

/// User-facing text used by [`TimePicker`].
///
/// Supply this struct to override every string as one coherent locale snapshot.
/// When omitted, the active `arkit_i18n` locale is used.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimePickerLabels {
    pub placeholder: String,
    pub title: String,
    pub hour: String,
    pub minute: String,
    pub clear: String,
    pub confirm: String,
    pub am: String,
    pub pm: String,
}

impl TimePickerLabels {
    pub fn english() -> Self {
        Self {
            placeholder: "Pick a time".to_string(),
            title: "Select time".to_string(),
            hour: "Hour".to_string(),
            minute: "Minute".to_string(),
            clear: "Clear".to_string(),
            confirm: "Done".to_string(),
            am: "AM".to_string(),
            pm: "PM".to_string(),
        }
    }

    pub(crate) fn localized(i18n: ComponentI18n) -> Self {
        Self {
            placeholder: i18n.time_picker_placeholder(),
            title: i18n.time_picker_title(),
            hour: i18n.time_picker_hour(),
            minute: i18n.time_picker_minute(),
            clear: i18n.time_picker_clear(),
            confirm: i18n.time_picker_confirm(),
            am: i18n.time_picker_am(),
            pm: i18n.time_picker_pm(),
        }
    }
}

/// Props for [`TimePicker`].
#[derive(Props, Clone, PartialEq)]
pub struct TimePickerProps {
    /// Controlled selected time. When omitted, the picker owns its selection
    /// and starts from `default_selected`.
    pub selected: Option<TimeValue>,
    pub default_selected: Option<TimeValue>,
    /// Clock notation shown by both the trigger and selection sheet.
    #[props(default)]
    pub format: TimePickerFormat,
    /// Minute interval offered by the selector. Values are clamped to `1..=59`.
    /// A currently selected off-step minute remains available.
    pub minute_step: Option<u8>,
    /// Overrides all localized copy used by the picker.
    #[props(default)]
    pub labels: Option<TimePickerLabels>,
    /// Controlled sheet state.
    pub open: Option<bool>,
    #[props(default)]
    pub default_open: bool,
    #[props(default)]
    pub disabled: bool,
    #[props(default)]
    pub on_change: EventHandler<Option<TimeValue>>,
    #[props(default)]
    pub on_open_change: EventHandler<bool>,
}

/// A mobile time picker with 12/24-hour modes and configurable minute steps.
#[component]
pub fn TimePicker(props: TimePickerProps) -> Element {
    let theme = use_theme();
    let i18n = use_component_i18n();
    let mut internal_selected = use_signal(|| props.default_selected);
    let mut internal_open = use_signal(|| props.default_open);
    let open_controlled = props.open.is_some();
    let selected = props.selected.or_else(|| *internal_selected.read());
    let open = props.open.unwrap_or_else(|| *internal_open.read());
    let labels = props
        .labels
        .unwrap_or_else(|| TimePickerLabels::localized(i18n));
    let format = props.format;
    let minute_step = normalize_minute_step(props.minute_step);
    let mut draft = use_signal(|| selected.unwrap_or(TimeValue::MIDNIGHT));
    let on_change = props.on_change;
    let on_open_change = props.on_open_change;

    let set_open = EventHandler::new(move |next: bool| {
        if !open_controlled {
            internal_open.set(next);
        }
        on_open_change.call(next);
    });

    let trigger_label = selected
        .map(|value| format_time(value, format, &labels))
        .unwrap_or_else(|| labels.placeholder.clone());
    let trigger_color = if selected.is_some() {
        theme.colors.foreground
    } else {
        theme.colors.muted_foreground
    };
    let disabled = props.disabled;
    let hour_options = hour_options(format);
    let minute_options = minute_options(minute_step, draft().minute());
    let selected_hour = display_hour(draft(), format);
    let hour_offset = option_scroll_offset(&hour_options, selected_hour);
    let minute_offset = option_scroll_offset(&minute_options, draft().minute());
    let hour_label = labels.hour.clone();
    let minute_label = labels.minute.clone();
    let am_label = labels.am.clone();
    let pm_label = labels.pm.clone();
    let title = labels.title.clone();
    let clear_label = labels.clear.clone();
    let confirm_label = labels.confirm.clone();

    let select_hour = EventHandler::new(move |next_hour: u8| {
        let current = draft();
        let hour = match format {
            TimePickerFormat::TwentyFourHour => next_hour,
            TimePickerFormat::TwelveHour => {
                twelve_hour_to_twenty_four(next_hour, current.hour() >= 12)
            }
        };
        if let Some(next) = TimeValue::new(hour, current.minute()) {
            draft.set(next);
        }
    });
    let select_minute = EventHandler::new(move |minute: u8| {
        let current = draft();
        if let Some(next) = TimeValue::new(current.hour(), minute) {
            draft.set(next);
        }
    });
    let select_period = EventHandler::new(move |is_pm: bool| {
        let current = draft();
        let hour = if is_pm {
            current.hour() % 12 + 12
        } else {
            current.hour() % 12
        };
        if let Some(next) = TimeValue::new(hour, current.minute()) {
            draft.set(next);
        }
    });

    rsx! {
        Button {
            variant: ButtonVariant::Outline,
            disabled: Some(disabled),
            onclick: move |_| {
                let next_draft = selected.unwrap_or(TimeValue::MIDNIGHT);
                if draft() != next_draft {
                    draft.set(next_draft);
                }
                set_open.call(true);
            },
            row {
                align_items: "center",
                justify_content: "center",
                {icon_placeholder("clock", 21.0, trigger_color)}
                row { width: spacing::MD }
                text {
                    content: trigger_label,
                    font_size: typography::MD,
                    font_weight: 500_i32,
                    font_color: trigger_color,
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
                padding_right: TIME_PICKER_CONTENT_INSET,
                padding_left: TIME_PICKER_CONTENT_INSET,
                text {
                    content: title,
                    width: "100%",
                    font_size: typography::LG,
                    font_weight: 600_i32,
                    font_color: theme.colors.foreground,
                    text_align: "center",
                    line_height: 24.0,
                }
                row { height: spacing::LG }
                row {
                    width: "100%",
                    align_items: "start",
                    column {
                        layout_weight: 1.0,
                        {time_picker_column(
                            hour_label,
                            hour_options,
                            selected_hour,
                            hour_offset,
                            &theme,
                            select_hour,
                        )}
                    }
                    row { width: spacing::SM }
                    column {
                        layout_weight: 1.0,
                        {time_picker_column(
                            minute_label,
                            minute_options,
                            draft().minute(),
                            minute_offset,
                            &theme,
                            select_minute,
                        )}
                    }
                    if format == TimePickerFormat::TwelveHour {
                        row { width: spacing::SM }
                        column {
                            width: 88.0,
                            {period_column(
                                am_label,
                                pm_label,
                                draft().hour() >= 12,
                                &theme,
                                select_period,
                            )}
                        }
                    }
                }
                row { height: spacing::LG }
                row {
                    width: "100%",
                    Button {
                        size: ButtonSize::Sm,
                        variant: ButtonVariant::Outline,
                        width: "48%",
                        onclick: move |_| {
                            // Close first. A second state-driven render before
                            // BottomSheet observes `open = false` can otherwise
                            // leave a stale, visible overlay snapshot behind.
                            set_open.call(false);
                            internal_selected.set(None);
                            on_change.call(None);
                        },
                        {clear_label}
                    }
                    row { layout_weight: 1.0 }
                    Button {
                        size: ButtonSize::Sm,
                        width: "48%",
                        onclick: move |_| {
                            let next = Some(draft());
                            set_open.call(false);
                            internal_selected.set(next);
                            on_change.call(next);
                        },
                        {confirm_label}
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TimeOption {
    value: u8,
    label: String,
}

fn time_picker_column(
    label: String,
    options: Vec<TimeOption>,
    selected: u8,
    scroll_offset: f32,
    theme: &Theme,
    on_select: EventHandler<u8>,
) -> Element {
    let scroll_offset = format!("0,{scroll_offset}");

    rsx! {
        column {
            width: "100%",
            text {
                content: label,
                width: "100%",
                font_size: typography::XS,
                font_weight: 500_i32,
                font_color: theme.colors.muted_foreground,
                text_align: "center",
                line_height: 18.0,
            }
            row { height: spacing::XS }
            scroll {
                width: "100%",
                height: TIME_PICKER_COLUMN_HEIGHT,
                scroll_bar: "off",
                scroll_enabled: true,
                scroll_edge_effect: "spring",
                scroll_offset: scroll_offset,
                // BottomSheet handles downward drag gestures at its root.
                // Keep wheel scrolling local so a minute/hour swipe cannot
                // dismiss the entire sheet.
                on_touch: move |event| event.stop_propagation(),
                column {
                    width: "100%",
                    for option in options {
                        {
                            let active = option.value == selected;
                            let value = option.value;
                            rsx! {
                                button {
                                    width: "100%",
                                    height: TIME_PICKER_OPTION_HEIGHT,
                                    button_type: "normal",
                                    focusable: false,
                                    focus_on_touch: false,
                                    border_style: ARKUI_BORDER_STYLE_SOLID,
                                    border_width: if active { 1.0 } else { 0.0 },
                                    border_color: if active {
                                        theme.colors.primary
                                    } else {
                                        0x00000000
                                    },
                                    border_radius: theme.radii.md,
                                    background_color: if active {
                                        theme.colors.primary
                                    } else {
                                        0x00000000
                                    },
                                    font_size: typography::MD,
                                    font_weight: if active { 600_i32 } else { 400_i32 },
                                    font_color: if active {
                                        theme.colors.primary_foreground
                                    } else {
                                        theme.colors.foreground
                                    },
                                    foreground_color: if active {
                                        theme.colors.primary_foreground
                                    } else {
                                        theme.colors.foreground
                                    },
                                    onclick: move |_| on_select.call(value),
                                    {option.label}
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn period_column(
    am_label: String,
    pm_label: String,
    is_pm: bool,
    theme: &Theme,
    on_select: EventHandler<bool>,
) -> Element {
    rsx! {
        column {
            width: "100%",
            text {
                content: " ".to_string(),
                width: "100%",
                font_size: typography::XS,
                line_height: 18.0,
            }
            row { height: spacing::XS }
            column {
                width: "100%",
                height: TIME_PICKER_COLUMN_HEIGHT,
                justify_content: "center",
                {period_button(am_label, !is_pm, false, theme, on_select)}
                row { height: spacing::SM }
                {period_button(pm_label, is_pm, true, theme, on_select)}
            }
        }
    }
}

fn period_button(
    label: String,
    active: bool,
    is_pm: bool,
    theme: &Theme,
    on_select: EventHandler<bool>,
) -> Element {
    rsx! {
        button {
            width: "100%",
            height: TIME_PICKER_OPTION_HEIGHT,
            button_type: "normal",
            focusable: false,
            focus_on_touch: false,
            border_style: ARKUI_BORDER_STYLE_SOLID,
            border_width: if active { 1.0 } else { 0.0 },
            border_color: if active {
                theme.colors.primary
            } else {
                0x00000000
            },
            border_radius: theme.radii.md,
            background_color: if active {
                theme.colors.primary
            } else {
                0x00000000
            },
            font_size: typography::SM,
            font_weight: if active { 600_i32 } else { 400_i32 },
            font_color: if active {
                theme.colors.primary_foreground
            } else {
                theme.colors.foreground
            },
            foreground_color: if active {
                theme.colors.primary_foreground
            } else {
                theme.colors.foreground
            },
            onclick: move |_| on_select.call(is_pm),
            {label}
        }
    }
}

fn normalize_minute_step(step: Option<u8>) -> u8 {
    step.unwrap_or(TIME_PICKER_DEFAULT_MINUTE_STEP).clamp(1, 59)
}

fn hour_options(format: TimePickerFormat) -> Vec<TimeOption> {
    match format {
        TimePickerFormat::TwentyFourHour => (0..24)
            .map(|value| TimeOption {
                value,
                label: format!("{value:02}"),
            })
            .collect(),
        TimePickerFormat::TwelveHour => (1..=12)
            .map(|value| TimeOption {
                value,
                label: format!("{value:02}"),
            })
            .collect(),
    }
}

fn minute_options(step: u8, selected: u8) -> Vec<TimeOption> {
    let mut values = (0..60).step_by(usize::from(step)).collect::<Vec<_>>();
    if !values.contains(&selected) {
        values.push(selected);
        values.sort_unstable();
    }
    values
        .into_iter()
        .map(|value| TimeOption {
            value,
            label: format!("{value:02}"),
        })
        .collect()
}

fn display_hour(value: TimeValue, format: TimePickerFormat) -> u8 {
    match format {
        TimePickerFormat::TwentyFourHour => value.hour(),
        TimePickerFormat::TwelveHour => match value.hour() % 12 {
            0 => 12,
            hour => hour,
        },
    }
}

fn twelve_hour_to_twenty_four(hour: u8, is_pm: bool) -> u8 {
    if is_pm {
        hour % 12 + 12
    } else {
        hour % 12
    }
}

fn format_time(value: TimeValue, format: TimePickerFormat, labels: &TimePickerLabels) -> String {
    match format {
        TimePickerFormat::TwentyFourHour => value.to_string(),
        TimePickerFormat::TwelveHour => {
            let period = if value.hour() >= 12 {
                &labels.pm
            } else {
                &labels.am
            };
            format!(
                "{:02}:{:02} {period}",
                display_hour(value, format),
                value.minute()
            )
        }
    }
}

fn option_scroll_offset(options: &[TimeOption], selected: u8) -> f32 {
    let index = options
        .iter()
        .position(|option| option.value == selected)
        .unwrap_or(0);
    index.saturating_sub(2) as f32 * TIME_PICKER_OPTION_HEIGHT
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn time_value_validates_and_formats_hh_mm() {
        assert_eq!(
            TimeValue::new(23, 59).map(|value| value.to_string()),
            Some("23:59".into())
        );
        assert_eq!(TimeValue::parse("07:05"), TimeValue::new(7, 5));
        assert_eq!(TimeValue::parse("7:05"), None);
        assert_eq!(TimeValue::parse("24:00"), None);
        assert_eq!(TimeValue::new(12, 60), None);
    }

    #[test]
    fn twelve_hour_conversion_handles_midnight_and_noon() {
        assert_eq!(
            display_hour(TimeValue::MIDNIGHT, TimePickerFormat::TwelveHour),
            12
        );
        assert_eq!(
            display_hour(TimeValue::new(12, 0).unwrap(), TimePickerFormat::TwelveHour),
            12
        );
        assert_eq!(twelve_hour_to_twenty_four(12, false), 0);
        assert_eq!(twelve_hour_to_twenty_four(12, true), 12);
        assert_eq!(twelve_hour_to_twenty_four(3, true), 15);
    }

    #[test]
    fn minute_options_keep_an_off_step_selection() {
        let options = minute_options(15, 17);
        let values = options
            .into_iter()
            .map(|option| option.value)
            .collect::<Vec<_>>();

        assert_eq!(values, vec![0, 15, 17, 30, 45]);
        assert_eq!(normalize_minute_step(Some(0)), 1);
        assert_eq!(normalize_minute_step(Some(60)), 59);
    }

    #[test]
    fn twelve_hour_format_uses_localized_period_labels() {
        let labels = TimePickerLabels {
            am: "上午".to_string(),
            pm: "下午".to_string(),
            ..TimePickerLabels::english()
        };

        assert_eq!(
            format_time(
                TimeValue::new(0, 5).unwrap(),
                TimePickerFormat::TwelveHour,
                &labels
            ),
            "12:05 上午"
        );
        assert_eq!(
            format_time(
                TimeValue::new(15, 30).unwrap(),
                TimePickerFormat::TwelveHour,
                &labels
            ),
            "03:30 下午"
        );
    }
}
