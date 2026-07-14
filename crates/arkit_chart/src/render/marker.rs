//! Cartesian marker atoms for ECharts `markPoint`, `markLine`, and `markArea`.

use serde_json::{Map, Value};

use super::hit::{HitRegion, HitShape};
use super::series::{self, CartesianRenderContext};
use super::surface::{draw_text, fill_circle, fill_rect, stroke_line};
use crate::model::{ChartEvent, Series, SeriesOptions};

#[derive(Clone)]
struct SeriesPoint {
    index: usize,
    x_value: Option<f64>,
    y_value: f64,
}

pub(super) fn render(series: &Series, context: &mut CartesianRenderContext<'_>) {
    let Some(options) = options(series) else {
        return;
    };
    let points = series_points(series, context);
    if let Some(mark_area) = options.extra.get("markArea").and_then(Value::as_object) {
        render_mark_area(series, mark_area, context);
    }
    if let Some(mark_line) = options.extra.get("markLine").and_then(Value::as_object) {
        render_mark_line(series, mark_line, &points, context);
    }
    if let Some(mark_point) = options.extra.get("markPoint").and_then(Value::as_object) {
        render_mark_point(series, mark_point, &points, context);
    }
}

fn render_mark_point(
    series: &Series,
    options: &Map<String, Value>,
    points: &[SeriesPoint],
    context: &mut CartesianRenderContext<'_>,
) {
    let Some(data) = options.get("data").and_then(Value::as_array) else {
        return;
    };
    let symbol_size = options
        .get("symbolSize")
        .and_then(Value::as_f64)
        .unwrap_or(50.0) as f32;
    let default_color = marker_color(options, 0xFFEE6666);
    for (marker_index, value) in data.iter().enumerate() {
        let Some(object) = value.as_object() else {
            continue;
        };
        let Some(projected) = marker_point_position(object, points, context) else {
            continue;
        };
        let radius = object
            .get("symbolSize")
            .and_then(Value::as_f64)
            .map(|value| value as f32)
            .unwrap_or(symbol_size)
            .max(4.0)
            / 2.0;
        let fill = object
            .get("itemStyle")
            .and_then(Value::as_object)
            .map(|style| marker_color(style, default_color))
            .unwrap_or(default_color);
        let name = object
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("markPoint");
        if let Some(canvas) = context.canvas {
            fill_circle(canvas, projected.0, projected.1, radius, fill);
            let label_show = object
                .get("label")
                .and_then(Value::as_object)
                .and_then(|label| label.get("show"))
                .and_then(Value::as_bool)
                .unwrap_or(true);
            if label_show {
                draw_text(
                    canvas,
                    name,
                    projected.0 + radius + 3.0,
                    projected.1 + 4.0,
                    10.0,
                    0xFF333333,
                    500,
                );
            }
        }
        context.hits.push(HitRegion {
            shape: HitShape::Point {
                x: projected.0,
                y: projected.1,
                radius: radius.max(10.0),
            },
            event: ChartEvent {
                series_index: context.series_index,
                data_index: marker_index,
                series_name: series.name().map(ToOwned::to_owned),
                name: Some(name.to_string()),
                value: vec![projected.2],
                x: projected.0,
                y: projected.1,
                component_type: String::from("markPoint"),
            },
        });
    }
}

