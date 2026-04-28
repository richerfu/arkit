use super::*;

fn skeleton<Message: 'static>(width: f32, height: f32) -> Element<Message> {
    let skeleton_radius = if (width - height).abs() < f32::EPSILON && width >= 40.0 {
        radii().full
    } else {
        radii().md
    };

    arkit::row_component::<Message, arkit::Theme>()
        .width(width)
        .height(height)
        .background_color(colors().accent)
        .border_radius([
            skeleton_radius,
            skeleton_radius,
            skeleton_radius,
            skeleton_radius,
        ])
        .into()
}

// Struct component API
pub struct Skeleton<Message = ()> {
    width: f32,
    height: f32,
    _marker: std::marker::PhantomData<Message>,
}

impl<Message> Skeleton<Message> {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            width,
            height,
            _marker: std::marker::PhantomData,
        }
    }
}

impl_component_widget!(Skeleton<Message>, Message, |value: &Skeleton<Message>| {
    skeleton(value.width, value.height)
});
