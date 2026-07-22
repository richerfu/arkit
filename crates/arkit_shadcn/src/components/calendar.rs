//! Calendar — a shadcn-style month calendar.
//!
//! The React Native Reusables showcase uses `react-native-calendars`, not a
//! compact date input. This implementation renders the same month-view shape
//! directly with ArkUI primitives: month navigation, weekday headings, six
//! stable week rows, outside-month days, today highlighting, and controlled
//! single or multiple selections.

use std::time::{SystemTime, UNIX_EPOCH};

use crate::icon::icon_placeholder;
use crate::theme::{typography, use_theme, Theme, ThemeMode};
use arkit_prelude::*;

use super::ARKUI_BORDER_STYLE_SOLID;

const TRANSPARENT: u32 = 0x00000000;
const LIGHT_SELECTION: u32 = 0xFF0284C7;
const DARK_SELECTION: u32 = 0xFF0EA5E9;
const CALENDAR_PADDING: f32 = 12.0;
const DAY_SIZE: f32 = 36.0;
const WEEK_ROW_HEIGHT: f32 = 40.0;
const WEEKDAYS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

/// Props for [`Calendar`].
#[derive(Props, Clone, PartialEq)]
pub struct CalendarProps {
    /// Backwards-compatible single selected date in `YYYY-MM-DD` form.
    pub selected: Option<String>,
    /// Additional selected dates. Supplying this list enables controlled
    /// multi-selection without changing the component's interaction model.
    #[props(default)]
    pub selected_dates: Vec<String>,
    /// Initially visible month in `YYYY-MM` form. Defaults to the current
    /// month and is intentionally only read when the component mounts.
    pub initial_month: Option<String>,
    /// Selected-day fill. Defaults to the RNR sky-600/sky-500 colors.
    pub selection_color: Option<u32>,
    /// Today and navigation accent. Defaults to `selection_color`.
    pub today_color: Option<u32>,
    /// Removes the standalone card border, radius, and shadow when the
    /// calendar is embedded in another surface such as a bottom sheet.
    #[props(default)]
    pub embedded: bool,
    /// Called with a `YYYY-MM-DD` date for every enabled day press.
    #[props(default)]
    pub on_day_press: EventHandler<String>,
}

/// A full month calendar with controlled selection and internal navigation.
#[component]
pub fn Calendar(props: CalendarProps) -> Element {
    let theme = use_theme();
    let today = CalendarDate::today();
    let initial = props
        .initial_month
        .as_deref()
        .and_then(CalendarMonth::parse)
        .unwrap_or_else(|| today.month());
    let mut visible_month = use_signal(move || initial);
    let month = visible_month();

    let default_accent = match theme.mode {
        ThemeMode::Light => LIGHT_SELECTION,
        ThemeMode::Dark => DARK_SELECTION,
    };
    let selection_color = props.selection_color.unwrap_or(default_accent);
    let today_color = props.today_color.unwrap_or(selection_color);
    let embedded = props.embedded;
    let selected = props.selected.clone();
    let selected_dates = props.selected_dates.clone();
    let on_day_press = props.on_day_press;

    let previous = month.previous();
    let next = month.next();
    let title = month.title();
    let dates = month.grid_dates();
    let weeks = dates.chunks_exact(7).map(|week| {
        let cells = week.iter().copied().map(|date| {
            let date_string = date.to_string();
            let is_selected = selected.as_deref() == Some(date_string.as_str())
                || selected_dates.iter().any(|value| value == &date_string);
            let is_today = date == today;
            let is_outside = date.month() != month;
            let foreground = day_foreground(is_today, is_outside, today_color, &theme);
            let background = if is_selected {
                selection_color
            } else {
                TRANSPARENT
            };
            let selected_text = match theme.mode {
                ThemeMode::Light => 0xFFFFFFFF,
                ThemeMode::Dark => 0xFF000000,
            };
            let text_color = if is_selected {
                selected_text
            } else {
                foreground
            };
            let mut visible_month = visible_month;
            let pressed_date = date_string.clone();

            rsx! {
                row {
                    key: "{date_string}",
                    layout_weight: 1.0,
                    height: WEEK_ROW_HEIGHT,
                    align_items: "center",
                    justify_content: "center",
                    button {
                        button_type: "normal",
                        focusable: false,
                        focus_on_touch: false,
                        width: DAY_SIZE,
                        height: DAY_SIZE,
                        padding_top: 0.0,
                        padding_right: 0.0,
                        padding_bottom: 0.0,
                        padding_left: 0.0,
                        alignment: "center",
                        background_color: background,
                        border_style: ARKUI_BORDER_STYLE_SOLID,
                        border_width: 0.0,
                        border_color: TRANSPARENT,
                        border_radius: theme.radii.full,
                        opacity: if is_outside { 0.62 } else { 1.0 },
                        onclick: move |_| {
                            if is_outside {
                                visible_month.set(date.month());
                            }
                            on_day_press.call(pressed_date.clone());
                        },
                        text {
                            content: date.day.to_string(),
                            font_size: typography::SM,
                            font_weight: if is_selected || is_today { 600_i32 } else { 400_i32 },
                            font_color: text_color,
                            line_height: 20.0,
                        }
                    }
                }
            }
        });

        rsx! {
            row {
                width: "100%",
                height: WEEK_ROW_HEIGHT,
                align_items: "center",
                {cells}
            }
        }
    });

    rsx! {
        column {
            width: "100%",
            background_color: theme.colors.card,
            border_width: if embedded { 0.0 } else { 1.0 },
            border_color: theme.colors.border,
            border_style: ARKUI_BORDER_STYLE_SOLID,
            border_radius: if embedded { 0.0 } else { theme.radii.lg },
            shadow: if !embedded { "sm" },
            clip: true,
            padding_top: CALENDAR_PADDING,
            padding_right: CALENDAR_PADDING,
            padding_bottom: CALENDAR_PADDING,
            padding_left: CALENDAR_PADDING,
            row {
                width: "100%",
                height: 40.0,
                align_items: "center",
                CalendarNavigationButton {
                    icon: "chevron-left".to_string(),
                    accent: today_color,
                    onclick: move |_| visible_month.set(previous),
                }
                row {
                    layout_weight: 1.0,
                    align_items: "center",
                    justify_content: "center",
                    text {
                        content: title,
                        font_size: typography::MD,
                        font_weight: 500_i32,
                        font_color: theme.colors.card_foreground,
                        line_height: 24.0,
                    }
                }
                CalendarNavigationButton {
                    icon: "chevron-right".to_string(),
                    accent: today_color,
                    onclick: move |_| visible_month.set(next),
                }
            }
            row { height: 4.0 }
            row {
                width: "100%",
                height: 28.0,
                align_items: "center",
                for weekday in WEEKDAYS {
                    row {
                        key: "{weekday}",
                        layout_weight: 1.0,
                        align_items: "center",
                        justify_content: "center",
                        text {
                            content: weekday,
                            font_size: typography::XS,
                            font_weight: 500_i32,
                            font_color: theme.colors.muted_foreground,
                            line_height: 16.0,
                        }
                    }
                }
            }
            {weeks}
        }
    }
}

