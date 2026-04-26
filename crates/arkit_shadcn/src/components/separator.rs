use super::*;

fn separator<Message: 'static>() -> Element<Message> {
    arkit::row_component::<Message, arkit::Theme>()
        .height(1.0)
        .percent_width(1.0)
        .background_color(colors().border)
        .into()
}

pub(super) fn separator_vertical<Message: 'static>(height: f32) -> Element<Message> {
    arkit::column_component::<Message, arkit::Theme>()
        .width(1.0)
        .height(height)
        .background_color(colors().border)
        .into()
}

// Struct component API
pub struct Separator<Message = ()> {
    vertical_height: Option<f32>,
    _marker: std::marker::PhantomData<Message>,
}

impl<Message> Separator<Message> {
    pub fn new() -> Self {
        Self {
            vertical_height: None,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn vertical(height: f32) -> Self {
        Self {
            vertical_height: Some(height),
            _marker: std::marker::PhantomData,
        }
    }
}

impl_component_widget!(Separator<Message>, Message, |value: &Separator<Message>| {
    if let Some(height) = value.vertical_height {
        separator_vertical(height)
    } else {
        separator()
    }
});
