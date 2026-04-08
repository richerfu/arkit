use super::*;

const TEXTAREA_PLACEHOLDER_COLOR: u32 = 0x80737373;

pub fn textarea<Message: Send + 'static>(
    placeholder: impl Into<String>,
) -> TextAreaElement<Message> {
    input_surface(arkit::text_area::<Message, arkit::Theme>(placeholder, ""))
        .background_color(0x00000000)
        .padding([spacing::SM, spacing::MD, spacing::SM, spacing::MD])
        .placeholder_color(TEXTAREA_PLACEHOLDER_COLOR)
        .font_size(typography::MD)
        .line_height(20.0)
        .height(64.0)
}
