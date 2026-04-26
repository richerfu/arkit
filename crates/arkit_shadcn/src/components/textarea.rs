use super::*;

fn textarea<Message: Send + 'static>(placeholder: impl Into<String>) -> TextAreaElement<Message> {
    input_surface(arkit::text_area::<Message, arkit::Theme>(placeholder, ""))
        .background_color(0x00000000)
        .padding([spacing::SM, spacing::MD, spacing::SM, spacing::MD])
        .placeholder_color(with_alpha(colors().muted_foreground, 0x80))
        .font_size(typography::MD)
        .line_height(20.0)
        .height(64.0)
}

// Struct component API
pub struct Textarea<Message = ()> {
    placeholder: String,
    value: Option<String>,
    height: Option<arkit::Length>,
    percent_width: Option<f32>,
    _marker: std::marker::PhantomData<Message>,
}

impl<Message> Textarea<Message> {
    pub fn new(placeholder: impl Into<String>) -> Self {
        Self {
            placeholder: placeholder.into(),
            value: None,
            height: None,
            percent_width: None,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }

    pub fn height(mut self, height: impl Into<arkit::Length>) -> Self {
        self.height = Some(height.into());
        self
    }

    pub fn percent_width(mut self, value: f32) -> Self {
        self.percent_width = Some(value);
        self
    }
}

impl<Message: Send + 'static> arkit::advanced::Widget<Message, arkit::Theme, arkit::Renderer>
    for Textarea<Message>
{
    fn body(
        &self,
        _tree: &mut arkit::advanced::widget::Tree,
        _renderer: &arkit::Renderer,
    ) -> Option<Element<Message>> {
        let mut textarea = textarea::<Message>(self.placeholder.clone());
        if let Some(value) = self.value.clone() {
            textarea = textarea.value(value);
        }
        if let Some(height) = self.height {
            textarea = textarea.height(height);
        }
        if let Some(width) = self.percent_width {
            textarea = textarea.percent_width(width);
        }
        Some(textarea.into())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn into_any(self: Box<Self>) -> Box<dyn std::any::Any> {
        self
    }
}

impl<Message: Send + 'static> From<Textarea<Message>> for Element<Message> {
    fn from(value: Textarea<Message>) -> Self {
        Element::new(value)
    }
}
