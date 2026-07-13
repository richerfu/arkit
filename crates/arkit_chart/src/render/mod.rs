mod cartesian;
mod chrome;
mod compat;
mod engine;
mod geometry;
mod graphic;
mod hit;
mod label_layout;
mod layout;
mod marker;
mod prelude;
mod scale;
mod series;
mod style;
mod surface;
mod symbol;
mod viewport;

pub use engine::hit_test;
pub(crate) use engine::{hit_test_with_hidden, nearest_axis_event};
pub(crate) use hit::HitRegion;
pub(crate) use viewport::{
    drag_window_at, initial_windows, inside_zoom_at, ZoomDrag, ZoomHandle, ZoomWindow,
};

pub(crate) fn coordinate_to_pixel(
    option: &crate::model::ChartOption,
    finder: &crate::model::ChartCoordinateFinder,
    value: &crate::model::ChartCoordinatePoint,
    hidden_series: &std::collections::BTreeSet<usize>,
    zoom_windows: &[ZoomWindow],
    width: f32,
    height: f32,
) -> Option<[f32; 2]> {
    let (plot, layout, x_axis_index, y_axis_index) =
        cartesian_conversion_context(option, finder, hidden_series, zoom_windows, width, height)?;
    let x_axis = option.x_axis.get(x_axis_index)?;
    let y_axis = option.y_axis.get(y_axis_index)?;
    let (x_value, x_index) = coordinate_axis_input(x_axis, &value.x)?;
    let (y_value, y_index) = coordinate_axis_input(y_axis, &value.y)?;
    Some([
        layout.x.position_unclamped(&plot, x_value, x_index, false),
        layout.y.position_unclamped(&plot, y_value, y_index, true),
    ])
}

pub(crate) fn coordinate_from_pixel(
    option: &crate::model::ChartOption,
    finder: &crate::model::ChartCoordinateFinder,
    pixel: [f32; 2],
    hidden_series: &std::collections::BTreeSet<usize>,
    zoom_windows: &[ZoomWindow],
    width: f32,
    height: f32,
) -> Option<crate::model::ChartCoordinatePoint> {
    let (plot, layout, x_axis_index, y_axis_index) =
        cartesian_conversion_context(option, finder, hidden_series, zoom_windows, width, height)?;
    let x = layout.x.value_at_position(&plot, pixel[0], false);
    let y = layout.y.value_at_position(&plot, pixel[1], true);
    Some(crate::model::ChartCoordinatePoint {
        x: coordinate_axis_output(option.x_axis.get(x_axis_index)?, x),
        y: coordinate_axis_output(option.y_axis.get(y_axis_index)?, y),
    })
}

pub(crate) fn coordinate_contains_pixel(
    option: &crate::model::ChartOption,
    finder: &crate::model::ChartCoordinateFinder,
    pixel: [f32; 2],
    hidden_series: &std::collections::BTreeSet<usize>,
    zoom_windows: &[ZoomWindow],
    width: f32,
    height: f32,
) -> Option<bool> {
    let (plot, _, _, _) =
        cartesian_conversion_context(option, finder, hidden_series, zoom_windows, width, height)?;
    Some(
        pixel[0] >= plot.x
            && pixel[0] <= plot.x + plot.width
            && pixel[1] >= plot.y
            && pixel[1] <= plot.y + plot.height,
    )
}

fn cartesian_conversion_context(
    option: &crate::model::ChartOption,
    finder: &crate::model::ChartCoordinateFinder,
    hidden_series: &std::collections::BTreeSet<usize>,
    zoom_windows: &[ZoomWindow],
    width: f32,
    height: f32,
) -> Option<(geometry::Plot, scale::CartesianLayout, usize, usize)> {
    let series_axes = finder.series_index.and_then(|index| {
        let series = option.series.get(index)?;
        series::is_cartesian(series).then(|| series::cartesian_axis_indices(series))
    });
    if finder.series_index.is_some() && series_axes.is_none() {
        return None;
    }
    let requested_grid = finder.grid_index;
    let x_axis_index = finder
        .x_axis_index
        .or_else(|| series_axes.map(|axes| axes.0))
        .or_else(|| {
            requested_grid.and_then(|grid| {
                option
                    .x_axis
                    .iter()
                    .position(|axis| axis.grid_index == grid)
            })
        })
        .unwrap_or(0);
    let y_axis_index = finder
        .y_axis_index
        .or_else(|| series_axes.map(|axes| axes.1))
        .or_else(|| {
            requested_grid.and_then(|grid| {
                option
                    .y_axis
                    .iter()
                    .position(|axis| axis.grid_index == grid)
            })
        })
        .unwrap_or(0);
    let x_axis = option.x_axis.get(x_axis_index)?;
    let y_axis = option.y_axis.get(y_axis_index)?;
    let grid_index = requested_grid.unwrap_or(x_axis.grid_index);
    if x_axis.grid_index != grid_index || y_axis.grid_index != grid_index {
        return None;
    }
    let mut series_indices = option
        .series
        .iter()
        .enumerate()
        .filter_map(|(index, series)| {
            (series::is_cartesian(series)
                && series::cartesian_axis_indices(series) == (x_axis_index, y_axis_index)
                && !hidden_series.contains(&index))
            .then_some(index)
        })
        .collect::<Vec<_>>();
    if series_indices.is_empty() {
        series_indices = option
            .series
            .iter()
            .enumerate()
            .filter_map(|(index, series)| {
                (series::is_cartesian(series)
                    && series::cartesian_axis_indices(series) == (x_axis_index, y_axis_index))
                    .then_some(index)
            })
            .collect();
    }
    let plot = layout::grid_plot(option, grid_index, width, height);
    let layout = scale::CartesianLayout::collect(
        option,
        &series_indices,
        x_axis_index,
        y_axis_index,
        zoom_windows,
    );
    Some((plot, layout, x_axis_index, y_axis_index))
}

