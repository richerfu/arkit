//! Interactive viewport atoms shared by data zoom rendering, scale domains,
//! hit testing, and the Dioxus component.

use crate::model::{AxisOrientation, AxisType, ChartOption, DataValue, DataZoom};

use super::compat;
use super::geometry::Plot;
use super::layout::grid_plot;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ZoomWindow {
    pub(crate) start: f64,
    pub(crate) end: f64,
    use_start_value: bool,
    use_end_value: bool,
}

impl ZoomWindow {
    pub(crate) fn new(start: f64, end: f64) -> Self {
        let start = start.clamp(0.0, 100.0);
        let end = end.clamp(0.0, 100.0);
        if start <= end {
            Self {
                start,
                end,
                use_start_value: false,
                use_end_value: false,
            }
        } else {
            Self {
                start: end,
                end: start,
                use_start_value: false,
                use_end_value: false,
            }
        }
    }

    fn from_data_zoom(data_zoom: &DataZoom) -> Self {
        Self {
            use_start_value: data_zoom.start_value.is_some(),
            use_end_value: data_zoom.end_value.is_some(),
            ..Self::new(data_zoom.start, data_zoom.end)
        }
    }

    pub(crate) fn span(self) -> f64 {
        self.end - self.start
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ZoomHandle {
    Start,
    End,
    Window,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ZoomDrag {
    pub(crate) data_zoom_index: usize,
    pub(crate) handle: ZoomHandle,
    pub(crate) pointer_start: f32,
    pub(crate) window_start: ZoomWindow,
}

pub(crate) fn initial_windows(option: &ChartOption) -> Vec<ZoomWindow> {
    option
        .data_zoom
        .iter()
        .map(|data_zoom| initial_window(option, data_zoom))
        .collect()
}

fn initial_window(option: &ChartOption, data_zoom: &DataZoom) -> ZoomWindow {
    let mut window = ZoomWindow::from_data_zoom(data_zoom);
    let axis = if let Some(index) = data_zoom.x_axis_index.first() {
        option.x_axis.get(*index)
    } else {
        data_zoom
            .y_axis_index
            .first()
            .and_then(|index| option.y_axis.get(*index))
    };
    let Some(axis) =
        axis.filter(|axis| axis.axis_type == AxisType::Category && !axis.data.is_empty())
    else {
        return window;
    };
    let count = axis.data.len();
    if let Some(index) = data_zoom
        .start_value
        .as_ref()
        .and_then(|value| category_value_index(value, &axis.data))
    {
        window.start = index.min(count - 1) as f64 / count as f64 * 100.0;
    }
    if let Some(index) = data_zoom
        .end_value
        .as_ref()
        .and_then(|value| category_value_index(value, &axis.data))
    {
        window.end = (index.min(count - 1) + 1) as f64 / count as f64 * 100.0;
    }
    window
}

fn category_value_index(value: &DataValue, labels: &[String]) -> Option<usize> {
    match value {
        DataValue::String(value) => labels
            .iter()
            .position(|label| label == value)
            .or_else(|| value.parse().ok()),
        DataValue::Number(value) if value.is_finite() && *value >= 0.0 => Some(*value as usize),
        DataValue::Number(_) => None,
    }
}

#[derive(Debug, Clone)]
pub(super) struct AxisZoom {
    pub(super) window: ZoomWindow,
    pub(super) start_value: Option<DataValue>,
    pub(super) end_value: Option<DataValue>,
}

pub(super) fn axis_window(
    option: &ChartOption,
    windows: &[ZoomWindow],
    orientation: AxisOrientation,
    axis_index: usize,
) -> Option<AxisZoom> {
    option
        .data_zoom
        .iter()
        .enumerate()
        .filter(|(_, value)| match orientation {
            AxisOrientation::X => value.x_axis_index.contains(&axis_index),
            AxisOrientation::Y => value.y_axis_index.contains(&axis_index),
        })
        .map(|(index, value)| {
            let window = windows
                .get(index)
                .copied()
                .unwrap_or_else(|| ZoomWindow::from_data_zoom(value));
            AxisZoom {
                start_value: window
                    .use_start_value
                    .then(|| value.start_value.clone())
                    .flatten(),
                end_value: window
                    .use_end_value
                    .then(|| value.end_value.clone())
                    .flatten(),
                window,
            }
        })
        .reduce(|left, right| AxisZoom {
            window: ZoomWindow::new(
                left.window.start.max(right.window.start),
                left.window.end.min(right.window.end),
            ),
            start_value: left.start_value.or(right.start_value),
            end_value: left.end_value.or(right.end_value),
        })
}

pub(super) fn slider_plot(
    option: &ChartOption,
    data_zoom_index: usize,
    width: f32,
    height: f32,
) -> Option<Plot> {
    let data_zoom = option.data_zoom.get(data_zoom_index)?;
    let vertical = data_zoom.orient == "vertical";
    let axis_index = if vertical {
        data_zoom.y_axis_index.first().copied().unwrap_or(0)
    } else {
        data_zoom.x_axis_index.first().copied().unwrap_or(0)
    };
    let grid_index = if vertical {
        option.y_axis.get(axis_index).map(|axis| axis.grid_index)
    } else {
        option.x_axis.get(axis_index).map(|axis| axis.grid_index)
    }
    .unwrap_or(0);
    let grid = grid_plot(option, grid_index, width, height);
    let thickness = data_zoom.height.max(4.0);
    if vertical {
        let top = compat::length(data_zoom.extra.get("top"), height, grid.y);
        let bottom = compat::length(
            data_zoom.extra.get("bottom"),
            height,
            (height - grid.y - grid.height).max(8.0),
        );
        let x = data_zoom
            .extra
            .get("left")
            .map(|value| compat::length(Some(value), width, 8.0))
            .unwrap_or(8.0);
        Some(Plot {
            x,
            y: top,
            width: thickness,
            height: (height - top - bottom).max(1.0),
        })
    } else {
        let left = compat::length(data_zoom.extra.get("left"), width, grid.x);
        let right = compat::length(
            data_zoom.extra.get("right"),
            width,
            (width - grid.x - grid.width).max(8.0),
        );
        let bottom = data_zoom
            .extra
            .get("bottom")
            .map(|value| compat::length(Some(value), height, 10.0))
            .unwrap_or(10.0);
        Some(Plot {
            x: left,
            y: height - bottom - thickness,
            width: (width - left - right).max(1.0),
            height: thickness,
        })
    }
}

pub(crate) fn drag_window(
    data_zoom: &DataZoom,
    drag: ZoomDrag,
    pointer: f32,
    track_extent: f32,
) -> ZoomWindow {
    let delta = f64::from(pointer - drag.pointer_start) / f64::from(track_extent.max(1.0)) * 100.0;
    let minimum_span = if data_zoom.zoom_lock {
        drag.window_start.span()
    } else {
        1.0
    };
    match drag.handle {
        ZoomHandle::Start => ZoomWindow::new(
            (drag.window_start.start + delta).clamp(0.0, drag.window_start.end - minimum_span),
            drag.window_start.end,
        ),
        ZoomHandle::End => ZoomWindow::new(
            drag.window_start.start,
            (drag.window_start.end + delta).clamp(drag.window_start.start + minimum_span, 100.0),
        ),
        ZoomHandle::Window => {
            let span = drag.window_start.span();
            let start = (drag.window_start.start + delta).clamp(0.0, 100.0 - span);
            ZoomWindow::new(start, start + span)
        }
    }
}

pub(crate) fn drag_window_at(
    option: &ChartOption,
    drag: ZoomDrag,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> Option<ZoomWindow> {
    let data_zoom = option.data_zoom.get(drag.data_zoom_index)?;
    let track = slider_plot(option, drag.data_zoom_index, width, height)?;
    let vertical = data_zoom.orient == "vertical";
    Some(drag_window(
        data_zoom,
        drag,
        if vertical { y } else { x },
        if vertical { track.height } else { track.width },
    ))
}

pub(crate) fn inside_zoom_at(
    option: &ChartOption,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> Option<usize> {
    option
        .data_zoom
        .iter()
        .enumerate()
        .find_map(|(index, data_zoom)| {
            if data_zoom.kind != "inside" {
                return None;
            }
            let axis_index = data_zoom.x_axis_index.first().copied().unwrap_or(0);
            let grid_index = option
                .x_axis
                .get(axis_index)
                .map(|axis| axis.grid_index)
                .unwrap_or(0);
            let plot = grid_plot(option, grid_index, width, height);
            (x >= plot.x && x <= plot.x + plot.width && y >= plot.y && y <= plot.y + plot.height)
                .then_some(index)
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Axis;

    #[test]
    fn window_drag_preserves_span_and_clamps() {
        let zoom = DataZoom::default();
        let drag = ZoomDrag {
            data_zoom_index: 0,
            handle: ZoomHandle::Window,
            pointer_start: 20.0,
            window_start: ZoomWindow::new(20.0, 50.0),
        };
        let result = drag_window(&zoom, drag, 120.0, 100.0);
        assert_eq!(result, ZoomWindow::new(70.0, 100.0));
        assert!(!result.use_start_value);
        assert!(!result.use_end_value);
    }

    #[test]
    fn category_value_window_positions_slider_handles() {
        let option = ChartOption::new()
            .x_axis(Axis::category(["A", "B", "C", "D"]))
            .data_zoom(DataZoom {
                start_value: Some(DataValue::String(String::from("B"))),
                end_value: Some(DataValue::String(String::from("C"))),
                ..DataZoom::default()
            });
        let window = initial_windows(&option)[0];
        assert_eq!(window.start, 25.0);
        assert_eq!(window.end, 75.0);
        assert!(window.use_start_value);
        assert!(window.use_end_value);
    }
}
