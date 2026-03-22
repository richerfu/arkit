use super::*;

const INPUT_PLACEHOLDER_COLOR: u32 = 0x80737373;

pub fn input(placeholder: impl Into<String>) -> TextInputElement {
    input_surface(
        arkit::text_input_component()
            .style(
                ArkUINodeAttributeType::TextInputPlaceholderColor,
                INPUT_PLACEHOLDER_COLOR,
            )
            .font_size(typography::MD)
            .height(40.0),
    )
    .placeholder(placeholder)
}
