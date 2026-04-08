use super::*;

pub fn progress<Message>(value: f32, total: f32) -> ProgressElement<Message> {
    rounded_progress(
        arkit::progress_component::<Message, arkit::Theme>()
            .style(ArkUINodeAttributeType::ProgressValue, value)
            .style(ArkUINodeAttributeType::ProgressTotal, total)
            .style(ArkUINodeAttributeType::ProgressColor, color::FOREGROUND)
            .height(8.0),
    )
}
