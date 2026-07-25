use arkit_i18n::{translate, I18nContext, TypedMessage};
use arkit_prelude::try_use_context;

arkit_i18n::i18n! {
    pub(crate) mod messages {
        path: "locales",
        fallback: "en-US",
        locales: ["en-US", "zh-CN"],
    }
}

#[derive(Clone, Copy)]
pub(crate) struct ComponentI18n {
    context: Option<I18nContext>,
    fallback: messages::Locale,
}

impl ComponentI18n {
    fn for_locale(locale: &str) -> Self {
        let fallback = match locale {
            "zh-CN" => messages::Locale::ZhCn,
            _ => messages::FALLBACK_LOCALE,
        };
        Self {
            context: None,
            fallback,
        }
    }

    fn locale(self) -> messages::Locale {
        self.context
            .map(|context| ComponentI18n::for_locale(&context.locale_id()).fallback)
            .unwrap_or(self.fallback)
    }

    fn tr(self, message: TypedMessage) -> String {
        translate(&messages::CATALOG, self.locale().id(), message)
    }

    pub(crate) fn calendar_weekdays(self) -> [String; 7] {
        [
            self.tr(messages::calendar_weekday_sunday()),
            self.tr(messages::calendar_weekday_monday()),
            self.tr(messages::calendar_weekday_tuesday()),
            self.tr(messages::calendar_weekday_wednesday()),
            self.tr(messages::calendar_weekday_thursday()),
            self.tr(messages::calendar_weekday_friday()),
            self.tr(messages::calendar_weekday_saturday()),
        ]
    }

    pub(crate) fn calendar_months(self) -> [String; 12] {
        [
            self.tr(messages::calendar_month_january()),
            self.tr(messages::calendar_month_february()),
            self.tr(messages::calendar_month_march()),
            self.tr(messages::calendar_month_april()),
            self.tr(messages::calendar_month_may()),
            self.tr(messages::calendar_month_june()),
            self.tr(messages::calendar_month_july()),
            self.tr(messages::calendar_month_august()),
            self.tr(messages::calendar_month_september()),
            self.tr(messages::calendar_month_october()),
            self.tr(messages::calendar_month_november()),
            self.tr(messages::calendar_month_december()),
        ]
    }

    pub(crate) fn calendar_month_title_template(self) -> String {
        self.tr(messages::calendar_month_title_template())
    }

    pub(crate) fn calendar_back_to_days(self) -> String {
        self.tr(messages::calendar_back_to_days())
    }

    pub(crate) fn chart_series(self, number: usize) -> String {
        self.tr(messages::chart_series(number))
    }

    pub(crate) fn combobox_placeholder(self) -> String {
        self.tr(messages::combobox_placeholder())
    }

    pub(crate) fn combobox_label(self) -> String {
        self.tr(messages::combobox_label())
    }

    pub(crate) fn command_placeholder(self) -> String {
        self.tr(messages::command_placeholder())
    }

    pub(crate) fn date_picker_placeholder(self) -> String {
        self.tr(messages::date_picker_placeholder())
    }

    pub(crate) fn date_picker_close(self) -> String {
        self.tr(messages::date_picker_close())
    }

    pub(crate) fn time_picker_placeholder(self) -> String {
        self.tr(messages::time_picker_placeholder())
    }

    pub(crate) fn time_picker_title(self) -> String {
        self.tr(messages::time_picker_title())
    }

    pub(crate) fn time_picker_hour(self) -> String {
        self.tr(messages::time_picker_hour())
    }

    pub(crate) fn time_picker_minute(self) -> String {
        self.tr(messages::time_picker_minute())
    }

    pub(crate) fn time_picker_clear(self) -> String {
        self.tr(messages::time_picker_clear())
    }

    pub(crate) fn time_picker_confirm(self) -> String {
        self.tr(messages::time_picker_confirm())
    }

    pub(crate) fn time_picker_am(self) -> String {
        self.tr(messages::time_picker_am())
    }

    pub(crate) fn time_picker_pm(self) -> String {
        self.tr(messages::time_picker_pm())
    }

    pub(crate) fn guide_previous(self) -> String {
        self.tr(messages::guide_previous())
    }

    pub(crate) fn guide_next(self) -> String {
        self.tr(messages::guide_next())
    }

    pub(crate) fn guide_skip(self) -> String {
        self.tr(messages::guide_skip())
    }

