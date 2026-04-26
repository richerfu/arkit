use arkit::entry;
use arkit::internal::dispatch;
use arkit::prelude::*;
use arkit::{application, Element, Task, WebViewController};

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
    Element::new(WebviewExampleView {
        state: state.clone(),
    })
}

struct WebviewExampleView {
    state: AppState,
}

impl arkit::advanced::Widget<Message, arkit::Theme, arkit::Renderer> for WebviewExampleView {
    fn body(
        &self,
        _tree: &mut arkit::advanced::widget::Tree,
        _renderer: &arkit::Renderer,
    ) -> Option<Element<Message>> {
        let state = &self.state;
        Some(
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
                            Element::new(
                                ToolbarButton::new(
                                    format!("Tap Probe {}", state.probe_taps),
                                    Message::ProbeTap,
                                )
                                .background_color(0xFF0F766E)
                                .margin_top(12.0),
                            ),
                            row_component()
                                .margin_top(12.0)
                                .children(vec![
                                    Element::new(ToolbarButton::new("Reload", Message::Reload)),
                                    Element::new(
                                        ToolbarButton::new("Focus", Message::Focus)
                                            .margin_left(8.0),
                                    ),
                                    Element::new(
                                        ToolbarButton::new("Eval JS", Message::EvalScript)
                                            .margin_left(8.0),
                                    ),
                                    Element::new(
                                        ToolbarButton::new(
                                            if state.visible { "Hide" } else { "Show" },
                                            Message::ToggleVisible,
                                        )
                                        .margin_left(8.0),
                                    ),
                                ])
                                .into(),
                            row_component()
                                .margin_top(8.0)
                                .children(vec![
                                    Element::new(ToolbarButton::new(
                                        "rust-lang.org",
                                        Message::LoadRust,
                                    )),
                                    Element::new(
                                        ToolbarButton::new("docs.rs", Message::LoadDocs)
                                            .margin_left(8.0),
                                    ),
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
                .into(),
        )
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn into_any(self: Box<Self>) -> Box<dyn std::any::Any> {
        self
    }
}

struct ToolbarButton {
    label: String,
    message: Message,
    background_color: u32,
    margin_top: f32,
    margin_left: f32,
}

impl ToolbarButton {
    fn new(label: impl Into<String>, message: Message) -> Self {
        Self {
            label: label.into(),
            message,
            background_color: 0xFF111827,
            margin_top: 0.0,
            margin_left: 0.0,
        }
    }

    fn background_color(mut self, color: u32) -> Self {
        self.background_color = color;
        self
    }

    fn margin_top(mut self, value: f32) -> Self {
        self.margin_top = value;
        self
    }

    fn margin_left(mut self, value: f32) -> Self {
        self.margin_left = value;
        self
    }
}

impl arkit::advanced::Widget<Message, arkit::Theme, arkit::Renderer> for ToolbarButton {
    fn body(
        &self,
        _tree: &mut arkit::advanced::widget::Tree,
        _renderer: &arkit::Renderer,
    ) -> Option<Element<Message>> {
        Some(
            container(
                text(self.label.clone())
                    .font_color(0xFFFFFFFF)
                    .font_size(13.0)
                    .line_height(16.0),
            )
            .align_items_center()
            .justify_content_center()
            .padding([10.0, 14.0, 10.0, 14.0])
            .border_radius(10.0)
            .background_color(self.background_color)
            .margin_top(self.margin_top)
            .margin_left(self.margin_left)
            .on_press(self.message.clone())
            .into(),
        )
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn into_any(self: Box<Self>) -> Box<dyn std::any::Any> {
        self
    }
}

#[entry]
fn app() -> impl arkit::EntryPoint {
    application(AppState::default, update, view)
}
