use arkit::entry;
use arkit::internal::dispatch;
use arkit::prelude::*;
use arkit::{application, Element, Node, Task, WebViewController};

const RUST_URL: &str = "https://www.rust-lang.org";
const DOCS_URL: &str = "https://docs.rs";

#[derive(Debug, Clone)]
enum Message {
    ProbeTap,
    Reload,
    Focus,
    LoadRust,
    LoadDocs,
    EvalScript,
    ToggleVisible,
    TitleChanged(String),
    StatusChanged(String),
}

#[derive(Clone)]
struct AppState {
    controller: WebViewController,
    current_url: String,
    title: String,
    status: String,
    visible: bool,
    probe_taps: u32,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            controller: WebViewController::with_id("arkit-example-webview"),
            current_url: RUST_URL.to_string(),
            title: String::from("loading..."),
            status: String::from("ready"),
            visible: true,
            probe_taps: 0,
        }
    }
}

fn update(state: &mut AppState, message: Message) -> Task<Message> {
    match message {
        Message::ProbeTap => {
            state.probe_taps += 1;
            state.status = format!("probe tapped {} time(s)", state.probe_taps);
        }
        Message::Reload => {
            state.status = match state.controller.reload() {
                Ok(()) => String::from("reloaded page"),
                Err(error) => format!("reload failed: {error}"),
            };
        }
        Message::Focus => {
            state.status = match state.controller.focus() {
                Ok(()) => String::from("webview focused"),
                Err(error) => format!("focus failed: {error}"),
            };
        }
        Message::LoadRust => {
            state.current_url = RUST_URL.to_string();
            state.status = match state.controller.load_url(RUST_URL) {
                Ok(()) => String::from("loaded rust-lang.org"),
                Err(error) => format!("load failed: {error}"),
            };
        }
        Message::LoadDocs => {
            state.current_url = DOCS_URL.to_string();
            state.status = match state.controller.load_url(DOCS_URL) {
                Ok(()) => String::from("loaded docs.rs"),
                Err(error) => format!("load failed: {error}"),
            };
        }
        Message::EvalScript => {
            state.status = match state.controller.evaluate_script_with_callback(
                "document.title",
                Some(Box::new(|title| {
                    dispatch(Message::StatusChanged(format!("document.title = {title}")));
                })),
            ) {
                Ok(()) => String::from("requested document.title"),
                Err(error) => format!("script failed: {error}"),
            };
        }
        Message::ToggleVisible => {
            state.visible = !state.visible;
            state.status = match state.controller.set_visible(state.visible) {
                Ok(()) => {
                    if state.visible {
                        String::from("webview visible")
                    } else {
                        String::from("webview hidden")
                    }
                }
                Err(error) => format!("toggle failed: {error}"),
            };
        }
        Message::TitleChanged(title) => {
            state.title = title;
            state.status = String::from("title updated from webview");
        }
        Message::StatusChanged(status) => {
            state.status = status;
        }
    }

    Task::none()
}

fn view(state: &AppState) -> Element<Message> {
    column_component()
        .percent_width(1.0)
        .percent_height(1.0)
        .background_color(0xFFF6F7FB)
        .children(vec![
            column_component()
                .padding(16.0)
                .background_color(0xFFFFFFFF)
                .children(vec![
                    text("arkit webview example")
                        .font_size(24.0)
                        .font_weight(FontWeight::W700)
                        .line_height(28.0)
                        .into(),
                    text(format!("title: {}", state.title))
                        .margin_top(8.0)
                        .font_size(14.0)
                        .line_height(18.0)
                        .font_color(0xFF334155)
                        .into(),
                    text(format!("url: {}", state.current_url))
                        .margin_top(4.0)
                        .font_size(13.0)
                        .line_height(18.0)
                        .font_color(0xFF64748B)
                        .into(),
                    text(format!("status: {}", state.status))
                        .margin_top(4.0)
                        .font_size(13.0)
                        .line_height(18.0)
                        .font_color(0xFF0F766E)
                        .into(),
                    toolbar_button(
                        &format!("Tap Probe {}", state.probe_taps),
                        Message::ProbeTap,
                    )
                    .margin_top(12.0)
                    .background_color(0xFF0F766E)
                    .into(),
                    row_component()
                        .margin_top(12.0)
                        .children(vec![
                            toolbar_button("Reload", Message::Reload).into(),
                            toolbar_button("Focus", Message::Focus)
                                .margin_left(8.0)
                                .into(),
                            toolbar_button("Eval JS", Message::EvalScript)
                                .margin_left(8.0)
                                .into(),
                            toolbar_button(
                                if state.visible { "Hide" } else { "Show" },
                                Message::ToggleVisible,
                            )
                            .margin_left(8.0)
                            .into(),
                        ])
                        .into(),
                    row_component()
                        .margin_top(8.0)
                        .children(vec![
                            toolbar_button("rust-lang.org", Message::LoadRust).into(),
                            toolbar_button("docs.rs", Message::LoadDocs)
                                .margin_left(8.0)
                                .into(),
                        ])
                        .into(),
                ])
                .into(),
            container(
                web_view(state.controller.clone(), state.current_url.clone())
                    .background_color(0xFFFFFFFF)
                    .on_title_change(|title| dispatch(Message::TitleChanged(title))),
            )
            .padding(16.0)
            .width(Length::Fill)
            .height(Length::Fill)
            .into(),
        ])
        .into()
}

fn toolbar_button(label: &str, message: Message) -> Node<Message> {
    container(
        text(label)
            .font_color(0xFFFFFFFF)
            .font_size(13.0)
            .line_height(16.0),
    )
    .align_items_center()
    .justify_content_center()
    .padding([10.0, 14.0, 10.0, 14.0])
    .border_radius(10.0)
    .background_color(0xFF111827)
    .on_press(message)
}

#[entry]
fn app() -> impl arkit::EntryPoint {
    application(AppState::default, update, view)
}
