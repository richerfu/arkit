use super::*;

pub fn input<Message: Send + 'static>(placeholder: impl Into<String>) -> TextInputElement<Message> {
    input_surface(
        arkit::text_input::<Message, arkit::Theme>(placeholder, "")
            .placeholder_color(with_alpha(colors().muted_foreground, 0x80))
            .font_size(typography::MD)
            .line_height(20.0)
            .height(40.0),
    )
}
