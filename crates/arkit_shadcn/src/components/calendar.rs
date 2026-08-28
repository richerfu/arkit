//! Calendar — a shadcn-style month calendar.
//!
//! The React Native Reusables showcase uses `react-native-calendars`, not a
//! compact date input. This implementation renders the same month-view shape
//! directly with ArkUI primitives: month/year quick navigation, weekday
//! headings, six stable week rows, outside-month days, today highlighting,
//! controlled single or multiple selections, and composable presentation and
//! interaction plugins.

use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::i18n::use_component_i18n;
use crate::icon::icon_placeholder;
use crate::theme::{spacing, typography, use_theme, Theme, ThemeMode};
use arkit_prelude::*;

use super::calendar_plugin::*;
use super::ARKUI_BORDER_STYLE_SOLID;

const TRANSPARENT: u32 = 0x00000000;
const LIGHT_SELECTION: u32 = 0xFF0284C7;
const DARK_SELECTION: u32 = 0xFF0EA5E9;
const CALENDAR_PADDING: f32 = 12.0;
const DAY_SIZE: f32 = 36.0;
const WEEK_ROW_HEIGHT: f32 = 40.0;
const PICKER_COLUMN_COUNT: usize = 3;
const PICKER_ROW_COUNT: usize = 4;
const PICKER_BACK_ROW_HEIGHT: f32 = 44.0;
const YEAR_PAGE_SIZE: i32 = (PICKER_COLUMN_COUNT * PICKER_ROW_COUNT) as i32;
const MIN_CALENDAR_YEAR: i32 = 1;
const MAX_CALENDAR_YEAR: i32 = 9_999;

/// Inclusive Gregorian year range available to calendar navigation and the
/// quick year picker. Bounds are normalized and limited to the four-digit
/// range supported by the Calendar's `YYYY-MM` string API.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CalendarYearRange {
    start: i32,
    end: i32,
}

impl CalendarYearRange {
    pub const fn new(start: i32, end: i32) -> Self {
        let (start, end) = if start <= end {
            (start, end)
        } else {
            (end, start)
        };
        let start = if start < MIN_CALENDAR_YEAR {
            MIN_CALENDAR_YEAR
        } else if start > MAX_CALENDAR_YEAR {
            MAX_CALENDAR_YEAR
        } else {
            start
        };
        let end = if end < MIN_CALENDAR_YEAR {
            MIN_CALENDAR_YEAR
        } else if end > MAX_CALENDAR_YEAR {
            MAX_CALENDAR_YEAR
        } else {
            end
        };
        Self { start, end }
    }

    pub const fn start(self) -> i32 {
        self.start
    }

    pub const fn end(self) -> i32 {
        self.end
    }

    pub const fn contains(self, year: i32) -> bool {
        year >= self.start && year <= self.end
    }

    fn clamp(self, year: i32) -> i32 {
        year.clamp(self.start, self.end)
    }

    fn page_start(self, year: i32) -> i32 {
        let year = self.clamp(year);
        self.start + (year - self.start).div_euclid(YEAR_PAGE_SIZE) * YEAR_PAGE_SIZE
    }
}

impl Default for CalendarYearRange {
    fn default() -> Self {
        Self::new(MIN_CALENDAR_YEAR, MAX_CALENDAR_YEAR)
    }
}

/// A validated proleptic-Gregorian civil date.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CalendarDate {
    year: i32,
    month: u8,
    day: u8,
}

impl CalendarDate {
    pub fn new(year: i32, month: u8, day: u8) -> Option<Self> {
        if !(1..=12).contains(&month) || day == 0 || day > days_in_month(year, month) {
            return None;
        }
        Some(Self { year, month, day })
    }

    pub const fn year(self) -> i32 {
        self.year
    }

    pub const fn month_number(self) -> u8 {
        self.month
    }

    pub const fn day(self) -> u8 {
        self.day
    }

    fn today_utc() -> Self {
        let days = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| (duration.as_secs() / 86_400) as i64)
            .unwrap_or_default();
        Self::from_days(days)
    }

    fn calendar_month(self) -> CalendarMonth {
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

/// User-visible calendar copy.
///
/// `month_title_template` replaces `{month}` with the matching entry from
/// `months` and `{year}` with the numeric year.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CalendarLabels {
    pub weekdays: [String; 7],
    pub months: [String; 12],
    pub month_title_template: String,
    pub back_to_days: String,
}

impl CalendarLabels {
    pub fn new(
        weekdays: [String; 7],
        months: [String; 12],
        month_title_template: impl Into<String>,
    ) -> Self {
        Self {
            weekdays,
            months,
            month_title_template: month_title_template.into(),
            back_to_days: "Back to dates".to_owned(),
        }
    }

    pub fn with_back_to_days(mut self, label: impl Into<String>) -> Self {
        self.back_to_days = label.into();
        self
    }

