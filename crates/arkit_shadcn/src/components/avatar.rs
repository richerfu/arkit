use super::*;

pub fn avatar(src: Option<String>, fallback_text: impl Into<String>) -> Element {
    avatar_with_radius(src, fallback_text, radius::FULL)
}

pub fn avatar_with_radius(
    src: Option<String>,
    fallback_text: impl Into<String>,
    radius_value: f32,
) -> Element {
    let fallback = fallback_text.into();
    if let Some(src) = src {
        arkit::image_component()
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
        arkit::row_component()
            .width(32.0)
            .height(32.0)
            .background_color(color::MUTED)
            .style(
                ArkUINodeAttributeType::BorderRadius,
                vec![radius_value, radius_value, radius_value, radius_value],
            )
            .style(ArkUINodeAttributeType::RowAlignItems, FLEX_ALIGN_CENTER)
            .style(ArkUINodeAttributeType::RowJustifyContent, FLEX_ALIGN_CENTER)
            .children(vec![muted_text(fallback).into()])
            .into()
    }
}

pub fn avatar_ring(src: Option<String>, fallback_text: impl Into<String>) -> Element {
    avatar_ring_with_radius(src, fallback_text, radius::FULL)
}

pub fn avatar_ring_with_radius(
    src: Option<String>,
    fallback_text: impl Into<String>,
    radius_value: f32,
) -> Element {
    let fallback = fallback_text.into();
    if let Some(src) = src {
        arkit::image_component()
            .attr(ArkUINodeAttributeType::ImageSrc, src)
            .width(32.0)
            .height(32.0)
            .style(
                ArkUINodeAttributeType::BorderRadius,
                vec![radius_value, radius_value, radius_value, radius_value],
            )
            .style(ArkUINodeAttributeType::Clip, true)
            .style(ArkUINodeAttributeType::BorderWidth, vec![2.0, 2.0, 2.0, 2.0])
            .style(ArkUINodeAttributeType::BorderColor, vec![color::BACKGROUND])
            .into()
    } else {
        arkit::row_component()
            .width(32.0)
            .height(32.0)
            .background_color(color::MUTED)
            .style(
                ArkUINodeAttributeType::BorderRadius,
                vec![radius_value, radius_value, radius_value, radius_value],
            )
            .style(ArkUINodeAttributeType::Clip, true)
            .style(ArkUINodeAttributeType::BorderWidth, vec![2.0, 2.0, 2.0, 2.0])
            .style(ArkUINodeAttributeType::BorderColor, vec![color::BACKGROUND])
            .style(ArkUINodeAttributeType::RowAlignItems, FLEX_ALIGN_CENTER)
            .style(ArkUINodeAttributeType::RowJustifyContent, FLEX_ALIGN_CENTER)
            .children(vec![muted_text(fallback).into()])
            .into()
    }
}