fn coordinate_axis_input(
    axis: &crate::model::Axis,
    value: &crate::model::DataValue,
) -> Option<(Option<f64>, usize)> {
    if axis.axis_type == crate::model::AxisType::Category {
        let index = match value {
            crate::model::DataValue::String(value) => {
                axis.data.iter().position(|label| label == value)?
            }
            crate::model::DataValue::Number(value) if value.is_finite() => {
                value.round().max(0.0) as usize
            }
            _ => return None,
        };
        Some((None, index))
    } else {
        Some((Some(value.as_f64()?), 0))
    }
}

fn coordinate_axis_output(axis: &crate::model::Axis, value: f64) -> crate::model::DataValue {
    if axis.axis_type == crate::model::AxisType::Category {
        axis.data
            .get(value.round().max(0.0) as usize)
            .cloned()
            .map(crate::model::DataValue::String)
            .unwrap_or(crate::model::DataValue::Null)
    } else {
        crate::model::DataValue::Number(value)
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_option(
    option: &crate::model::ChartOption,
    selected: Option<&crate::model::ChartEvent>,
    hidden_series: &std::collections::BTreeSet<usize>,
    zoom_windows: &[ZoomWindow],
    selected_items: &std::collections::BTreeSet<(usize, usize)>,
    canvas: Option<&ohos_drawing_binding::Canvas>,
    width: f32,
    height: f32,
) {
    draw_option_with_domain(
        option,
        option,
        selected,
        hidden_series,
        zoom_windows,
        selected_items,
        canvas,
        width,
        height,
    );
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_option_with_domain(
    option: &crate::model::ChartOption,
    domain_option: &crate::model::ChartOption,
    selected: Option<&crate::model::ChartEvent>,
    hidden_series: &std::collections::BTreeSet<usize>,
    zoom_windows: &[ZoomWindow],
    selected_items: &std::collections::BTreeSet<(usize, usize)>,
    canvas: Option<&ohos_drawing_binding::Canvas>,
    width: f32,
    height: f32,
) -> Vec<HitRegion> {
    engine::render_option_with_domain(
        option,
        domain_option,
        selected,
        hidden_series,
        zoom_windows,
        selected_items,
        canvas,
        width,
        height,
    )
}

pub(crate) fn cartesian_plot_at(
    option: &crate::model::ChartOption,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> Option<(f32, f32, f32, f32)> {
    (0..option.grid.len().max(1)).find_map(|index| {
        let plot = layout::grid_plot(option, index, width, height);
        (x >= plot.x && x <= plot.x + plot.width && y >= plot.y && y <= plot.y + plot.height)
            .then_some((plot.x, plot.y, plot.width, plot.height))
    })
}

pub(crate) fn draw_toolbox_zoom_selection(
    canvas: &ohos_drawing_binding::Canvas,
    area: crate::model::BrushArea,
) {
    let x = area.start[0].min(area.end[0]);
    let y = area.start[1].min(area.end[1]);
    let width = (area.end[0] - area.start[0]).abs();
    let height = (area.end[1] - area.start[1]).abs();
    surface::fill_rect(canvas, x, y, width, height, 0x225470C6);
    surface::stroke_rect(canvas, x, y, width, height, 0xFF5470C6, 1.0);
}

#[cfg(test)]
mod coordinate_tests {
    use std::collections::BTreeSet;

    use super::*;
    use crate::model::{
        Axis, ChartCoordinateFinder, ChartCoordinatePoint, ChartOption, DataValue, Series,
    };

    #[test]
    fn cartesian_pixel_conversion_round_trips_category_and_value() {
        let option = ChartOption::new()
            .x_axis(Axis::category(["A", "B", "C"]))
            .y_axis(Axis::value())
            .push_series(Series::line("values", [10.0, 20.0, 30.0]));
        let zoom = initial_windows(&option);
        let finder = ChartCoordinateFinder::series(0);
        let input = ChartCoordinatePoint::values("B", 20.0);
        let pixel = coordinate_to_pixel(
            &option,
            &finder,
            &input,
            &BTreeSet::new(),
            &zoom,
            400.0,
            300.0,
        )
        .unwrap();
        assert!(coordinate_contains_pixel(
            &option,
            &finder,
            pixel,
            &BTreeSet::new(),
            &zoom,
            400.0,
            300.0,
        )
        .unwrap());
        let output = coordinate_from_pixel(
            &option,
            &finder,
            pixel,
            &BTreeSet::new(),
            &zoom,
            400.0,
            300.0,
        )
        .unwrap();
        assert_eq!(output.x, DataValue::String(String::from("B")));
        let DataValue::Number(y) = output.y else {
            panic!("numeric y")
        };
        assert!((y - 20.0).abs() < 1e-9);
    }
}
