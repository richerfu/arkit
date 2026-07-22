//! i18n example — `arkit_i18n` dioxus context + `t!` macro, locale toggled
//! via a `Signal<String>` (the active locale id).

use framework::entry;
use framework::prelude::*;
use framework::{t, use_i18n, use_i18n_provider};

framework::i18n! {
    pub mod tr {
        path: "locales",
        fallback: "zh-CN",
        locales: ["zh-CN", "en-US"],
    }
}

#[entry]
fn app() -> Element {
    let _ = use_i18n_provider(&tr::CATALOG, tr::FALLBACK_LOCALE.id());
    let i18n = use_i18n();
    let mut value = use_signal(|| 0_i32);

    // Resolve strings up-front (the `t!` macro reads the i18n context, which
    // subscribes to the locale signal so these recompute on locale switch).
    let title = t!(tr::app_title());
    let welcome = t!(tr::welcome("Arkit"));
    let counter = t!(tr::counter_value(value()));
    let language_button = t!(tr::language_button());

    rsx! {
        column {
            width: "100%",
            height: "100%",
            align_items: "center",
            justify_content: "center",
            padding: 24.0,

            text { font_size: 28.0, line_height: 32.0, "{title}" }
            text { margin_top: 10.0, font_size: 18.0, line_height: 24.0, "{welcome}" }
            text { margin_top: 10.0, font_size: 18.0, line_height: 24.0, "{counter}" }

            row {
                margin_top: 20.0,
                button {
                    onclick: move |_| {
                        let next = match i18n.locale_id().as_str() {
                            "zh-CN" => "en-US",
                            _ => "zh-CN",
                        };
                        i18n.set_locale_id(next);
                    },
                    "{language_button}"
                }
                button {
                    margin_left: 12.0,
                    onclick: move |_| value += 1,
                    "+"
                }
            }
        }
    }
}
