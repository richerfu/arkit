use super::*;

pub fn separator<Message: 'static>() -> Element<Message> {
    arkit::row_component::<Message, arkit::Theme>()
        .height(1.0)
        .percent_width(1.0)
        .background_color(colors().border)
        .into()
}

pub fn separator_vertical<Message: 'static>(height: f32) -> Element<Message> {
    arkit::column_component::<Message, arkit::Theme>()
        .width(1.0)
        .height(height)
        .background_color(colors().border)
        .into()
}
