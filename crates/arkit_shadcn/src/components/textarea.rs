use super::*;

const TEXTAREA_PLACEHOLDER_COLOR: u32 = 0x80737373;

pub fn textarea(placeholder: impl Into<String>) -> TextAreaElement {
    input_surface(arkit::text_area(placeholder, ""))
        .background_color(0x00000000)
        .padding([spacing::SM, spacing::MD, spacing::SM, spacing::MD])
        .placeholder_color(TEXTAREA_PLACEHOLDER_COLOR)
        .font_size(typography::MD)
        .line_height(20.0)
        .height(64.0)
}