    pub(crate) fn guide_finish(self) -> String {
        self.tr(messages::guide_finish())
    }

    pub(crate) fn load_more_idle(self) -> String {
        self.tr(messages::load_more_idle())
    }

    pub(crate) fn load_more_loading(self) -> String {
        self.tr(messages::load_more_loading())
    }

    pub(crate) fn load_more_failed(self) -> String {
        self.tr(messages::load_more_failed())
    }

    pub(crate) fn load_more_no_more(self) -> String {
        self.tr(messages::load_more_no_more())
    }

    pub(crate) fn load_more_retry(self) -> String {
        self.tr(messages::load_more_retry())
    }

    pub(crate) fn markdown_admonition_note(self) -> String {
        self.tr(messages::markdown_admonition_note())
    }

    pub(crate) fn markdown_admonition_tip(self) -> String {
        self.tr(messages::markdown_admonition_tip())
    }

    pub(crate) fn markdown_admonition_important(self) -> String {
        self.tr(messages::markdown_admonition_important())
    }

    pub(crate) fn markdown_admonition_warning(self) -> String {
        self.tr(messages::markdown_admonition_warning())
    }

    pub(crate) fn markdown_admonition_caution(self) -> String {
        self.tr(messages::markdown_admonition_caution())
    }

    pub(crate) fn pagination_previous(self) -> String {
        self.tr(messages::pagination_previous())
    }

    pub(crate) fn pagination_next(self) -> String {
        self.tr(messages::pagination_next())
    }

    pub(crate) fn select_placeholder(self) -> String {
        self.tr(messages::select_placeholder())
    }

    pub(crate) fn select_label(self) -> String {
        self.tr(messages::select_label())
    }
}

pub(crate) fn use_component_i18n() -> ComponentI18n {
    ComponentI18n {
        context: try_use_context::<I18nContext>(),
        fallback: messages::FALLBACK_LOCALE,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translates_english_component_copy() {
        let i18n = ComponentI18n::for_locale("en-US");

        assert_eq!(i18n.select_placeholder(), "Select an option");
        assert_eq!(i18n.calendar_months()[6], "July");
        assert_eq!(i18n.calendar_month_title_template(), "{month} {year}");
        assert_eq!(i18n.calendar_back_to_days(), "Back to dates");
        assert_eq!(i18n.chart_series(2), "Series 2");
        assert_eq!(i18n.guide_previous(), "Previous");
        assert_eq!(i18n.guide_next(), "Next");
        assert_eq!(i18n.guide_skip(), "Skip");
        assert_eq!(i18n.guide_finish(), "Finish");
        assert_eq!(i18n.time_picker_placeholder(), "Pick a time");
        assert_eq!(i18n.time_picker_title(), "Select time");
        assert_eq!(i18n.time_picker_confirm(), "Done");
        assert_eq!(i18n.time_picker_am(), "AM");
        assert_eq!(i18n.load_more_loading(), "Loading…");
        assert_eq!(i18n.load_more_no_more(), "No more items");
    }

    #[test]
    fn translates_chinese_component_copy() {
        let i18n = ComponentI18n::for_locale("zh-CN");

        assert_eq!(i18n.select_placeholder(), "请选择");
        assert_eq!(i18n.calendar_months()[6], "七月");
        assert_eq!(i18n.calendar_month_title_template(), "{year}年{month}");
        assert_eq!(i18n.calendar_back_to_days(), "返回日期");
        assert_eq!(i18n.chart_series(2), "系列 2");
        assert_eq!(i18n.guide_previous(), "上一步");
        assert_eq!(i18n.guide_next(), "下一步");
        assert_eq!(i18n.guide_skip(), "跳过");
        assert_eq!(i18n.guide_finish(), "完成");
        assert_eq!(i18n.time_picker_placeholder(), "选择时间");
        assert_eq!(i18n.time_picker_title(), "选择时间");
        assert_eq!(i18n.time_picker_confirm(), "完成");
        assert_eq!(i18n.time_picker_am(), "上午");
        assert_eq!(i18n.load_more_loading(), "加载中…");
        assert_eq!(i18n.load_more_no_more(), "没有更多了");
    }

    #[test]
    fn unsupported_locale_uses_english_fallback() {
        let i18n = ComponentI18n::for_locale("fr-FR");

        assert_eq!(i18n.pagination_previous(), "Prev");
    }
}
