use super::*;

pub fn avatar<Message: 'static>(
    src: Option<String>,
    fallback_text: impl Into<String>,
) -> Element<Message> {
    avatar_with_radius(src, fallback_text, radii().full)
}

pub fn avatar_with_radius<Message: 'static>(
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

pub fn avatar_ring<Message: 'static>(
    src: Option<String>,
    fallback_text: impl Into<String>,
) -> Element<Message> {
    avatar_ring_with_radius(src, fallback_text, radii().full)
}

pub fn avatar_ring_with_radius<Message: 'static>(
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
