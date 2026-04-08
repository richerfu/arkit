use super::*;

pub fn avatar<Message: 'static>(
    src: Option<String>,
    fallback_text: impl Into<String>,
) -> Element<Message> {
    avatar_with_radius(src, fallback_text, radius::FULL)
}

pub fn avatar_with_radius<Message: 'static>(
    src: Option<String>,
    fallback_text: impl Into<String>,
    radius_value: f32,
) -> Element<Message> {
    let fallback = fallback_text.into();
    if let Some(src) = src {
        arkit::image_component::<Message, arkit::Theme>()
            .attr(ArkUINodeAttributeType::ImageSrc, src)
            .width(32.0)
            .height(32.0)
            .style(
                ArkUINodeAttributeType::BorderRadius,
                vec![radius_value, radius_value, radius_value, radius_value],
            )
            .style(ArkUINodeAttributeType::Clip, true)
            .into()
    } else {
        arkit::row_component::<Message, arkit::Theme>()
            .width(32.0)
            .height(32.0)
            .background_color(color::MUTED)
            .style(
                ArkUINodeAttributeType::BorderRadius,
                vec![radius_value, radius_value, radius_value, radius_value],
            )
            .align_items_center()
            .style(ArkUINodeAttributeType::RowJustifyContent, FLEX_ALIGN_CENTER)
            .children(vec![muted_text(fallback).into()])
            .into()
    }
}

pub fn avatar_ring<Message: 'static>(
    src: Option<String>,
    fallback_text: impl Into<String>,
) -> Element<Message> {
    avatar_ring_with_radius(src, fallback_text, radius::FULL)
}

pub fn avatar_ring_with_radius<Message: 'static>(
    src: Option<String>,
    fallback_text: impl Into<String>,
    radius_value: f32,
) -> Element<Message> {
    let fallback = fallback_text.into();
    if let Some(src) = src {
        arkit::image_component::<Message, arkit::Theme>()
            .attr(ArkUINodeAttributeType::ImageSrc, src)
            .width(32.0)
            .height(32.0)
            .style(
                ArkUINodeAttributeType::BorderRadius,
                vec![radius_value, radius_value, radius_value, radius_value],
            )
            .style(ArkUINodeAttributeType::Clip, true)
            .style(
                ArkUINodeAttributeType::BorderWidth,
                vec![2.0, 2.0, 2.0, 2.0],
            )
            .style(ArkUINodeAttributeType::BorderColor, vec![color::BACKGROUND])
            .into()
    } else {
        arkit::row_component::<Message, arkit::Theme>()
            .width(32.0)
            .height(32.0)
            .background_color(color::MUTED)
            .style(
                ArkUINodeAttributeType::BorderRadius,
                vec![radius_value, radius_value, radius_value, radius_value],
            )
            .style(ArkUINodeAttributeType::Clip, true)
            .style(
                ArkUINodeAttributeType::BorderWidth,
                vec![2.0, 2.0, 2.0, 2.0],
            )
            .style(ArkUINodeAttributeType::BorderColor, vec![color::BACKGROUND])
            .align_items_center()
            .style(ArkUINodeAttributeType::RowJustifyContent, FLEX_ALIGN_CENTER)
            .children(vec![muted_text(fallback).into()])
            .into()
    }
}
