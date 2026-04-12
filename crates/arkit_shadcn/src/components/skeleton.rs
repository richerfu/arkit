use super::*;

pub fn skeleton<Message: 'static>(width: f32, height: f32) -> Element<Message> {
    let skeleton_radius = if (width - height).abs() < f32::EPSILON && width >= 40.0 {
        radius::FULL
    } else {
        radius::MD
    };

    arkit::row_component::<Message, arkit::Theme>()
        .width(width)
        .height(height)
        .background_color(color::ACCENT)
        .border_radius([
            skeleton_radius,
            skeleton_radius,
            skeleton_radius,
            skeleton_radius,
        ])
        .into()
}
