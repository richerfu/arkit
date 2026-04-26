use std::time::Duration;

use arkit::entry;
use arkit::prelude::*;
use arkit::{application, Element, Task};

#[derive(Debug, Clone)]
enum Message {
    Start,
    Finished(u32),
}

#[derive(Debug, Clone, Default)]
struct AppState {
    loading: bool,
    request_id: u32,
    last_finished: Option<u32>,
}

impl AppState {
    fn new() -> Self {
        Self::default()
    }
}

fn update(state: &mut AppState, message: Message) -> Task<Message> {
    match message {
        Message::Start => {
            state.request_id += 1;
            state.loading = true;
            let request_id = state.request_id;

            Task::perform(
                async move {
                    tokio::time::sleep(Duration::from_millis(800)).await;
                    request_id
                },
                Message::Finished,
            )
        }
        Message::Finished(request_id) => {
            if request_id == state.request_id {
                state.loading = false;
                state.last_finished = Some(request_id);
            }

            Task::none()
        }
    }
}

fn view(state: &AppState) -> Element<Message> {
    Element::new(AsyncTaskView {
        loading: state.loading,
        request_id: state.request_id,
        last_finished: state.last_finished,
    })
}

struct AsyncTaskView {
    loading: bool,
    request_id: u32,
    last_finished: Option<u32>,
}

impl AsyncTaskView {
    fn status(&self) -> String {
        if self.loading {
            format!("running task #{}", self.request_id)
        } else if let Some(id) = self.last_finished {
            format!("finished task #{}", id)
        } else {
            String::from("idle")
        }
    }
}

impl arkit::advanced::Widget<Message, arkit::Theme, arkit::Renderer> for AsyncTaskView {
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
                    text("arkit async task")
                        .font_size(28.0)
                        .font_weight(FontWeight::W600)
                        .line_height(32.0)
                        .into(),
                    text(self.status())
                        .margin_top(12.0)
                        .font_size(18.0)
                        .line_height(24.0)
                        .into(),
                    text(format!("latest request = {}", self.request_id))
                        .margin_top(8.0)
                        .font_size(14.0)
                        .line_height(20.0)
                        .into(),
                    button("start async task")
                        .margin_top(20.0)
                        .padding([8.0, 12.0, 8.0, 12.0])
                        .on_press(Message::Start)
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

#[entry]
fn app() -> impl arkit::EntryPoint {
    application(AppState::new, update, view)
}
