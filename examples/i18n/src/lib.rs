use arkit::entry;
use arkit::prelude::*;
use arkit::{application, Element, Task};

arkit::i18n::i18n! {
    pub mod tr {
        path: "locales",
        fallback: "zh-CN",
        locales: ["zh-CN", "en-US"],
    }
}

#[derive(Debug, Clone)]
enum Message {
    Increment,
    ToggleLocale,
}

#[derive(Debug, Clone)]
struct AppState {
    i18n: tr::I18n,
    value: i32,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            i18n: tr::I18n::default(),
            value: 0,
        }
    }
}

fn update(state: &mut AppState, message: Message) -> Task<Message> {
    match message {
        Message::Increment => state.value += 1,
        Message::ToggleLocale => {
            let next = match state.i18n.locale() {
                tr::Locale::ZhCn => tr::Locale::EnUs,
                tr::Locale::EnUs => tr::Locale::ZhCn,
            };
            state.i18n.set_locale(next);
        }
    }

    Task::none()
}

fn view(state: &AppState) -> Element<Message> {
    Element::new(CounterView {
        i18n: state.i18n.clone(),
        value: state.value,
    })
}

struct CounterView {
    i18n: tr::I18n,
    value: i32,
}

impl arkit::advanced::Widget<Message, arkit::Theme, arkit::Renderer> for CounterView {
    fn body(
        &self,
        _tree: &mut arkit::advanced::widget::Tree,
        _renderer: &arkit::Renderer,
    ) -> Option<Element<Message>> {
        Some(
            column_component()
                .percent_width(1.0)
                .percent_height(1.0)
                .align_items_center()
                .justify_content_center()
                .padding(24.0)
                .children(vec![
                    text(self.i18n.tr(tr::app_title()))
                        .font_size(28.0)
                        .font_weight(FontWeight::W600)
                        .line_height(32.0)
                        .into(),
                    text(self.i18n.tr(tr::welcome("Arkit")))
                        .margin_top(10.0)
                        .font_size(18.0)
                        .line_height(24.0)
                        .into(),
                    text(self.i18n.tr(tr::counter_value(self.value)))
                        .margin_top(10.0)
                        .font_size(18.0)
                        .line_height(24.0)
                        .into(),
                    row_component()
                        .margin_top(20.0)
                        .children(vec![
                            button(self.i18n.tr(tr::language_button()))
                                .padding([8.0, 12.0, 8.0, 12.0])
                                .on_press(Message::ToggleLocale)
                                .into(),
                            button("+")
                                .margin_left(12.0)
                                .padding([8.0, 12.0, 8.0, 12.0])
                                .on_press(Message::Increment)
                                .into(),
                        ])
                        .into(),
                ])
                .into(),
        )
    }
}

#[entry]
fn app() -> impl arkit::EntryPoint {
    application(AppState::default, update, view)
}
