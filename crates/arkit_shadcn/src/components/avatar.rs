use super::*;

fn avatar<Message: 'static>(
    src: Option<String>,
    fallback_text: impl Into<String>,
) -> Element<Message> {
    avatar_with_radius(src, fallback_text, radii().full)
}

fn avatar_with_radius<Message: 'static>(
    src: Option<String>,
    fallback_text: impl Into<String>,
    radius_value: f32,
) -> Element<Message> {
    let fallback = fallback_text.into();
    if let Some(src) = src {
        arkit::image::<Message, arkit::Theme>(src)
            .width(32.0)
            .height(32.0)
            .border_radius([radius_value, radius_value, radius_value, radius_value])
            .clip(true)
            .into()
    } else {
        arkit::row_component::<Message, arkit::Theme>()
            .width(32.0)
            .height(32.0)
            .background_color(colors().muted)
            .border_radius([radius_value, radius_value, radius_value, radius_value])
            .align_items_center()
            .justify_content_center()
            .children(vec![muted_text(fallback).into()])
            .into()
    }
}

fn avatar_ring<Message: 'static>(
    src: Option<String>,
    fallback_text: impl Into<String>,
) -> Element<Message> {
    avatar_ring_with_radius(src, fallback_text, radii().full)
}

fn avatar_ring_with_radius<Message: 'static>(
    src: Option<String>,
    fallback_text: impl Into<String>,
    radius_value: f32,
) -> Element<Message> {
    let fallback = fallback_text.into();
    if let Some(src) = src {
        arkit::image::<Message, arkit::Theme>(src)
            .width(32.0)
            .height(32.0)
            .border_radius([radius_value, radius_value, radius_value, radius_value])
            .clip(true)
            .border_width([2.0, 2.0, 2.0, 2.0])
            .border_color(colors().background)
            .into()
    } else {
        arkit::row_component::<Message, arkit::Theme>()
            .width(32.0)
            .height(32.0)
            .background_color(colors().muted)
            .border_radius([radius_value, radius_value, radius_value, radius_value])
            .clip(true)
            .border_width([2.0, 2.0, 2.0, 2.0])
            .border_color(colors().background)
            .align_items_center()
            .justify_content_center()
            .children(vec![muted_text(fallback).into()])
            .into()
    }
}

// Struct component API
pub struct Avatar<Message = ()> {
    src: Option<String>,
    fallback: String,
    ring: bool,
    radius: Option<f32>,
    _marker: std::marker::PhantomData<Message>,
}

impl<Message> Avatar<Message> {
    pub fn new(src: Option<String>, fallback: impl Into<String>) -> Self {
        Self {
            src,
            fallback: fallback.into(),
            ring: false,
            radius: None,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn ring(mut self, ring: bool) -> Self {
        self.ring = ring;
        self
    }

    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = Some(radius);
        self
    }
}

impl_component_widget!(Avatar<Message>, Message, |value: &Avatar<Message>| {
    match (value.ring, value.radius) {
        (true, Some(radius)) => {
            avatar_ring_with_radius(value.src.clone(), value.fallback.clone(), radius)
        }
        (true, None) => avatar_ring(value.src.clone(), value.fallback.clone()),
        (false, Some(radius)) => {
            avatar_with_radius(value.src.clone(), value.fallback.clone(), radius)
        }
        (false, None) => avatar(value.src.clone(), value.fallback.clone()),
    }
});
