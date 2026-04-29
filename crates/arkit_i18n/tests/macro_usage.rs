arkit_i18n::i18n! {
    mod tr {
        path: "tests/locales",
        fallback: "zh-CN",
        locales: ["zh-CN", "en-US"],
    }
}

#[test]
fn generated_api_translates_plain_message() {
    let i18n = tr::I18n::new(tr::Locale::ZhCn);

    assert_eq!(i18n.tr(tr::app_title()), "阅读");
}

#[test]
fn generated_api_translates_message_with_args() {
    let i18n = tr::I18n::new(tr::Locale::EnUs);

    assert_eq!(i18n.tr(tr::welcome_user("Ada")), "Hello, Ada");
    assert_eq!(
        i18n.tr(tr::book_detail_chapter_count(1293)),
        "1293 chapters"
    );
}

#[test]
fn generated_locale_metadata_is_available() {
    let i18n = tr::I18n::default();

    assert_eq!(tr::FALLBACK_LOCALE, tr::Locale::ZhCn);
    assert_eq!(i18n.locale(), tr::Locale::ZhCn);
    assert_eq!(
        i18n.available_locales(),
        &[tr::Locale::ZhCn, tr::Locale::EnUs]
    );
}
