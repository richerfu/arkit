mod cartesian;
mod chrome;
mod compat;
mod engine;
mod geometry;
mod hit;
mod layout;
mod marker;
mod prelude;
mod scale;
mod series;
mod style;
mod surface;
mod viewport;

pub use engine::hit_test;
pub(crate) use engine::{hit_test_with_hidden, nearest_axis_event};
pub(crate) use viewport::{
    drag_window_at, initial_windows, inside_zoom_at, ZoomDrag, ZoomHandle, ZoomWindow,
};

pub(crate) fn draw_option(
    option: &crate::model::ChartOption,
    selected: Option<&crate::model::ChartEvent>,
    hidden_series: &std::collections::BTreeSet<usize>,
    zoom_windows: &[ZoomWindow],
    canvas: Option<&ohos_drawing_binding::Canvas>,
    width: f32,
    height: f32,
) {
    engine::render_option(
        option,
        selected,
        hidden_series,
        zoom_windows,
        canvas,
        width,
        height,
    );
}
