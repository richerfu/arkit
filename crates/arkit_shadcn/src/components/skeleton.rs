use super::*;

pub fn skeleton<Message: 'static>(width: f32, height: f32) -> Element<Message> {
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
