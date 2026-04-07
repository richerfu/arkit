use super::*;

const INPUT_PLACEHOLDER_COLOR: u32 = 0x80737373;

pub fn input(placeholder: impl Into<String>) -> TextInputElement {
    input_surface(
        arkit::text_input(placeholder, "")
            .placeholder_color(INPUT_PLACEHOLDER_COLOR)
            .font_size(typography::MD)
            .line_height(20.0)
            .height(40.0),
    )
}
