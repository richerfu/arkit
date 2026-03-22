use super::*;

pub fn separator() -> Element {
    arkit::row_component()
        .height(1.0)
        .percent_width(1.0)
        .background_color(color::BORDER)
        .into()
}

pub fn separator_vertical(height: f32) -> Element {
    arkit::column_component()
        .width(1.0)
        .height(height)
        .background_color(color::BORDER)
        .into()
}
