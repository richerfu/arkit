//! ICU4X-backed optional plugins for [`arkit_shadcn::components::Calendar`].
//!
//! The crate is separate from `arkit_shadcn` so applications that do not need
//! non-Gregorian calendars do not pay for ICU4X in their dependency graph.

use arkit_i18n::{translate, I18nContext, TypedMessage};
use arkit_prelude::*;
use arkit_shadcn::components::{CalendarDate, CalendarDayContext, CalendarDayPlugin};
use dioxus_hooks::use_callback;
use icu_calendar::{cal::ChineseTraditional, Date};

arkit_i18n::i18n! {
    mod messages {
        path: "locales",
        fallback: "en-US",
        locales: ["en-US", "zh-CN"],
    }
}

/// Accuracy policy used by the Chinese lunar calendar conversion.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ChineseLunarAccuracy {
    /// Render only the ICU4X range cross-checked against published almanacs.
    ///
    /// ICU4X documents agreement from 1900 through 2100 for this calendar.
    #[default]
    VerifiedOnly,
    /// Allow ICU4X's best-effort calculation outside the verified range.
    BestEffort,
}

/// Presentation and accuracy options for [`use_chinese_lunar_plugin`].
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ChineseLunarOptions {
    pub accuracy: ChineseLunarAccuracy,
    /// Optional fixed label color. By default the calendar supplies a color
    /// that remains legible for selected and unselected day cells.
    pub color: Option<u32>,
}

/// Result of converting a Gregorian date to the traditional Chinese calendar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChineseLunarDate {
    /// Gregorian year in which this lunar year begins.
    pub related_iso_year: i32,
    pub month: u8,
    pub day: u8,
    pub is_leap_month: bool,
}

/// Convert a Gregorian calendar date using ICU4X.
///
/// `None` means the date is outside the selected accuracy policy or ICU4X
/// rejected the Gregorian input.
pub fn chinese_lunar_date(
    date: CalendarDate,
    accuracy: ChineseLunarAccuracy,
) -> Option<ChineseLunarDate> {
    let iso = Date::try_new_iso(date.year(), date.month_number(), date.day()).ok()?;
    let lunar = iso.to_calendar(ChineseTraditional::new());
    let related_iso_year = lunar.year().extended_year();
    if accuracy == ChineseLunarAccuracy::VerifiedOnly && !(1900..=2100).contains(&related_iso_year)
    {
        return None;
    }
    let month = lunar.month().to_input();
    Some(ChineseLunarDate {
        related_iso_year,
        month: month.number(),
        day: lunar.day_of_month().0,
        is_leap_month: month.is_leap(),
    })
}

/// Create a Calendar supporting-content plugin that renders Chinese lunar
/// month/day labels. Call this hook once in the component that owns Calendar.
pub fn use_chinese_lunar_plugin(options: ChineseLunarOptions) -> CalendarDayPlugin {
    let i18n = LunarI18n::current();
    let renderer = use_callback(move |context: CalendarDayContext| {
        let label = chinese_lunar_label(context.date, options.accuracy, i18n);
        let color = options.color.unwrap_or(context.supporting_color);
        rsx! {
            if let Some(label) = label {
                text {
                    content: label,
                    font_size: 10.0,
                    font_weight: 400_i32,
                    font_color: color,
                    max_lines: 1,
                    text_overflow: "ellipsis",
                    line_height: 12.0,
                }
            }
        }
    });
    CalendarDayPlugin::new(renderer)
}

#[derive(Clone, Copy)]
struct LunarI18n {
    locale: messages::Locale,
}

impl LunarI18n {
    fn current() -> Self {
        let locale = try_use_context::<I18nContext>()
            .map(|context| Self::for_locale(&context.locale_id()).locale)
            .unwrap_or(messages::FALLBACK_LOCALE);
        Self { locale }
    }

    fn for_locale(locale: &str) -> Self {
        Self {
            locale: match locale {
                "zh-CN" => messages::Locale::ZhCn,
                _ => messages::FALLBACK_LOCALE,
            },
        }
    }

    fn tr(self, message: TypedMessage) -> String {
        translate(&messages::CATALOG, self.locale.id(), message)
    }

    fn month(self, month: u8, leap: bool) -> String {
        if leap {
            self.tr(messages::lunar_leap_month(month))
        } else {
            self.tr(messages::lunar_month(month))
        }
    }

    fn day(self, day: u8) -> String {
        self.tr(messages::lunar_day(day))
    }
}

fn chinese_lunar_label(
    date: CalendarDate,
    accuracy: ChineseLunarAccuracy,
    i18n: LunarI18n,
) -> Option<String> {
    let lunar = chinese_lunar_date(date, accuracy)?;
    if lunar.day == 1 {
        Some(i18n.month(lunar.month, lunar.is_leap_month))
    } else {
        Some(i18n.day(lunar.day))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(year: i32, month: u8, day: u8) -> CalendarDate {
        CalendarDate::new(year, month, day).expect("test date must be valid")
    }

    #[test]
    fn converts_chinese_new_year() {
        assert_eq!(
            chinese_lunar_date(date(2024, 2, 10), ChineseLunarAccuracy::VerifiedOnly),
            Some(ChineseLunarDate {
                related_iso_year: 2024,
                month: 1,
                day: 1,
                is_leap_month: false,
            })
        );
    }

    #[test]
    fn recognizes_leap_months() {
        assert_eq!(
            chinese_lunar_date(date(2023, 3, 22), ChineseLunarAccuracy::VerifiedOnly),
            Some(ChineseLunarDate {
                related_iso_year: 2023,
                month: 2,
                day: 1,
                is_leap_month: true,
            })
        );
    }

    #[test]
    fn verified_policy_hides_dates_outside_the_documented_range() {
        assert_eq!(
            chinese_lunar_date(date(1899, 12, 31), ChineseLunarAccuracy::VerifiedOnly),
            None
        );
        // Gregorian 1900 begins in lunar year 1899; the verified policy is
        // deliberately based on the converted calendar year, not the input.
        assert_eq!(
            chinese_lunar_date(date(1900, 1, 1), ChineseLunarAccuracy::VerifiedOnly),
            None
        );
        assert!(
            chinese_lunar_date(date(1900, 1, 31), ChineseLunarAccuracy::VerifiedOnly).is_some()
        );
        assert!(chinese_lunar_date(date(1899, 12, 31), ChineseLunarAccuracy::BestEffort).is_some());
    }

    #[test]
    fn localizes_compact_lunar_labels() {
        let new_year = date(2024, 2, 10);
        let next_day = date(2024, 2, 11);

        assert_eq!(
            chinese_lunar_label(
                new_year,
                ChineseLunarAccuracy::VerifiedOnly,
                LunarI18n::for_locale("zh-CN")
            ),
            Some("正月".to_owned())
        );
        assert_eq!(
            chinese_lunar_label(
                next_day,
                ChineseLunarAccuracy::VerifiedOnly,
                LunarI18n::for_locale("zh-CN")
            ),
            Some("初二".to_owned())
        );
        assert_eq!(
            chinese_lunar_label(
                new_year,
                ChineseLunarAccuracy::VerifiedOnly,
                LunarI18n::for_locale("en-US")
            ),
            Some("L1".to_owned())
        );
    }
}