    /// English labels matching the original component presentation.
    pub fn english() -> Self {
        Self::new(
            ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"].map(str::to_owned),
            [
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
            ]
            .map(str::to_owned),
            "{month} {year}",
        )
    }

    pub(crate) fn localized(i18n: crate::i18n::ComponentI18n) -> Self {
        Self::new(
            i18n.calendar_weekdays(),
            i18n.calendar_months(),
            i18n.calendar_month_title_template(),
        )
        .with_back_to_days(i18n.calendar_back_to_days())
    }

    fn month_title(&self, year: i32, month: u8) -> String {
        self.month_title_template
            .replace("{year}", &year.to_string())
            .replace("{month}", &self.months[usize::from(month - 1)])
    }
}

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
    /// Controlled visible month in `YYYY-MM` form.
    #[props(default)]
    pub month: Option<String>,
    /// Inclusive range used by arrows and the quick year picker.
    #[props(default)]
    pub year_range: Option<CalendarYearRange>,
    /// Explicit civil date used for the "today" state. Supplying this avoids
    /// making the component itself responsible for application time zones.
    #[props(default)]
    pub today: Option<CalendarDate>,
    /// Month names, weekday headings, and month-title formatting. When
    /// omitted, the active i18n locale selects the built-in labels.
    #[props(default)]
    pub labels: Option<CalendarLabels>,
    /// Selected-day fill. Defaults to the RNR sky-600/sky-500 colors.
    pub selection_color: Option<u32>,
    /// Today and navigation accent. Defaults to `selection_color`.
    pub today_color: Option<u32>,
    /// Removes the standalone card border, radius, and shadow when the
    /// calendar is embedded in another surface such as a bottom sheet.
    #[props(default)]
    pub embedded: bool,
    /// Ordered presentation and interaction plugins.
    ///
    /// Supporting and overlay content is additive. Style and replacement
    /// conflicts resolve in declaration order, with later plugins winning.
    #[props(default)]
    pub plugins: Vec<CalendarPlugin>,
    /// Called with a `YYYY-MM-DD` date for every enabled day press.
    #[props(default)]
    pub on_day_press: EventHandler<String>,
    /// Called whenever arrows or quick pickers request a visible month change.
    #[props(default)]
    pub on_month_change: EventHandler<String>,
}

