//! Coordinate-domain collection and scale projection atoms.
//!
//! Series renderers do not infer domains independently. This mirrors the
//! ECharts pipeline: collect dimensions once, normalize the axes once, then
//! let every series consume the same coordinate system.

use super::geometry::Plot;
use super::series;
use super::viewport::{self, AxisZoom, ZoomWindow};
use crate::model::{Axis, AxisOrientation, AxisType, ChartOption, DataValue, Series};

#[derive(Debug, Clone)]
pub(super) struct CartesianLayout {
    pub(super) x: Scale,
    pub(super) y: Scale,
}

impl CartesianLayout {
    pub(super) fn collect(
        option: &ChartOption,
        series_indices: &[usize],
        x_axis_index: usize,
        y_axis_index: usize,
        zoom_windows: &[ZoomWindow],
    ) -> Self {
        let mut domain = Domain::default();
        for series_index in series_indices {
            domain.collect_series(&option.series[*series_index]);
        }
        domain.collect_stacks(option, series_indices);

        let x_axis = option
            .x_axis
            .get(x_axis_index)
            .cloned()
            .unwrap_or_else(|| Axis::category(Vec::<String>::new()));
        let y_axis = option
            .y_axis
            .get(y_axis_index)
            .cloned()
            .unwrap_or_else(Axis::value);

        Self {
            x: Scale::from_axis(
                x_axis,
                &domain.x_values,
                domain.x_count,
                false,
                viewport::axis_window(option, zoom_windows, AxisOrientation::X, x_axis_index),
            ),
            y: Scale::from_axis(
                y_axis,
                &domain.y_values,
                domain.y_count,
                true,
                viewport::axis_window(option, zoom_windows, AxisOrientation::Y, y_axis_index),
            ),
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct Scale {
    axis: Axis,
    kind: ScaleKind,
}

#[derive(Debug, Clone)]
enum ScaleKind {
    Category {
        labels: Vec<String>,
        start: usize,
        end: usize,
    },
    Linear {
        min: f64,
        max: f64,
        step: f64,
    },
    Time {
        min: f64,
        max: f64,
        step: f64,
    },
    Log {
        min: f64,
        max: f64,
    },
}

#[derive(Debug, Clone)]
pub(super) struct ScaleTick {
    pub(super) value: f64,
    pub(super) index: usize,
    pub(super) label: String,
}

impl Scale {
    fn from_axis(
        axis: Axis,
        values: &[f64],
        inferred_count: usize,
        include_zero: bool,
        zoom: Option<AxisZoom>,
    ) -> Self {
        let kind = match axis.axis_type {
            AxisType::Category => {
                let count = axis.data.len().max(inferred_count).max(1);
                let labels: Vec<String> = (0..count)
                    .map(|index| {
                        axis.data
                            .get(index)
                            .cloned()
                            .unwrap_or_else(|| (index + 1).to_string())
                    })
                    .collect();
                let (start, end) = category_window(count, &labels, zoom.as_ref());
                ScaleKind::Category { labels, start, end }
            }
            AxisType::Log => {
                let positive: Vec<f64> = values
                    .iter()
                    .copied()
                    .filter(|value| value.is_finite() && *value > 0.0)
                    .collect();
                let data_min = positive.iter().copied().reduce(f64::min).unwrap_or(1.0);
                let data_max = positive.iter().copied().reduce(f64::max).unwrap_or(10.0);
                let min = axis.min.unwrap_or(data_min).max(f64::MIN_POSITIVE);
                let mut max = axis.max.unwrap_or(data_max).max(min);
                if (max - min).abs() < f64::EPSILON {
                    max = min * 10.0;
                }
                let (min, max) = logarithmic_window(min, max, zoom.as_ref());
                ScaleKind::Log { min, max }
            }
            AxisType::Value => {
                let (min, max, _) =
                    nice_extent(values, axis.min, axis.max, axis.split_number, include_zero);
                let (min, max, step) = linear_window(min, max, zoom.as_ref(), axis.split_number);
                ScaleKind::Linear { min, max, step }
            }
            AxisType::Time => {
                let (min, max, _) = time_extent(values, axis.min, axis.max, axis.split_number);
                let (min, max, step) = time_window(min, max, zoom.as_ref(), axis.split_number);
                ScaleKind::Time { min, max, step }
            }
        };
        Self { axis, kind }
    }

    pub(super) fn is_category(&self) -> bool {
        matches!(self.kind, ScaleKind::Category { .. })
    }

    pub(super) fn is_visible(&self) -> bool {
        self.axis.show
    }

    pub(super) fn draws_split_line(&self) -> bool {
        self.axis.split_line
    }

    pub(super) fn draws_labels(&self) -> bool {
        self.axis.axis_label
    }

    pub(super) fn name(&self) -> Option<&str> {
        self.axis.name.as_deref()
    }

    pub(super) fn count(&self) -> usize {
        match &self.kind {
            ScaleKind::Category { start, end, .. } => end.saturating_sub(*start),
            ScaleKind::Linear { .. } | ScaleKind::Time { .. } | ScaleKind::Log { .. } => 0,
        }
    }

    pub(super) fn position(
        &self,
        plot: &Plot,
        value: Option<f64>,
        index: usize,
        vertical: bool,
    ) -> f32 {
        let normalized = match &self.kind {
            ScaleKind::Category { start, end, .. } => {
                let count = end.saturating_sub(*start).max(1);
                let local_index = index.saturating_sub(*start).min(count - 1);
                if self.axis.boundary_gap {
                    (local_index as f64 + 0.5) / count as f64
                } else if count == 1 {
                    0.5
                } else {
                    local_index as f64 / (count - 1) as f64
                }
            }
            ScaleKind::Linear { min, max, .. } => {
                (value.unwrap_or(index as f64) - min) / (max - min).max(1e-12)
            }
            ScaleKind::Time { min, max, .. } => {
                (value.unwrap_or(index as f64) - min) / (max - min).max(1e-12)
            }
            ScaleKind::Log { min, max } => {
                let value = value.unwrap_or(index as f64).max(f64::MIN_POSITIVE);
                (value.ln() - min.ln()) / (max.ln() - min.ln()).max(1e-12)
            }
        }
        .clamp(0.0, 1.0);
        let normalized = if self.axis.inverse {
            1.0 - normalized
        } else {
            normalized
        };

        if vertical {
            plot.y + plot.height * (1.0 - normalized as f32)
        } else {
            plot.x + plot.width * normalized as f32
        }
    }

    pub(super) fn band_width(&self, plot: &Plot, vertical: bool, fallback_count: usize) -> f32 {
        let length = if vertical { plot.height } else { plot.width };
        match &self.kind {
            ScaleKind::Category { start, end, .. } => {
                length / end.saturating_sub(*start).max(1) as f32
            }
            ScaleKind::Linear { .. } | ScaleKind::Time { .. } | ScaleKind::Log { .. } => {
                length / fallback_count.max(1) as f32
            }
        }
    }

    pub(super) fn band_start(
        &self,
        plot: &Plot,
        value: Option<f64>,
        index: usize,
        vertical: bool,
        fallback_count: usize,
    ) -> f32 {
        let center = self.position(plot, value, index, vertical);
        center - self.band_width(plot, vertical, fallback_count) / 2.0
    }

    pub(super) fn zero_position(&self, plot: &Plot, vertical: bool) -> f32 {
        self.position(plot, Some(0.0), 0, vertical)
    }

    pub(super) fn contains(&self, value: Option<f64>, index: usize) -> bool {
        match &self.kind {
            ScaleKind::Category { start, end, .. } => index >= *start && index < *end,
            ScaleKind::Linear { min, max, .. } | ScaleKind::Time { min, max, .. } => {
                let value = value.unwrap_or(index as f64);
                value >= *min && value <= *max
            }
            ScaleKind::Log { min, max } => {
                let value = value.unwrap_or(index as f64);
                value >= *min && value <= *max
            }
        }
    }

    pub(super) fn ticks(&self) -> Vec<ScaleTick> {
        match &self.kind {
            ScaleKind::Category { labels, start, end } => labels
                .iter()
                .enumerate()
                .skip(*start)
                .take(end.saturating_sub(*start))
                .map(|(index, label)| ScaleTick {
                    value: index as f64,
                    index,
                    label: label.clone(),
                })
                .collect(),
            ScaleKind::Linear { min, max, step } => {
                let mut ticks = Vec::new();
                let mut value = *min;
                let limit = self.axis.split_number.saturating_add(3).max(3);
                while value <= *max + *step * 1e-6 && ticks.len() < limit {
                    ticks.push(ScaleTick {
                        value,
                        index: ticks.len(),
                        label: format_number(value),
                    });
                    value += *step;
                }
                if ticks
                    .last()
                    .is_none_or(|tick| (tick.value - max).abs() > step * 1e-6)
                {
                    ticks.push(ScaleTick {
                        value: *max,
                        index: ticks.len(),
                        label: format_number(*max),
                    });
                }
                ticks
            }
            ScaleKind::Time { min, max, step } => {
                let mut ticks = Vec::new();
                let mut value = *min;
                let limit = self.axis.split_number.saturating_add(3).max(3);
                while value <= *max + *step * 1e-6 && ticks.len() < limit {
                    ticks.push(ScaleTick {
                        value,
                        index: ticks.len(),
                        label: format_time(value, *max - *min),
                    });
                    value += *step;
                }
                ticks
            }
            ScaleKind::Log { min, max } => {
                let first = min.log10().floor() as i32;
                let last = max.log10().ceil() as i32;
                (first..=last)
                    .enumerate()
                    .map(|(index, power)| {
                        let value = 10_f64.powi(power);
                        ScaleTick {
                            value,
                            index,
                            label: format_number(value),
                        }
                    })
                    .collect()
            }
        }
    }
}

#[derive(Default)]
struct Domain {
    x_values: Vec<f64>,
    y_values: Vec<f64>,
    x_count: usize,
    y_count: usize,
}

impl Domain {
    fn collect_series(&mut self, value: &Series) {
        let data = series::data(value);
        self.x_count = self.x_count.max(data.len());
        match value {
            Series::Line(_) | Series::Bar(_) | Series::PictorialBar(_) => {
                for (index, point) in data.iter().enumerate() {
                    if point.values.len() > 1 {
                        self.x_values.push(point.number(0));
                        self.y_values.push(point.number(1));
                    } else {
                        self.x_values.push(index as f64);
                        self.y_values.push(point.number(0));
                    }
                }
            }
            Series::Scatter(_) | Series::EffectScatter(_) => {
                for (index, point) in data.iter().enumerate() {
                    self.x_values.push(if point.values.len() > 1 {
                        point.number(0)
                    } else {
                        index as f64
                    });
                    self.y_values.push(if point.values.len() > 1 {
                        point.number(1)
                    } else {
                        point.number(0)
                    });
                }
            }
            Series::Heatmap(_) => {
                for point in data {
                    let x = point.number(0);
                    let y = point.number(1);
                    self.x_values.push(x);
                    self.y_values.push(y);
                    self.x_count = self.x_count.max(x.max(0.0) as usize + 1);
                    self.y_count = self.y_count.max(y.max(0.0) as usize + 1);
                }
            }
            Series::Candlestick(_) => {
                for (index, point) in data.iter().enumerate() {
                    self.x_values.push(index as f64);
                    self.y_values.extend(
                        point
                            .values
                            .iter()
                            .take(4)
                            .filter_map(|value| value.as_f64()),
                    );
                }
            }
            Series::Boxplot(_) => {
                for (index, point) in data.iter().enumerate() {
                    self.x_values.push(index as f64);
                    self.y_values.extend(
                        point
                            .values
                            .iter()
                            .take(5)
                            .filter_map(|value| value.as_f64()),
                    );
                }
            }
            _ => {}
        }
    }

    fn collect_stacks(&mut self, option: &ChartOption, series_indices: &[usize]) {
        let mut accumulators: std::collections::BTreeMap<String, Vec<(f64, f64)>> =
            std::collections::BTreeMap::new();
        for series_index in series_indices {
            let (data, stack) = match &option.series[*series_index] {
                Series::Line(series) | Series::Bar(series) => {
                    (&series.data, series.options.stack.as_deref())
                }
                _ => continue,
            };
            let Some(stack) = stack else { continue };
            let accumulator = accumulators
                .entry(stack.to_string())
                .or_insert_with(|| vec![(0.0, 0.0); data.len()]);
            accumulator.resize(data.len(), (0.0, 0.0));
            for (index, point) in data.iter().enumerate() {
                let value = if point.values.len() > 1 {
                    point.number(1)
                } else {
                    point.number(0)
                };
                if value >= 0.0 {
                    accumulator[index].0 += value;
                    self.y_values.push(accumulator[index].0);
                } else {
                    accumulator[index].1 += value;
                    self.y_values.push(accumulator[index].1);
                }
            }
        }
    }
}

fn category_window(count: usize, labels: &[String], zoom: Option<&AxisZoom>) -> (usize, usize) {
    let Some(zoom) = zoom else {
        return (0, count);
    };
    let start = zoom
        .start_value
        .as_ref()
        .and_then(|value| category_index(value, labels))
        .unwrap_or_else(|| (zoom.window.start / 100.0 * count as f64 + 1e-9).floor() as usize);
    let end = zoom
        .end_value
        .as_ref()
        .and_then(|value| category_index(value, labels))
        .map(|index| index + 1)
        .unwrap_or_else(|| (zoom.window.end / 100.0 * count as f64 - 1e-9).ceil() as usize);
    let start = start.min(count.saturating_sub(1));
    (start, end.clamp(start + 1, count))
}

fn category_index(value: &DataValue, labels: &[String]) -> Option<usize> {
    match value {
        DataValue::String(value) => labels
            .iter()
            .position(|label| label == value)
            .or_else(|| value.parse().ok()),
        DataValue::Number(value) if value.is_finite() && *value >= 0.0 => Some(*value as usize),
        DataValue::Number(_) => None,
    }
}

fn linear_window(
    min: f64,
    max: f64,
    zoom: Option<&AxisZoom>,
    split_number: usize,
) -> (f64, f64, f64) {
    let Some(zoom) = zoom else {
        let step = nice_step(((max - min) / split_number.max(1) as f64).max(f64::MIN_POSITIVE));
        return (min, max, step);
    };
    let span = max - min;
    let window_min = zoom
        .start_value
        .as_ref()
        .and_then(DataValue::as_f64)
        .unwrap_or(min + span * zoom.window.start / 100.0);
    let mut window_max = zoom
        .end_value
        .as_ref()
        .and_then(DataValue::as_f64)
        .unwrap_or(min + span * zoom.window.end / 100.0);
    if window_max <= window_min {
        window_max = window_min + span.abs().max(1.0) * 0.01;
    }
    let step =
        nice_step(((window_max - window_min) / split_number.max(1) as f64).max(f64::MIN_POSITIVE));
    (window_min, window_max, step)
}

fn time_window(
    min: f64,
    max: f64,
    zoom: Option<&AxisZoom>,
    split_number: usize,
) -> (f64, f64, f64) {
    let (min, max, _) = linear_window(min, max, zoom, split_number);
    time_extent(&[], Some(min), Some(max), split_number)
}

fn logarithmic_window(min: f64, max: f64, zoom: Option<&AxisZoom>) -> (f64, f64) {
    let Some(zoom) = zoom else {
        return (min, max);
    };
    let min_log = min.ln();
    let span = max.ln() - min_log;
    let start = zoom
        .start_value
        .as_ref()
        .and_then(DataValue::as_f64)
        .filter(|value| *value > 0.0)
        .unwrap_or_else(|| (min_log + span * zoom.window.start / 100.0).exp());
    let end = zoom
        .end_value
        .as_ref()
        .and_then(DataValue::as_f64)
        .filter(|value| *value > 0.0)
        .unwrap_or_else(|| (min_log + span * zoom.window.end / 100.0).exp());
    if end > start {
        (start, end)
    } else {
        (start, start * 10.0)
    }
}

fn nice_extent(
    values: &[f64],
    explicit_min: Option<f64>,
    explicit_max: Option<f64>,
    split_number: usize,
    include_zero: bool,
) -> (f64, f64, f64) {
    let mut min = values
        .iter()
        .copied()
        .filter(|value| value.is_finite())
        .reduce(f64::min)
        .unwrap_or(0.0);
    let mut max = values
        .iter()
        .copied()
        .filter(|value| value.is_finite())
        .reduce(f64::max)
        .unwrap_or(1.0);

    if include_zero {
        min = min.min(0.0);
        max = max.max(0.0);
    }
    if (max - min).abs() < f64::EPSILON {
        let padding = if min.abs() < 1.0 {
            1.0
        } else {
            min.abs() * 0.5
        };
        min -= padding;
        max += padding;
    }

    let raw_step = (max - min) / split_number.max(1) as f64;
    let step = nice_step(raw_step.max(f64::MIN_POSITIVE));
    if explicit_min.is_none() {
        min = (min / step).floor() * step;
    }
    if explicit_max.is_none() {
        max = (max / step).ceil() * step;
    }
    min = explicit_min.unwrap_or(min);
    max = explicit_max.unwrap_or(max);
    if max <= min {
        max = min + step;
    }
    (min, max, step)
}

fn nice_step(value: f64) -> f64 {
    let power = value.log10().floor();
    let base = 10_f64.powf(power);
    let normalized = value / base;
    let factor = if normalized <= 1.0 {
        1.0
    } else if normalized <= 2.0 {
        2.0
    } else if normalized <= 5.0 {
        5.0
    } else {
        10.0
    };
    factor * base
}

fn time_extent(
    values: &[f64],
    explicit_min: Option<f64>,
    explicit_max: Option<f64>,
    split_number: usize,
) -> (f64, f64, f64) {
    let min = explicit_min
        .or_else(|| values.iter().copied().reduce(f64::min))
        .unwrap_or(0.0);
    let mut max = explicit_max
        .or_else(|| values.iter().copied().reduce(f64::max))
        .unwrap_or(min + 86_400_000.0);
    if max <= min {
        max = min + 86_400_000.0;
    }
    const STEPS: [f64; 16] = [
        1_000.0,
        5_000.0,
        10_000.0,
        30_000.0,
        60_000.0,
        300_000.0,
        900_000.0,
        1_800_000.0,
        3_600_000.0,
        10_800_000.0,
        21_600_000.0,
        43_200_000.0,
        86_400_000.0,
        604_800_000.0,
        2_592_000_000.0,
        31_536_000_000.0,
    ];
    let target = (max - min) / split_number.max(1) as f64;
    let step = STEPS
        .iter()
        .copied()
        .find(|step| *step >= target)
        .unwrap_or(31_536_000_000.0);
    let min = explicit_min.unwrap_or_else(|| (min / step).floor() * step);
    let max = explicit_max.unwrap_or_else(|| (max / step).ceil() * step);
    (min, max, step)
}

fn format_time(timestamp_ms: f64, span_ms: f64) -> String {
    let total_seconds = (timestamp_ms / 1_000.0).floor() as i64;
    let days = total_seconds.div_euclid(86_400);
    let seconds = total_seconds.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    let hour = seconds / 3_600;
    let minute = seconds % 3_600 / 60;
    if span_ms >= 31_536_000_000.0 {
        format!("{year}")
    } else if span_ms >= 86_400_000.0 {
        format!("{month:02}-{day:02}")
    } else {
        format!("{hour:02}:{minute:02}")
    }
}

fn civil_from_days(days: i64) -> (i32, u32, u32) {
    let days = days + 719_468;
    let era = if days >= 0 { days } else { days - 146_096 } / 146_097;
    let day_of_era = days - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    let year = year + i64::from(month <= 2);
    (year as i32, month as u32, day as u32)
}

fn format_number(value: f64) -> String {
    if value.abs() >= 1_000_000.0 || (value != 0.0 && value.abs() < 0.001) {
        return format!("{value:.1e}");
    }
    if (value - value.round()).abs() < 1e-8 {
        return format!("{value:.0}");
    }
    let mut value = format!("{value:.3}");
    while value.ends_with('0') {
        value.pop();
    }
    if value.ends_with('.') {
        value.pop();
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{DataPoint, Series};

    #[test]
    fn scatter_domain_uses_xy_dimensions_across_the_whole_series() {
        let option = ChartOption::new()
            .x_axis(Axis::value())
            .push_series(Series::scatter(
                "points",
                [
                    DataPoint::values([10.0, 3.0]),
                    DataPoint::values([50.0, 8.0]),
                ],
            ));
        let layout = CartesianLayout::collect(&option, &[0], 0, 0, &[]);
        let plot = Plot {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        };
        let left = layout.x.position(&plot, Some(10.0), 0, false);
        let right = layout.x.position(&plot, Some(50.0), 1, false);
        assert!(left >= 0.0);
        assert!(right <= 100.0);
        assert!(right > left);
    }

    #[test]
    fn candle_domain_only_uses_ohlc_values_for_y() {
        let option = ChartOption::new().push_series(Series::candlestick(
            "ohlc",
            [DataPoint::values([20.0, 32.0, 18.0, 36.0])],
        ));
        let layout = CartesianLayout::collect(&option, &[0], 0, 0, &[]);
        let labels: Vec<String> = layout
            .y
            .ticks()
            .into_iter()
            .map(|tick| tick.label)
            .collect();
        assert!(labels.iter().any(|label| label == "40"));
        assert!(!labels.iter().any(|label| label == "1"));
    }

    #[test]
    fn category_data_zoom_limits_ticks_and_positions() {
        let option = ChartOption::new()
            .x_axis(Axis::category(["A", "B", "C", "D", "E", "F"]))
            .data_zoom(crate::model::DataZoom {
                start: 20.0,
                end: 70.0,
                ..crate::model::DataZoom::default()
            })
            .push_series(Series::line("line", [1.0, 2.0, 3.0, 4.0, 5.0, 6.0]));
        let windows = crate::render::initial_windows(&option);
        let layout = CartesianLayout::collect(&option, &[0], 0, 0, &windows);
        let ticks = layout.x.ticks();
        assert_eq!(
            ticks
                .iter()
                .map(|tick| tick.label.as_str())
                .collect::<Vec<_>>(),
            ["B", "C", "D", "E"]
        );
        assert!(!layout.x.contains(None, 0));
        assert!(layout.x.contains(None, 3));
        assert!(!layout.x.contains(None, 5));
    }

    #[test]
    fn data_zoom_value_bounds_override_percent_bounds() {
        let option = ChartOption::new()
            .x_axis(Axis::category(["A", "B", "C", "D", "E"]))
            .data_zoom(crate::model::DataZoom {
                start_value: Some(DataValue::String(String::from("B"))),
                end_value: Some(DataValue::String(String::from("D"))),
                ..crate::model::DataZoom::default()
            })
            .push_series(Series::line("line", [1.0, 2.0, 3.0, 4.0, 5.0]));
        let windows = crate::render::initial_windows(&option);
        let layout = CartesianLayout::collect(&option, &[0], 0, 0, &windows);
        let labels = layout
            .x
            .ticks()
            .into_iter()
            .map(|tick| tick.label)
            .collect::<Vec<_>>();
        assert_eq!(labels, ["B", "C", "D"]);
    }

    #[test]
    fn numeric_data_zoom_value_bounds_define_scale_extent() {
        let option = ChartOption::new()
            .data_zoom(crate::model::DataZoom {
                start_value: Some(DataValue::Number(20.0)),
                end_value: Some(DataValue::Number(60.0)),
                x_axis_index: Vec::new(),
                y_axis_index: vec![0],
                ..crate::model::DataZoom::default()
            })
            .push_series(Series::line("line", [0.0, 20.0, 60.0, 100.0]));
        let windows = crate::render::initial_windows(&option);
        let layout = CartesianLayout::collect(&option, &[0], 0, 0, &windows);
        assert!(!layout.y.contains(Some(10.0), 0));
        assert!(layout.y.contains(Some(20.0), 1));
        assert!(layout.y.contains(Some(60.0), 2));
        assert!(!layout.y.contains(Some(70.0), 3));
    }

    #[test]
    fn category_percent_round_trip_stays_on_exact_boundary() {
        let labels = (1..=12).map(|value| value.to_string()).collect::<Vec<_>>();
        let zoom = AxisZoom {
            window: ZoomWindow::new(100.0 / 12.0, 900.0 / 12.0),
            start_value: None,
            end_value: None,
        };
        assert_eq!(category_window(12, &labels, Some(&zoom)), (1, 9));
    }
}
