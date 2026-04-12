use super::*;
use arkit::TextAlignment;

pub fn label<Message: 'static>(content: impl Into<String>) -> TextElement<Message> {
    arkit::text::<Message, arkit::Theme>(content)
        .font_size(typography::SM)
        .font_weight(FontWeight::W500)
        .font_color(color::FOREGROUND)
        .text_align(TextAlignment::Start)
}
