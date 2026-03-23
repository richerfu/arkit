use super::*;

pub fn skeleton(width: f32, height: f32) -> Element {
    let skeleton_radius = if (width - height).abs() < f32::EPSILON && width >= 40.0 {
        radius::FULL
    } else {
        radius::MD
    };

    arkit::row_component()
        .width(width)
        .height(height)
        .background_color(color::ACCENT)
        .style(
            ArkUINodeAttributeType::BorderRadius,
            vec![
                skeleton_radius,
                skeleton_radius,
                skeleton_radius,
                skeleton_radius,
            ],
        )
        .into()
}
