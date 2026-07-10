//! Chart-level orchestration. This module lays out shared chrome and delegates
//! each series to its own renderer; it contains no series drawing logic.

use ohos_drawing_binding::Canvas;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

use super::cartesian;
use super::chrome::{draw_data_zoom, draw_legend, draw_title, draw_tooltip, draw_visual_map};
use super::geometry::{effective_palette, Plot};
use super::hit::HitRegion;
use super::layout::grid_plot;
use super::series;
use super::surface::fill_rect;
use super::viewport::{initial_windows, ZoomWindow};
use crate::model::{ChartEvent, ChartOption, Series};

pub fn hit_test(
    option: &ChartOption,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> Option<ChartEvent> {
    hit_test_with_hidden(
        option,
        x,
        y,
        width,
        height,
        &BTreeSet::new(),
        &initial_windows(option),
    )
}

pub(crate) fn hit_test_with_hidden(
    option: &ChartOption,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    hidden_series: &BTreeSet<usize>,
    zoom_windows: &[ZoomWindow],
) -> Option<ChartEvent> {
    render_option(
        option,
        None,
        hidden_series,
        zoom_windows,
        None,
        width,
        height,
    )
    .into_iter()
    .filter_map(|region| region.hit(x, y).map(|distance| (distance, region.event)))
    .min_by(|left, right| left.0.total_cmp(&right.0))
    .map(|(_, event)| event)
}

pub(crate) fn nearest_axis_event(
    option: &ChartOption,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    hidden_series: &BTreeSet<usize>,
    zoom_windows: &[ZoomWindow],
) -> Option<ChartEvent> {
    render_option(
        option,
        None,
        hidden_series,
        zoom_windows,
        None,
        width,
        height,
    )
    .into_iter()
    .map(|region| region.event)
    .filter(|event| {
        let Some(value) = option.series.get(event.series_index) else {
            return false;
        };
        if !series::is_cartesian(value) {
            return false;
        }
        let (x_axis_index, y_axis_index) = series::cartesian_axis_indices(value);
        let grid_index = option
            .x_axis
            .get(x_axis_index)
            .map(|axis| axis.grid_index)
            .or_else(|| option.y_axis.get(y_axis_index).map(|axis| axis.grid_index))
            .unwrap_or(0);
        let plot = grid_plot(option, grid_index, width, height);
        x >= plot.x && x <= plot.x + plot.width && y >= plot.y && y <= plot.y + plot.height
    })
    .min_by(|left, right| {
        let left_distance = (left.x - x).abs() + (left.y - y).abs() * 0.05;
        let right_distance = (right.x - x).abs() + (right.y - y).abs() * 0.05;
        left_distance.total_cmp(&right_distance)
    })
}

pub(super) fn render_option(
    option: &ChartOption,
    selected: Option<&ChartEvent>,
    hidden_series: &BTreeSet<usize>,
    zoom_windows: &[ZoomWindow],
    canvas: Option<&Canvas>,
    width: f32,
    height: f32,
) -> Vec<HitRegion> {
    let width = width.max(1.0);
    let height = height.max(1.0);
    let mut hits = Vec::new();
    let palette = effective_palette(option);
    let view = Plot {
        x: 0.0,
        y: 0.0,
        width,
        height,
    };

    if let Some(canvas) = canvas {
        fill_rect(
            canvas,
            0.0,
            0.0,
            width,
            height,
            option.visual_style.background_color,
        );
        if let Some(title) = &option.title {
            draw_title(canvas, option, title, width, height);
        }
        draw_visual_map(canvas, option, width, height);
    }
    draw_legend(
        canvas,
        option,
        width,
        height,
        &palette,
        hidden_series,
        &mut hits,
    );

    let mut cartesian_groups: BTreeMap<(usize, usize), Vec<usize>> = BTreeMap::new();
    for (series_index, value) in option.series.iter().enumerate() {
        if series::is_cartesian(value) && !hidden_series.contains(&series_index) {
            cartesian_groups
                .entry(series::cartesian_axis_indices(value))
                .or_default()
                .push(series_index);
        }
    }
    for ((x_axis_index, y_axis_index), series_indices) in cartesian_groups {
        let grid_index = option
            .x_axis
            .get(x_axis_index)
            .map(|axis| axis.grid_index)
            .or_else(|| option.y_axis.get(y_axis_index).map(|axis| axis.grid_index))
            .unwrap_or(0);
        let plot = grid_plot(option, grid_index, width, height);
        cartesian::render(cartesian::CartesianChartRenderContext {
            option,
            series_indices: &series_indices,
            plot: &plot,
            axis_indices: (x_axis_index, y_axis_index),
            palette: &palette,
            canvas,
            hits: &mut hits,
            zoom_windows,
            selected,
        });
    }

    let free_series: Vec<(usize, &Series)> = option
        .series
        .iter()
        .enumerate()
        .filter(|(index, value)| !series::is_cartesian(value) && !hidden_series.contains(index))
        .collect();
    for (series_index, value) in free_series {
        series::render_free(
            option,
            series_index,
            value,
            view,
            &palette,
            canvas,
            &mut hits,
        );
    }

    draw_data_zoom(canvas, option, zoom_windows, width, height, &mut hits);

    if let (Some(canvas), Some(selected)) = (canvas, selected) {
        if option.tooltip.show {
            draw_tooltip(canvas, option, selected, hidden_series, width, height);
        }
    }

    hits
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Series;

    #[test]
    fn hit_test_returns_data_event() {
        let option = ChartOption::new().push_series(Series::bar("B", [10.0]));
        let hit = hit_test(&option, 210.0, 60.0, 320.0, 180.0).unwrap();
        assert_eq!(hit.component_type, "bar");
        assert_eq!(hit.series_index, 0);
        assert_eq!(hit.data_index, 0);
    }
}
