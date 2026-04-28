use super::*;

fn progress<Message>(value: f32, total: f32) -> ProgressElement<Message> {
    rounded_progress(
        arkit::progress::<Message, arkit::Theme>(value, total)
            .progress_color(colors().primary)
            .height(8.0),
    )
}

// Struct component API
pub struct Progress<Message = ()> {
    value: f32,
    total: f32,
    _marker: std::marker::PhantomData<Message>,
}

impl<Message> Progress<Message> {
    pub fn new(value: f32, total: f32) -> Self {
        Self {
            value,
            total,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<Message: 'static> arkit::advanced::Widget<Message, arkit::Theme, arkit::Renderer>
    for Progress<Message>
{
    fn body(
        &self,
        _tree: &mut arkit::advanced::widget::Tree,
        _renderer: &arkit::Renderer,
    ) -> Option<Element<Message>> {
        Some(progress::<Message>(self.value, self.total).into())
    }
}

impl<Message: 'static> From<Progress<Message>> for Element<Message> {
    fn from(value: Progress<Message>) -> Self {
        Element::new(value)
    }
}
