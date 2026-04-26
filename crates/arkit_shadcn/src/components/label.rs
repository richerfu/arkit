use super::*;
use arkit::TextAlignment;

pub(super) fn label<Message: 'static>(content: impl Into<String>) -> TextElement<Message> {
    arkit::text::<Message, arkit::Theme>(content)
        .font_size(typography::SM)
        .font_weight(FontWeight::W500)
        .font_color(colors().foreground)
        .text_align(TextAlignment::Start)
}

// Struct component API
pub struct Label<Message = ()> {
    content: String,
    _marker: std::marker::PhantomData<Message>,
}

impl<Message> Label<Message> {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            _marker: std::marker::PhantomData,
        }
    }
}

impl<Message: 'static> From<Label<Message>> for arkit::TextElement<Message> {
    fn from(value: Label<Message>) -> Self {
        label(value.content)
    }
}

impl_component_widget!(Label<Message>, Message, |value: &Label<Message>| {
    label(value.content.clone()).into()
});