fn render_mark_line(
    series: &Series,
    options: &Map<String, Value>,
    points: &[SeriesPoint],
    context: &mut CartesianRenderContext<'_>,
) {
    let Some(data) = options.get("data").and_then(Value::as_array) else {
        return;
    };
    let line_style = options.get("lineStyle").and_then(Value::as_object);
    let color = line_style
        .map(|style| marker_color(style, 0xFFEE6666))
        .unwrap_or(0xFFEE6666);
    let width = line_style
        .and_then(|style| style.get("width"))
        .and_then(Value::as_f64)
        .unwrap_or(1.0) as f32;
    for (marker_index, value) in data.iter().enumerate() {
        let (from, to, name, marker_value) = if let Some(pair) = value.as_array() {
            if pair.len() < 2 {
                continue;
            }
            let (Some(from), Some(to)) = (
                pair.first()
                    .and_then(Value::as_object)
                    .and_then(|value| coordinate(value, context)),
                pair.get(1)
                    .and_then(Value::as_object)
                    .and_then(|value| coordinate(value, context)),
            ) else {
                continue;
            };
            (from, to, "markLine", 0.0)
        } else {
            let Some(object) = value.as_object() else {
                continue;
            };
            let name = object
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("markLine");
            if let Some(y_axis) = object.get("yAxis").and_then(Value::as_f64) {
                let y = context
                    .layout
                    .y
                    .position(context.plot, Some(y_axis), 0, true);
                (
                    (context.plot.x, y),
                    (context.plot.x + context.plot.width, y),
                    name,
                    y_axis,
                )
            } else if let Some(x_axis) = object.get("xAxis") {
                let Some((value, index)) = resolve_dimension(x_axis, &context.layout.x) else {
                    continue;
                };
                let x = context.layout.x.position(context.plot, value, index, false);
                (
                    (x, context.plot.y),
                    (x, context.plot.y + context.plot.height),
                    name,
                    value.unwrap_or(index as f64),
                )
            } else if let Some(statistic) = object.get("type").and_then(Value::as_str) {
                let Some(statistic) = statistic_value(statistic, points) else {
                    continue;
                };
                let y = context
                    .layout
                    .y
                    .position(context.plot, Some(statistic), 0, true);
                (
                    (context.plot.x, y),
                    (context.plot.x + context.plot.width, y),
                    name,
                    statistic,
                )
            } else {
                continue;
            }
        };
        if let Some(canvas) = context.canvas {
            stroke_line(canvas, from.0, from.1, to.0, to.1, color, width.max(0.5));
            draw_text(
                canvas,
                name,
                (from.0 + to.0) / 2.0 + 3.0,
                (from.1 + to.1) / 2.0 - 3.0,
                10.0,
                color,
                500,
            );
        }
        context.hits.push(HitRegion {
            shape: HitShape::Rect {
                x: from.0.min(to.0) - 5.0,
                y: from.1.min(to.1) - 5.0,
                width: (to.0 - from.0).abs().max(10.0),
                height: (to.1 - from.1).abs().max(10.0),
            },
            event: ChartEvent {
                series_index: context.series_index,
                data_index: marker_index,
                series_name: series.name().map(ToOwned::to_owned),
                name: Some(name.to_string()),
                value: vec![marker_value],
                x: (from.0 + to.0) / 2.0,
                y: (from.1 + to.1) / 2.0,
                component_type: String::from("markLine"),
            },
        });
    }
}

fn render_mark_area(
    series: &Series,
    options: &Map<String, Value>,
    context: &mut CartesianRenderContext<'_>,
) {
    let Some(data) = options.get("data").and_then(Value::as_array) else {
        return;
    };
    let color = options
        .get("itemStyle")
        .and_then(Value::as_object)
        .map(|style| marker_color(style, 0x225470C6))
        .unwrap_or(0x225470C6);
    for (marker_index, value) in data.iter().enumerate() {
        let Some(pair) = value.as_array() else {
            continue;
        };
        let Some(from) = pair.first().and_then(Value::as_object) else {
            continue;
        };
        let Some(to) = pair.get(1).and_then(Value::as_object) else {
            continue;
        };
        let from = area_coordinate(from, context, true);
        let to = area_coordinate(to, context, false);
        let x = from.0.min(to.0);
        let y = from.1.min(to.1);
        let width = (to.0 - from.0).abs().max(1.0);
        let height = (to.1 - from.1).abs().max(1.0);
        if let Some(canvas) = context.canvas {
            fill_rect(canvas, x, y, width, height, color);
        }
        context.hits.push(HitRegion {
            shape: HitShape::Rect {
                x,
                y,
                width,
                height,
            },
            event: ChartEvent {
                series_index: context.series_index,
                data_index: marker_index,
                series_name: series.name().map(ToOwned::to_owned),
                name: Some(String::from("markArea")),
                value: Vec::new(),
                x: x + width / 2.0,
                y: y + height / 2.0,
                component_type: String::from("markArea"),
            },
        });
    }
}

fn marker_point_position(
    object: &Map<String, Value>,
    points: &[SeriesPoint],
    context: &CartesianRenderContext<'_>,
) -> Option<(f32, f32, f64)> {
    if let Some(statistic) = object.get("type").and_then(Value::as_str) {
        let point = match statistic {
            "max" => points
                .iter()
                .max_by(|left, right| left.y_value.total_cmp(&right.y_value))?,
            "min" => points
                .iter()
                .min_by(|left, right| left.y_value.total_cmp(&right.y_value))?,
            "average" => {
                let average = statistic_value("average", points)?;
                points.iter().min_by(|left, right| {
                    (left.y_value - average)
                        .abs()
                        .total_cmp(&(right.y_value - average).abs())
                })?
            }
            _ => return None,
        };
        let x = context
            .layout
            .x
            .position(context.plot, point.x_value, point.index, false);
        let y = context
            .layout
            .y
            .position(context.plot, Some(point.y_value), point.index, true);
        return Some((x, y, point.y_value));
    }
    let coordinate = coordinate(object, context)?;
    let value = object
        .get("value")
        .and_then(Value::as_f64)
        .unwrap_or_default();
    Some((coordinate.0, coordinate.1, value))
}

