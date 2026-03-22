use super::*;

pub fn slider(value: f32, min: f32, max: f32) -> SliderElement {
    input_surface(
        arkit::slider_component()
            .style(ArkUINodeAttributeType::SliderValue, value)
            .style(ArkUINodeAttributeType::SliderMinValue, min)
            .style(ArkUINodeAttributeType::SliderMaxValue, max),
    )
}
