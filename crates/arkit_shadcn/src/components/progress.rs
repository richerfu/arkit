use super::*;

pub fn progress<Message>(value: f32, total: f32) -> ProgressElement<Message> {
    rounded_progress(
        arkit::progress::<Message, arkit::Theme>(value, total)
            .progress_color(colors().primary)
            .height(8.0),
    )
}