/// A full month calendar with controlled selection and internal navigation.
#[component]
pub fn Calendar(props: CalendarProps) -> Element {
    let theme = use_theme();
    let i18n = use_component_i18n();
    let today = props.today.unwrap_or_else(CalendarDate::today_utc);
    let year_range = props.year_range.unwrap_or_default();
    let initial = props
        .initial_month
        .as_deref()
        .and_then(CalendarMonth::parse)
        .unwrap_or_else(|| today.calendar_month())
        .clamp(year_range);
    let mut internal_month = use_signal(move || initial);
    let controlled_month = props
        .month
        .as_deref()
        .and_then(CalendarMonth::parse)
        .map(|month| month.clamp(year_range));
    let month_controlled = controlled_month.is_some();
    let month = controlled_month
        .unwrap_or(internal_month())
        .clamp(year_range);
    let mut view = use_signal(|| CalendarView::Days);
    let mut picker_year = use_signal(move || initial.year);
    let mut year_page = use_signal(move || year_range.page_start(initial.year));

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
    let on_month_change = props.on_month_change;
    let plugins: Rc<[CalendarPlugin]> = props.plugins.into();
    let (day_size, week_row_height) = resolve_plugin_layout(&plugins);
    let picker_height = 28.0 + week_row_height * 6.0;
    let picker_grid_height = picker_height - PICKER_BACK_ROW_HEIGHT;
    let labels = props
        .labels
        .unwrap_or_else(|| CalendarLabels::localized(i18n));
    let weekday_labels = labels.weekdays.clone();
    let month_labels = labels.months.clone();
    let back_to_days_label = labels.back_to_days.clone();
    let current_view = view();
    let set_month = EventHandler::new(move |next: CalendarMonth| {
        let next = next.clamp(year_range);
        if !month_controlled {
            internal_month.set(next);
        }
        on_month_change.call(next.to_string());
    });
    let first_year_page = year_range.page_start(year_range.start);
    let last_year_page = year_range.page_start(year_range.end);
    let previous_disabled = match current_view {
        CalendarView::Days => month.previous().year < year_range.start,
        CalendarView::Months => picker_year() <= year_range.start,
        CalendarView::Years => year_page() <= first_year_page,
    };
    let next_disabled = match current_view {
        CalendarView::Days => month.next().year > year_range.end,
        CalendarView::Months => picker_year() >= year_range.end,
        CalendarView::Years => year_page() >= last_year_page,
    };
    let navigate_previous = EventHandler::new(move |_: ()| match current_view {
        CalendarView::Days => {
            let previous = month.previous();
            if year_range.contains(previous.year) {
                set_month.call(previous);
            }
        }
        CalendarView::Months => {
            picker_year.set(year_range.clamp(picker_year() - 1));
        }
        CalendarView::Years => {
            year_page.set((year_page() - YEAR_PAGE_SIZE).max(first_year_page));
        }
    });
    let navigate_next = EventHandler::new(move |_: ()| match current_view {
        CalendarView::Days => {
            let next = month.next();
            if year_range.contains(next.year) {
                set_month.call(next);
            }
        }
        CalendarView::Months => {
            picker_year.set(year_range.clamp(picker_year() + 1));
        }
        CalendarView::Years => {
            year_page.set((year_page() + YEAR_PAGE_SIZE).min(last_year_page));
        }
    });
    let default_header_title = match current_view {
        CalendarView::Days => labels.month_title(month.year, month.month),
        CalendarView::Months => picker_year().to_string(),
        CalendarView::Years => {
            let start = year_page();
            let end = (start + YEAR_PAGE_SIZE - 1).min(year_range.end);
            format!("{start}–{end}")
        }
    };
    let (header_title, month_supporting_content) = if current_view == CalendarView::Days {
        resolve_month_plugins(month, default_header_title, &plugins)
    } else {
        (default_header_title, Vec::new())
    };
    let toggle_picker = EventHandler::new(move |_: ()| match current_view {
        CalendarView::Days => {
            picker_year.set(month.year);
            view.set(CalendarView::Months);
        }
        CalendarView::Months => {
            year_page.set(year_range.page_start(picker_year()));
            view.set(CalendarView::Years);
        }
        CalendarView::Years => view.set(CalendarView::Months),
    });
    let back_to_days = EventHandler::new(move |_: ()| view.set(CalendarView::Days));
    let select_month = EventHandler::new(move |selected_month: u8| {
        set_month.call(CalendarMonth {
            year: picker_year(),
            month: selected_month,
        });
        view.set(CalendarView::Days);
    });
    let select_year = EventHandler::new(move |selected_year: i32| {
        picker_year.set(selected_year);
        view.set(CalendarView::Months);
    });
    let visible_years = (year_page()..year_page() + YEAR_PAGE_SIZE)
        .filter(|year| year_range.contains(*year))
        .collect::<Vec<_>>();

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
                    disabled: previous_disabled,
                    onclick: move |_| navigate_previous.call(()),
                }
                row {
                    layout_weight: 1.0,
                    CalendarHeaderButton {
                        label: header_title,
                        expanded: current_view != CalendarView::Days,
                        onclick: move |_| toggle_picker.call(()),
                    }
                }
                CalendarNavigationButton {
                    icon: "chevron-right".to_string(),
                    accent: today_color,
                    disabled: next_disabled,
                    onclick: move |_| navigate_next.call(()),
                }
            }
            for (plugin_index, content) in month_supporting_content.into_iter().enumerate() {
                row {
                    key: "month-plugin-{plugin_index}",
                    width: "100%",
                    hit_test_behavior: "transparent",
                    {content}
                }
            }
            row { height: 4.0 }
            if current_view == CalendarView::Days {
                row {
                    width: "100%",
                    height: 28.0,
                    align_items: "center",
                    for (weekday_index, weekday) in weekday_labels.into_iter().enumerate() {
                        row {
                            key: "{weekday_index}",
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
                CalendarDays {
                    month,
                    today,
                    year_range,
                    selected,
                    selected_dates,
                    selection_color,
                    today_color,
                    day_size,
                    week_row_height,
                    plugins: plugins.clone(),
                    on_visible_month: move |next| set_month.call(next),
                    on_day_press: move |date| on_day_press.call(date),
                }
            } else if current_view == CalendarView::Months {
                column {
                    width: "100%",
                    height: picker_height,
                    CalendarMonthGrid {
                        labels: month_labels,
                        active_month: if picker_year() == month.year { Some(month.month) } else { None },
                        height: picker_grid_height,
                        on_select: move |selected| select_month.call(selected),
                    }
                    CalendarPickerBackButton {
                        label: back_to_days_label,
                        onclick: move |_| back_to_days.call(()),
                    }
                }
            } else {
                column {
                    width: "100%",
                    height: picker_height,
                    CalendarYearGrid {
                        years: visible_years,
                        active_year: month.year,
                        height: picker_grid_height,
                        on_select: move |selected| select_year.call(selected),
                    }
                    CalendarPickerBackButton {
                        label: back_to_days_label,
                        onclick: move |_| back_to_days.call(()),
                    }
                }
            }
        }
    }
}

fn resolve_plugin_layout(plugins: &[CalendarPlugin]) -> (f32, f32) {
    plugins.iter().fold(
        (DAY_SIZE, WEEK_ROW_HEIGHT),
        |(day_size, row_height), plugin| {
            let layout = plugin.layout();
            (
                layout.minimum_day_size.unwrap_or(day_size).max(day_size),
                layout
                    .minimum_week_row_height
                    .unwrap_or(row_height)
                    .max(row_height),
            )
        },
    )
}

