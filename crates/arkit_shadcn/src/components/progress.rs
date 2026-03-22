use super::*;

pub fn progress(value: f32, total: f32) -> ProgressElement {
    rounded_progress(
        arkit::progress_component()
            .style(ArkUINodeAttributeType::ProgressValue, value)
            .style(ArkUINodeAttributeType::ProgressTotal, total)
            .style(ArkUINodeAttributeType::ProgressColor, color::FOREGROUND)
            .height(8.0),
    )
}