fn coordinate(
    object: &Map<String, Value>,
    context: &CartesianRenderContext<'_>,
) -> Option<(f32, f32)> {
    let (x, y) = if let Some(coord) = object.get("coord").and_then(Value::as_array) {
        (coord.first()?, coord.get(1)?)
    } else {
        (object.get("xAxis")?, object.get("yAxis")?)
    };
    let (x_value, x_index) = resolve_dimension(x, &context.layout.x)?;
    let (y_value, y_index) = resolve_dimension(y, &context.layout.y)?;
    Some((
        context
            .layout
            .x
            .position(context.plot, x_value, x_index, false),
        context
            .layout
            .y
            .position(context.plot, y_value, y_index, true),
    ))
}

fn area_coordinate(
    object: &Map<String, Value>,
    context: &CartesianRenderContext<'_>,
    start: bool,
) -> (f32, f32) {
    let x = object
        .get("xAxis")
        .and_then(|value| resolve_dimension(value, &context.layout.x))
        .map(|(value, index)| context.layout.x.position(context.plot, value, index, false))
        .unwrap_or(if start {
            context.plot.x
        } else {
            context.plot.x + context.plot.width
        });
    let y = object
        .get("yAxis")
        .and_then(|value| resolve_dimension(value, &context.layout.y))
        .map(|(value, index)| context.layout.y.position(context.plot, value, index, true))
        .unwrap_or(if start {
            context.plot.y + context.plot.height
        } else {
            context.plot.y
        });
    (x, y)
}

fn resolve_dimension(value: &Value, scale: &super::scale::Scale) -> Option<(Option<f64>, usize)> {
    if scale.is_category() {
        if let Some(label) = value.as_str() {
            let tick = scale.ticks().into_iter().find(|tick| tick.label == label)?;
            return Some((None, tick.index));
        }
        let index = value
            .as_u64()
            .or_else(|| value.as_f64().map(|value| value as u64))? as usize;
        Some((None, index))
    } else {
        value.as_f64().map(|value| (Some(value), 0))
    }
}

fn statistic_value(kind: &str, points: &[SeriesPoint]) -> Option<f64> {
    match kind {
        "max" => points.iter().map(|point| point.y_value).reduce(f64::max),
        "min" => points.iter().map(|point| point.y_value).reduce(f64::min),
        "average" => (!points.is_empty())
            .then(|| points.iter().map(|point| point.y_value).sum::<f64>() / points.len() as f64),
        _ => None,
    }
}

fn series_points(series: &Series, context: &CartesianRenderContext<'_>) -> Vec<SeriesPoint> {
    series::data(series)
        .iter()
        .enumerate()
        .filter_map(|(index, point)| {
            let (x_value, y_value) = match series {
                Series::Line(_) | Series::Bar(_) => {
                    if point.values.len() > 1 {
                        (Some(point.number_opt(0)?), point.number_opt(1)?)
                    } else {
                        (None, point.number_opt(0)?)
                    }
                }
                Series::Scatter(_) | Series::EffectScatter(_) | Series::Heatmap(_) => {
                    (Some(point.number_opt(0)?), point.number_opt(1)?)
                }
                Series::Candlestick(_) => (None, point.number_opt(1)?),
                Series::Boxplot(_) => (None, point.number_opt(2)?),
                Series::PictorialBar(_) => (None, point.number_opt(0)?),
                _ => (None, point.number_opt(0)?),
            };
            if !context.layout.x.contains(x_value, index)
                || !context.layout.y.contains(Some(y_value), index)
            {
                return None;
            }
            Some(SeriesPoint {
                index,
                x_value,
                y_value,
            })
        })
        .collect()
}

fn options(series: &Series) -> Option<&SeriesOptions> {
    match series {
        Series::Line(value)
        | Series::Bar(value)
        | Series::Scatter(value)
        | Series::EffectScatter(value)
        | Series::Heatmap(value)
        | Series::Candlestick(value)
        | Series::Boxplot(value)
        | Series::PictorialBar(value) => Some(&value.options),
        _ => None,
    }
}

fn marker_color(options: &Map<String, Value>, default: u32) -> u32 {
    options
        .get("color")
        .and_then(crate::parser::parse_color)
        .map(|color| {
            let opacity = options
                .get("opacity")
                .and_then(Value::as_f64)
                .unwrap_or(f64::from((color >> 24) as u8) / 255.0)
                .clamp(0.0, 1.0);
            ((opacity * 255.0).round() as u32) << 24 | (color & 0x00FFFFFF)
        })
        .unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::DataPoint;

    #[test]
    fn average_marker_uses_series_values() {
        let points = [
            DataPoint::scalar(2.0),
            DataPoint::scalar(4.0),
            DataPoint::scalar(9.0),
        ];
        let series_points: Vec<SeriesPoint> = points
            .iter()
            .enumerate()
            .map(|(index, point)| SeriesPoint {
                index,
                x_value: None,
                y_value: point.number(0),
            })
            .collect();
        assert_eq!(statistic_value("average", &series_points), Some(5.0));
    }
}