#[component]
fn CalendarNavigationButton(icon: String, accent: u32, onclick: EventHandler<()>) -> Element {
    rsx! {
        button {
            button_type: "normal",
            focusable: false,
            focus_on_touch: false,
            width: 36.0,
            height: 36.0,
            padding_top: 0.0,
            padding_right: 0.0,
            padding_bottom: 0.0,
            padding_left: 0.0,
            alignment: "center",
            background_color: TRANSPARENT,
            border_style: ARKUI_BORDER_STYLE_SOLID,
            border_width: 0.0,
            border_color: TRANSPARENT,
            onclick: move |_| onclick.call(()),
            {icon_placeholder(icon.as_str(), 18.0, accent)}
        }
    }
}

fn day_foreground(today: bool, outside: bool, today_color: u32, theme: &Theme) -> u32 {
    if today {
        today_color
    } else if outside {
        theme.colors.muted_foreground
    } else {
        theme.colors.card_foreground
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CalendarMonth {
    year: i32,
    month: u8,
}

impl CalendarMonth {
    fn parse(value: &str) -> Option<Self> {
        if value.len() != 7 || value.as_bytes().get(4) != Some(&b'-') {
            return None;
        }
        let year = value.get(..4)?.parse().ok()?;
        let month = value.get(5..)?.parse().ok()?;
        (1..=12).contains(&month).then_some(Self { year, month })
    }

    fn previous(self) -> Self {
        if self.month == 1 {
            Self {
                year: self.year - 1,
                month: 12,
            }
        } else {
            Self {
                year: self.year,
                month: self.month - 1,
            }
        }
    }

    fn next(self) -> Self {
        if self.month == 12 {
            Self {
                year: self.year + 1,
                month: 1,
            }
        } else {
            Self {
                year: self.year,
                month: self.month + 1,
            }
        }
    }

    fn title(self) -> String {
        format!("{} {}", MONTHS[usize::from(self.month - 1)], self.year)
    }

    fn grid_dates(self) -> [CalendarDate; 42] {
        let first = CalendarDate {
            year: self.year,
            month: self.month,
            day: 1,
        };
        let grid_start = first.days_since_epoch() - i64::from(first.weekday());
        std::array::from_fn(|index| CalendarDate::from_days(grid_start + index as i64))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CalendarDate {
    year: i32,
    month: u8,
    day: u8,
}

impl CalendarDate {
    fn today() -> Self {
        let days = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| (duration.as_secs() / 86_400) as i64)
            .unwrap_or_default();
        Self::from_days(days)
    }

    fn month(self) -> CalendarMonth {
        CalendarMonth {
            year: self.year,
            month: self.month,
        }
    }

    fn weekday(self) -> u8 {
        (self.days_since_epoch() + 4).rem_euclid(7) as u8
    }

    fn days_since_epoch(self) -> i64 {
        days_from_civil(self.year, self.month, self.day)
    }

    fn from_days(days: i64) -> Self {
        civil_from_days(days)
    }
}

impl std::fmt::Display for CalendarDate {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "{:04}-{:02}-{:02}",
            self.year, self.month, self.day
        )
    }
}

// Howard Hinnant's proleptic-Gregorian civil date algorithms. Keeping these
// pure and local avoids adding a date dependency to every shadcn consumer.
fn days_from_civil(year: i32, month: u8, day: u8) -> i64 {
    let mut year = i64::from(year);
    let month = i64::from(month);
    let day = i64::from(day);
    year -= i64::from(month <= 2);
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = year - era * 400;
    let month_prime = month + if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * month_prime + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

fn civil_from_days(days: i64) -> CalendarDate {
    let days = days + 719_468;
    let era = if days >= 0 { days } else { days - 146_096 } / 146_097;
    let day_of_era = days - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    year += i64::from(month <= 2);

    CalendarDate {
        year: year as i32,
        month: month as u8,
        day: day as u8,
    }
}
