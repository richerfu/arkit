use super::*;

pub(super) fn input<Message: Send + 'static>(
    placeholder: impl Into<String>,
) -> TextInputElement<Message> {
    input_surface(
        arkit::text_input::<Message, arkit::Theme>(placeholder, "")
            .placeholder_color(with_alpha(colors().muted_foreground, 0x80))
            .font_size(typography::MD)
            .line_height(20.0)
            .height(40.0),
    )
}

// Struct component API
pub struct Input<Message = ()> {
    placeholder: String,
    value: Option<String>,
    width: Option<arkit::Length>,
    height: Option<arkit::Length>,
    percent_width: Option<f32>,
    on_input: Option<std::rc::Rc<dyn Fn(String) -> Message>>,
}

impl<Message> Input<Message> {
    pub fn new(placeholder: impl Into<String>) -> Self {
        Self {
            placeholder: placeholder.into(),
            value: None,
            width: None,
            height: None,
            percent_width: None,
            on_input: None,
        }
    }

    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }

    pub fn width(mut self, width: impl Into<arkit::Length>) -> Self {
        self.width = Some(width.into());
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

    pub fn on_input(mut self, handler: impl Fn(String) -> Message + 'static) -> Self {
        self.on_input = Some(std::rc::Rc::new(handler));
        self
    }

    pub fn on_change(self, handler: impl Fn(String) -> Message + 'static) -> Self {
        self.on_input(handler)
    }
}

impl<Message: Send + 'static> arkit::advanced::Widget<Message, arkit::Theme, arkit::Renderer>
    for Input<Message>
{
    fn body(
        &self,
        _tree: &mut arkit::advanced::widget::Tree,
        _renderer: &arkit::Renderer,
    ) -> Option<Element<Message>> {
        let mut input = input::<Message>(self.placeholder.clone());
        if let Some(value) = self.value.clone() {
            input = input.value(value);
        }
        if let Some(width) = self.width {
            input = input.width(width);
        }
        if let Some(height) = self.height {
            input = input.height(height);
        }
        if let Some(width) = self.percent_width {
            input = input.percent_width(width);
        }
        if let Some(handler) = self.on_input.clone() {
            input = input.on_input(move |value| handler(value));
        }
        Some(input.into())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn into_any(self: Box<Self>) -> Box<dyn std::any::Any> {
        self
    }
}

impl<Message: Send + 'static> From<Input<Message>> for Element<Message> {
    fn from(value: Input<Message>) -> Self {
        Element::new(value)
    }
}