fn resolve_month_plugins(
    month: CalendarMonth,
    title: String,
    plugins: &[CalendarPlugin],
) -> (String, Vec<Element>) {
    let dates = month.grid_dates();
    let mut resolved_title = title;
    let mut supporting_content = Vec::new();

    for plugin in plugins {
        let decoration = plugin.decorate_month(CalendarMonthContext {
            year: month.year,
            month: month.month,
            first_visible_date: dates[0],
            last_visible_date: dates[dates.len() - 1],
            title: resolved_title.clone(),
        });
        if let Some(title) = decoration.title {
            resolved_title = title;
        }
        if let Some(content) = decoration.supporting {
            supporting_content.push(content);
        }
    }

    (resolved_title, supporting_content)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CalendarView {
    Days,
    Months,
    Years,
}

#[component]
fn CalendarHeaderButton(label: String, expanded: bool, onclick: EventHandler<()>) -> Element {
    let theme = use_theme();
    rsx! {
        button {
            button_type: "normal",
            focusable: false,
            focus_on_touch: false,
            width: "100%",
            height: 36.0,
            padding_top: 0.0,
            padding_right: spacing::SM,
            padding_bottom: 0.0,
            padding_left: spacing::SM,
            alignment: "center",
            background_color: TRANSPARENT,
            border_style: ARKUI_BORDER_STYLE_SOLID,
            border_width: 0.0,
            border_color: TRANSPARENT,
            onclick: move |_| onclick.call(()),
            row {
                align_items: "center",
                justify_content: "center",
                text {
                    content: label,
                    font_size: typography::MD,
                    font_weight: 500_i32,
                    font_color: theme.colors.card_foreground,
                    line_height: 24.0,
                }
                row { width: spacing::XXS }
                {icon_placeholder(
                    if expanded { "chevron-up" } else { "chevron-down" },
                    14.0,
                    theme.colors.muted_foreground,
                )}
            }
        }
    }
}

#[component]
fn CalendarPickerBackButton(label: String, onclick: EventHandler<()>) -> Element {
    let theme = use_theme();
    rsx! {
        row {
            width: "100%",
            height: PICKER_BACK_ROW_HEIGHT,
            align_items: "center",
            justify_content: "center",
            button {
                button_type: "normal",
                focusable: false,
                focus_on_touch: false,
                height: 36.0,
                padding_top: 0.0,
                padding_right: spacing::MD,
                padding_bottom: 0.0,
                padding_left: spacing::MD,
                alignment: "center",
                background_color: TRANSPARENT,
                border_style: ARKUI_BORDER_STYLE_SOLID,
                border_width: 1.0,
                border_color: theme.colors.border,
                border_radius: theme.radii.md,
                onclick: move |_| onclick.call(()),
                row {
                    align_items: "center",
                    justify_content: "center",
                    {icon_placeholder("calendar", 15.0, theme.colors.muted_foreground)}
                    row { width: spacing::XS }
                    text {
                        content: label,
                        font_size: typography::SM,
                        font_weight: 500_i32,
                        font_color: theme.colors.card_foreground,
                        line_height: 20.0,
                    }
                }
            }
        }
    }
}

#[component]
fn CalendarNavigationButton(
    icon: String,
    accent: u32,
    disabled: bool,
    onclick: EventHandler<()>,
) -> Element {
    rsx! {
        button {
            button_type: "normal",
            focusable: false,
            focus_on_touch: false,
            enabled: !disabled,
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
            opacity: if disabled { 0.36 } else { 1.0 },
            onclick: move |_| onclick.call(()),
            {icon_placeholder(icon.as_str(), 18.0, accent)}
        }
    }
}

#[component]
fn CalendarDays(
    month: CalendarMonth,
    today: CalendarDate,
    year_range: CalendarYearRange,
    selected: Option<String>,
    selected_dates: Vec<String>,
    selection_color: u32,
    today_color: u32,
    day_size: f32,
    week_row_height: f32,
    plugins: Rc<[CalendarPlugin]>,
    on_visible_month: EventHandler<CalendarMonth>,
    on_day_press: EventHandler<String>,
) -> Element {
    let theme = use_theme();
    let extended_layout = day_size > DAY_SIZE || week_row_height > WEEK_ROW_HEIGHT;
    let selected_text = match theme.mode {
        ThemeMode::Light => 0xFFFFFFFF,
        ThemeMode::Dark => 0xFF000000,
    };
    let dates = month.grid_dates();
    let weeks = dates.chunks_exact(7).map(|week| {
        let cells = week.iter().copied().map(|date| {
            let date_string = date.to_string();
            let is_selected = selected.as_deref() == Some(date_string.as_str())
                || selected_dates.iter().any(|value| value == &date_string);
            let is_today = date == today;
            let is_outside = date.calendar_month() != month;
            let enabled = year_range.contains(date.year());
            let foreground = day_foreground(is_today, is_outside, today_color, &theme);
            let background = if is_selected {
                selection_color
            } else {
                TRANSPARENT
            };
            let text_color = if is_selected {
                selected_text
            } else {
                foreground
            };
            let supporting_color = if is_selected {
                selected_text
            } else {
                theme.colors.muted_foreground
            };
            let decoration = resolve_plugins_for_day(
                CalendarDayContext {
                    date,
                    selected: is_selected,
                    today: is_today,
                    outside_month: is_outside,
                    enabled,
                    primary_color: text_color,
                    background_color: background,
                    supporting_color,
                },
                &plugins,
            );
            let context = decoration.context;
            let enabled = context.enabled;
            let border_color = decoration.style.border_color.unwrap_or(TRANSPARENT);
            let border_width = decoration.style.border_width.unwrap_or(0.0).max(0.0);
            let border_radius = decoration
                .style
                .border_radius
                .unwrap_or(theme.radii.full)
                .max(0.0);
            let default_opacity = if !enabled {
                0.36
            } else if is_outside {
                0.62
            } else {
                1.0
            };
            let mut opacity = decoration
                .style
                .opacity
                .unwrap_or(default_opacity)
                .clamp(0.0, 1.0);
            if !enabled {
                opacity = opacity.min(0.36);
            }
            let primary_font_weight = decoration.style.primary_font_weight.unwrap_or(
                if is_selected || is_today {
                    600_i32
                } else {
                    400_i32
                },
            );
            let supporting_content = decoration.supporting;
            let overlay_content = decoration.overlays;
            let replacement_content = decoration.replacement;
            let pressed_date = date_string.clone();
            let press_plugins = plugins.clone();
            let long_press_plugins = plugins.clone();

            rsx! {
                row {
                    key: "{date_string}",
                    layout_weight: 1.0,
                    height: week_row_height,
                    align_items: "center",
                    justify_content: "center",
                    button {
                        button_type: "normal",
                        focusable: false,
                        focus_on_touch: false,
                        enabled,
                        width: day_size,
                        height: day_size,
                        padding_top: 0.0,
                        padding_right: 0.0,
                        padding_bottom: 0.0,
                        padding_left: 0.0,
                        alignment: "center",
                        background_color: context.background_color,
                        border_style: ARKUI_BORDER_STYLE_SOLID,
                        border_width,
                        border_color,
                        border_radius,
                        opacity,
                        onclick: move |_| {
                            if enabled {
                                let response = dispatch_plugins_for_day(
                                    &press_plugins,
                                    CalendarDayEvent {
                                        context,
                                        kind: CalendarDayEventKind::Press,
                                    },
                                );
                                if !response.prevent_default {
                                    if is_outside {
                                        on_visible_month.call(date.calendar_month());
                                    }
                                    on_day_press.call(pressed_date.clone());
                                }
                            }
                        },
                        onlongpress: move |_| {
                            if enabled {
                                dispatch_plugins_for_day(
                                    &long_press_plugins,
                                    CalendarDayEvent {
                                        context,
                                        kind: CalendarDayEventKind::LongPress,
                                    },
                                );
                            }
                        },
                        stack {
                            width: "100%",
                            height: "100%",
                            hit_test_behavior: "none",
                            if let Some(content) = replacement_content {
                                row {
                                    width: "100%",
                                    height: "100%",
                                    align_items: "center",
                                    justify_content: "center",
                                    hit_test_behavior: "none",
                                    {content}
                                }
                            } else {
                                column {
                                    width: "100%",
                                    height: "100%",
                                    align_items: "center",
                                    justify_content: "center",
                                    hit_test_behavior: "none",
                                    text {
                                        content: date.day().to_string(),
                                        font_size: typography::SM,
                                        font_weight: primary_font_weight,
                                        font_color: context.primary_color,
                                        line_height: if extended_layout { 18.0 } else { 20.0 },
                                    }
                                    if !supporting_content.is_empty() {
                                        row {
                                            align_items: "center",
                                            justify_content: "center",
                                            hit_test_behavior: "none",
                                            for (plugin_index, content) in supporting_content.into_iter().enumerate() {
                                                row {
                                                    key: "day-supporting-{plugin_index}",
                                                    align_items: "center",
                                                    justify_content: "center",
                                                    hit_test_behavior: "none",
                                                    {content}
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            for (plugin_index, content) in overlay_content.into_iter().enumerate() {
                                row {
                                    key: "day-overlay-{plugin_index}",
                                    width: "100%",
                                    height: "100%",
                                    align_items: "center",
                                    justify_content: "center",
                                    hit_test_behavior: "none",
                                    {content}
                                }
                            }
                        }
                    }
                }
            }
        });

        rsx! {
            row {
                width: "100%",
                height: week_row_height,
                align_items: "center",
                {cells}
            }
        }
    });

    rsx! { {weeks} }
}

struct ResolvedCalendarDay {
    context: CalendarDayContext,
    style: CalendarDayStyle,
    supporting: Vec<Element>,
    overlays: Vec<Element>,
    replacement: Option<Element>,
}

fn resolve_plugins_for_day(
    mut context: CalendarDayContext,
    plugins: &[CalendarPlugin],
) -> ResolvedCalendarDay {
    let mut style = CalendarDayStyle::default();
    let mut supporting = Vec::new();
    let mut overlays = Vec::new();
    let mut replacement = None;

    for plugin in plugins {
        let decoration = plugin.decorate_day(context);
        style.merge(decoration.style);
        if let Some(color) = decoration.style.primary_color {
            context.primary_color = color;
        }
        if let Some(color) = decoration.style.background_color {
            context.background_color = color;
        }
        if decoration.disabled {
            context.enabled = false;
        }
        if let Some(content) = decoration.supporting {
            supporting.push(content);
        }
        if let Some(content) = decoration.overlay {
            overlays.push(content);
        }
        if decoration.replacement.is_some() {
            replacement = decoration.replacement;
        }
    }

    ResolvedCalendarDay {
        context,
        style,
        supporting,
        overlays,
        replacement,
    }
}

fn dispatch_plugins_for_day(
    plugins: &[CalendarPlugin],
    event: CalendarDayEvent,
) -> CalendarDayEventResponse {
    let prevent_default = plugins
        .iter()
        .map(|plugin| plugin.dispatch_day_event(event).prevent_default)
        .fold(false, |prevented, next| prevented | next);
    CalendarDayEventResponse { prevent_default }
}

#[component]
fn CalendarMonthGrid(
    labels: [String; 12],
    active_month: Option<u8>,
    height: f32,
    on_select: EventHandler<u8>,
) -> Element {
    let theme = use_theme();
    let selected_text = match theme.mode {
        ThemeMode::Light => 0xFFFFFFFF,
        ThemeMode::Dark => 0xFF000000,
    };

    rsx! {
        column {
            width: "100%",
            height,
            for row_index in 0..PICKER_ROW_COUNT {
                row {
                    key: "{row_index}",
                    width: "100%",
                    layout_weight: 1.0,
                    align_items: "center",
                    for column_index in 0..PICKER_COLUMN_COUNT {
                        {
                            let index = row_index * PICKER_COLUMN_COUNT + column_index;
                            let month_number = (index + 1) as u8;
                            let selected = active_month == Some(month_number);
                            let label = labels[index].clone();
                            rsx! {
                                row {
                                    key: "{month_number}",
                                    layout_weight: 1.0,
                                    align_items: "center",
                                    justify_content: "center",
                                    button {
                                        button_type: "normal",
                                        focusable: false,
                                        focus_on_touch: false,
                                        width: "88%",
                                        height: 42.0,
                                        padding_top: 0.0,
                                        padding_right: spacing::XXS,
                                        padding_bottom: 0.0,
                                        padding_left: spacing::XXS,
                                        alignment: "center",
                                        background_color: if selected {
                                            theme.colors.primary
                                        } else {
                                            TRANSPARENT
                                        },
                                        border_style: ARKUI_BORDER_STYLE_SOLID,
                                        border_width: 0.0,
                                        border_color: TRANSPARENT,
                                        border_radius: theme.radii.md,
                                        onclick: move |_| on_select.call(month_number),
                                        text {
                                            content: label,
                                            font_size: typography::SM,
                                            font_weight: if selected { 600_i32 } else { 400_i32 },
                                            font_color: if selected {
                                                selected_text
                                            } else {
                                                theme.colors.card_foreground
                                            },
                                            max_lines: 1,
                                            text_overflow: "ellipsis",
                                            line_height: 20.0,
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn CalendarYearGrid(
    years: Vec<i32>,
    active_year: i32,
    height: f32,
    on_select: EventHandler<i32>,
) -> Element {
    let theme = use_theme();
    let selected_text = match theme.mode {
        ThemeMode::Light => 0xFFFFFFFF,
        ThemeMode::Dark => 0xFF000000,
    };

    rsx! {
        column {
            width: "100%",
            height,
            for row_index in 0..PICKER_ROW_COUNT {
                row {
                    key: "{row_index}",
                    width: "100%",
                    layout_weight: 1.0,
                    align_items: "center",
                    for column_index in 0..PICKER_COLUMN_COUNT {
                        {
                            let index = row_index * PICKER_COLUMN_COUNT + column_index;
                            if let Some(year) = years.get(index).copied() {
                                let selected = year == active_year;
                                rsx! {
                                    row {
                                        key: "{year}",
                                        layout_weight: 1.0,
                                        align_items: "center",
                                        justify_content: "center",
                                        button {
                                            button_type: "normal",
                                            focusable: false,
                                            focus_on_touch: false,
                                            width: "88%",
                                            height: 42.0,
                                            padding_top: 0.0,
                                            padding_right: 0.0,
                                            padding_bottom: 0.0,
                                            padding_left: 0.0,
                                            alignment: "center",
                                            background_color: if selected {
                                                theme.colors.primary
                                            } else {
                                                TRANSPARENT
                                            },
                                            border_style: ARKUI_BORDER_STYLE_SOLID,
                                            border_width: 0.0,
                                            border_color: TRANSPARENT,
                                            border_radius: theme.radii.md,
                                            onclick: move |_| on_select.call(year),
                                            text {
                                                content: year.to_string(),
                                                font_size: typography::SM,
                                                font_weight: if selected { 600_i32 } else { 400_i32 },
                                                font_color: if selected {
                                                    selected_text
                                                } else {
                                                    theme.colors.card_foreground
                                                },
                                                line_height: 20.0,
                                            }
                                        }
                                    }
                                }
                            } else {
                                rsx! { row { layout_weight: 1.0 } }
                            }
                        }
                    }
                }
            }
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

    fn clamp(self, range: CalendarYearRange) -> Self {
        Self {
            year: range.clamp(self.year),
            month: self.month,
        }
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

impl std::fmt::Display for CalendarMonth {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{:04}-{:02}", self.year, self.month)
    }
}

// Howard Hinnant's proleptic-Gregorian civil date algorithms. Keeping these
// pure and local avoids adding a date dependency to every shadcn consumer.
const fn is_leap_year(year: i32) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

const fn days_in_month(year: i32, month: u8) -> u8 {
    match month {
        2 if is_leap_year(year) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        _ => 0,
    }
}

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

#[cfg(test)]
mod tests {
    use std::cell::{Cell, RefCell};

    use dioxus_core::{NoOpMutations, VNode, VirtualDom};
    use dioxus_hooks::use_callback;

    use super::*;

    thread_local! {
        static PLUGIN_RESULT: RefCell<Option<(CalendarDayContext, CalendarDayStyle, usize, bool)>> =
            const { RefCell::new(None) };
        static MONTH_PLUGIN_RESULT: RefCell<Option<(String, usize)>> = const { RefCell::new(None) };
        static PLUGIN_EVENT_CALLS: Cell<usize> = const { Cell::new(0) };
        static PLUGIN_EVENT_PREVENTED: Cell<bool> = const { Cell::new(false) };
    }

    fn plugin_pipeline_test_app() -> Element {
        let first_renderer = use_callback(|_: CalendarDayContext| {
            CalendarDayDecoration::new()
                .with_style(CalendarDayStyle {
                    primary_color: Some(0xFF111111),
                    background_color: Some(0xFFEEEEEE),
                    ..CalendarDayStyle::default()
                })
                .with_supporting(VNode::empty())
                .with_disabled(true)
        });
        let second_renderer = use_callback(|_: CalendarDayContext| {
            CalendarDayDecoration::new()
                .with_style(CalendarDayStyle {
                    primary_color: Some(0xFF222222),
                    border_width: Some(2.0),
                    ..CalendarDayStyle::default()
                })
                .with_supporting(VNode::empty())
                .with_replacement(VNode::empty())
        });
        let first_event = use_callback(|_: CalendarDayEvent| {
            PLUGIN_EVENT_CALLS.with(|calls| calls.set(calls.get() + 1));
            CalendarDayEventResponse::continue_default()
        });
        let second_event = use_callback(|_: CalendarDayEvent| {
            PLUGIN_EVENT_CALLS.with(|calls| calls.set(calls.get() + 1));
            CalendarDayEventResponse::prevent_default()
        });
        let first_month = use_callback(|_: CalendarMonthContext| {
            CalendarMonthDecoration::new()
                .with_title("Reiwa 8")
                .with_supporting(VNode::empty())
        });
        let second_month = use_callback(|context: CalendarMonthContext| {
            CalendarMonthDecoration::new().with_title(format!("{} · 3 memos", context.title))
        });
        let plugins = [
            CalendarPlugin::decorator(first_renderer)
                .with_day_event(first_event)
                .with_month_renderer(first_month),
            CalendarPlugin::decorator(second_renderer)
                .with_day_event(second_event)
                .with_month_renderer(second_month),
        ];

        use_hook(move || {
            let context = CalendarDayContext {
                date: CalendarDate::new(2026, 8, 28).expect("valid test date"),
                selected: false,
                today: false,
                outside_month: false,
                enabled: true,
                primary_color: 0xFF000000,
                background_color: TRANSPARENT,
                supporting_color: 0xFF777777,
            };
            let resolved = resolve_plugins_for_day(context, &plugins);
            let response = dispatch_plugins_for_day(
                &plugins,
                CalendarDayEvent {
                    context: resolved.context,
                    kind: CalendarDayEventKind::Press,
                },
            );
            PLUGIN_RESULT.with(|result| {
                result.replace(Some((
                    resolved.context,
                    resolved.style,
                    resolved.supporting.len(),
                    resolved.replacement.is_some(),
                )))
            });
            PLUGIN_EVENT_PREVENTED.with(|prevented| prevented.set(response.prevent_default));
            let month = resolve_month_plugins(
                CalendarMonth {
                    year: 2026,
                    month: 8,
                },
                "August 2026".to_string(),
                &plugins,
            );
            MONTH_PLUGIN_RESULT.with(|result| result.replace(Some((month.0, month.1.len()))));
        });

        VNode::empty()
    }

    #[test]
    fn formats_month_title_from_external_labels() {
        let labels = CalendarLabels::new(
            ["日", "一", "二", "三", "四", "五", "六"].map(str::to_owned),
            [
                "1月", "2月", "3月", "4月", "5月", "6月", "7月", "8月", "9月", "10月", "11月",
                "12月",
            ]
            .map(str::to_owned),
            "{year}年{month}",
        );

        assert_eq!(labels.month_title(2026, 7), "2026年7月");
        assert_eq!(labels.back_to_days, "Back to dates");
        assert_eq!(
            labels.clone().with_back_to_days("返回日期").back_to_days,
            "返回日期"
        );
    }

    #[test]
    fn validates_civil_dates_and_leap_years() {
        assert_eq!(
            CalendarDate::new(2024, 2, 29).map(|date| date.to_string()),
            Some("2024-02-29".to_owned())
        );
        assert_eq!(CalendarDate::new(2023, 2, 29), None);
        assert_eq!(CalendarDate::new(2024, 13, 1), None);
        assert_eq!(CalendarDate::new(2024, 1, 0), None);
    }

    #[test]
    fn normalizes_and_pages_year_ranges() {
        let range = CalendarYearRange::new(2100, 1900);
        assert_eq!(range.start(), 1900);
        assert_eq!(range.end(), 2100);
        assert_eq!(range.page_start(1900), 1900);
        assert_eq!(range.page_start(1912), 1912);
        assert_eq!(range.page_start(2100), 2092);
        assert_eq!(
            CalendarYearRange::new(i32::MIN, 20_000),
            CalendarYearRange::new(1, 9_999)
        );
    }

    #[test]
    fn clamps_months_without_changing_the_selected_month() {
        let month = CalendarMonth {
            year: 2200,
            month: 6,
        }
        .clamp(CalendarYearRange::new(1900, 2100));

        assert_eq!(month.to_string(), "2100-06");
    }

    #[test]
    fn month_grid_is_stable_and_starts_on_sunday() {
        let dates = CalendarMonth {
            year: 2024,
            month: 2,
        }
        .grid_dates();

        assert_eq!(dates.len(), 42);
        assert_eq!(dates[0].to_string(), "2024-01-28");
        assert_eq!(dates[41].to_string(), "2024-03-09");
    }

    #[test]
    fn plugin_pipeline_merges_content_styles_disable_and_events_in_order() {
        PLUGIN_RESULT.with(|result| result.borrow_mut().take());
        PLUGIN_EVENT_CALLS.with(|calls| calls.set(0));
        PLUGIN_EVENT_PREVENTED.with(|prevented| prevented.set(false));
        MONTH_PLUGIN_RESULT.with(|result| result.borrow_mut().take());

        let mut dom = VirtualDom::new(plugin_pipeline_test_app);
        let mut mutations = NoOpMutations;
        dom.rebuild(&mut mutations);

        let (context, style, supporting_count, has_replacement) = PLUGIN_RESULT
            .with(|result| result.borrow_mut().take())
            .expect("plugin pipeline should resolve during render");
        assert!(!context.enabled, "disabled contributions must be monotonic");
        assert_eq!(context.primary_color, 0xFF222222);
        assert_eq!(context.background_color, 0xFFEEEEEE);
        assert_eq!(style.border_width, Some(2.0));
        assert_eq!(supporting_count, 2, "supporting slots must append");
        assert!(has_replacement, "last replacement must be retained");
        assert_eq!(PLUGIN_EVENT_CALLS.with(Cell::get), 2);
        assert!(PLUGIN_EVENT_PREVENTED.with(Cell::get));
        assert_eq!(
            MONTH_PLUGIN_RESULT.with(|result| result.borrow_mut().take()),
            Some(("Reiwa 8 · 3 memos".to_string(), 1)),
        );
    }

    #[test]
    fn plugin_layout_uses_the_largest_declared_footprint() {
        let plugins = [
            CalendarPlugin::empty().with_layout(CalendarPluginLayout {
                minimum_day_size: Some(42.0),
                minimum_week_row_height: Some(50.0),
            }),
            CalendarPlugin::empty().with_layout(CalendarPluginLayout {
                minimum_day_size: Some(38.0),
                minimum_week_row_height: Some(64.0),
            }),
        ];

        assert_eq!(resolve_plugin_layout(&plugins), (42.0, 64.0));
    }
}
