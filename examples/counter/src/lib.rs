use arkit::entry;
use arkit::prelude::*;
use arkit::{application, Element, Task};

#[derive(Debug, Clone)]
enum Message {
    Increment,
    Decrement,
}

#[derive(Debug, Clone, Default)]
struct AppState {
    value: i32,
}

impl AppState {
    fn new() -> Self {
        Self::default()
    }
}

fn update(state: &mut AppState, message: Message) -> Task<Message> {
    match message {
        Message::Increment => state.value += 1,
        Message::Decrement => state.value -= 1,
    }

    Task::none()
}

fn view(state: &AppState) -> Element<Message> {
    column_component()
        .percent_width(1.0)
        .percent_height(1.0)
        .align_items_center()
        .style(ArkUINodeAttributeType::ColumnJustifyContent, 2_i32)
        .style(
            ArkUINodeAttributeType::Padding,
            vec![24.0, 24.0, 24.0, 24.0],
        )
        .children(vec![
            text("arkit counter")
                .font_size(28.0)
                .style(ArkUINodeAttributeType::FontWeight, 5_i32)
                .style(ArkUINodeAttributeType::TextLineHeight, 32.0)
                .into(),
            row_component()
                .style(ArkUINodeAttributeType::Margin, vec![12.0, 0.0, 0.0, 0.0])
                .children(vec![text(format!("value = {}", state.value))
                    .font_size(18.0)
                    .style(ArkUINodeAttributeType::TextLineHeight, 24.0)
                    .into()])
                .into(),
            row_component()
                .style(ArkUINodeAttributeType::Margin, vec![20.0, 0.0, 0.0, 0.0])
                .align_items_center()
                .children(vec![
                    button("decrement")
                        .padding([8.0, 12.0, 8.0, 12.0])
                        .on_press(Message::Decrement)
                        .into(),
                    row_component()
                        .style(ArkUINodeAttributeType::Margin, vec![0.0, 0.0, 0.0, 12.0])
                        .children(vec![button("increment")
                            .padding([8.0, 12.0, 8.0, 12.0])
                            .on_press(Message::Increment)
                            .into()])
                        .into(),
                ])
                .into(),
        ])
        .into()
}

#[entry]
fn app() -> impl arkit::EntryPoint {
    application(AppState::new, update, view)
}
