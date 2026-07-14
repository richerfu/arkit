use super::super::compat;
use super::super::prelude::*;
use super::super::symbol::{draw_symbol, resolve_symbol};

pub(super) fn render(series: &BasicSeries, context: &mut FreeRenderContext<'_>) {
    let series_index = context.series_index;
    let plot = context.plot;
    let palette = context.palette;
    let canvas = context.canvas;
    let hits = &mut *context.hits;

    let radar_index = compat::number(&series.options.extra, "radarIndex", 0.0).max(0.0) as usize;
    let radar = context.option.radar.get(radar_index);
    let inferred_count = series
        .data
        .iter()
        .map(|point| point.values.len())
        .max()
        .unwrap_or(0);
    let count = radar
        .map(|radar| radar.indicators.len())
        .unwrap_or(inferred_count)
        .max(3);
    let cx = radar
        .map(|radar| {
            compat::position(
                Some(&radar.center[0]),
                plot.x,
                plot.width,
                plot.x + plot.width / 2.0,
            )
        })
        .unwrap_or(plot.x + plot.width / 2.0);
    let cy = radar
        .map(|radar| {
            compat::position(
                Some(&radar.center[1]),
                plot.y,
                plot.height,
                plot.y + plot.height / 2.0,
            )
        })
        .unwrap_or(plot.y + plot.height / 2.0);
    let radius_base = plot.width.min(plot.height) / 2.0;
    let radius = radar
        .map(|radar| compat::length(Some(&radar.radius), radius_base, radius_base * 0.75))
        .unwrap_or(radius_base * 0.75);
    let start = -radar
        .map(|radar| radar.start_angle)
        .unwrap_or(90.0)
        .to_radians();
    let split_number = radar.map(|radar| radar.split_number).unwrap_or(5).max(1);
    let circular = radar.is_some_and(|radar| radar.shape == "circle");
    let radar_extra = radar.map(|radar| &radar.extra);
    let split_line_show = nested_bool(radar_extra, &["splitLine", "show"], true);
    let split_line_color =
        nested_color(radar_extra, &["splitLine", "lineStyle", "color"]).unwrap_or(0xFFE5E7EB);
    let split_line_width =
        nested_number(radar_extra, &["splitLine", "lineStyle", "width"], 1.0) as f32;
    let split_area_show = nested_bool(radar_extra, &["splitArea", "show"], true);
    let split_area_colors = nested_colors(radar_extra, &["splitArea", "areaStyle", "color"])
        .unwrap_or_else(|| vec![0x0DFAFAFA, 0x0DDBE4EE]);
    let axis_line_show = nested_bool(radar_extra, &["axisLine", "show"], true);
    let axis_line_color =
        nested_color(radar_extra, &["axisLine", "lineStyle", "color"]).unwrap_or(0xFFD1D5DB);
    let axis_line_width =
        nested_number(radar_extra, &["axisLine", "lineStyle", "width"], 1.0) as f32;
    let axis_name_show = nested_bool(radar_extra, &["axisName", "show"], true);
    let axis_name_color = nested_color(radar_extra, &["axisName", "color"]);
    let axis_name_size = nested_number(radar_extra, &["axisName", "fontSize"], 12.0);
    let axis_name_formatter = nested_string(radar_extra, &["axisName", "formatter"]);
    let maxima: Vec<(f64, f64)> = (0..count)
        .map(|dimension| {
            radar
                .and_then(|radar| radar.indicators.get(dimension))
                .map(|indicator| (indicator.min, indicator.max))
                .unwrap_or_else(|| {
                    let max = series
                        .data
                        .iter()
                        .filter_map(|point| point.number_opt(dimension))
                        .reduce(f64::max)
                        .unwrap_or(1.0)
                        .max(1.0);
                    (0.0, max)
                })
        })
        .collect();

    if let Some(canvas) = canvas {
        if split_area_show {
            for split in (1..=split_number).rev() {
                let split_radius = radius * split as f32 / split_number as f32;
                let fill = split_area_colors[(split - 1) % split_area_colors.len()];
                if circular {
                    fill_circle(canvas, cx, cy, split_radius, fill);
                } else {
                    let path = radar_polygon_path((cx, cy), split_radius, count, start);
                    fill_path(canvas, &path, fill);
                }
            }
        }
        for split in 1..=split_number {
            if !split_line_show {
                break;
            }
            let split_radius = radius * split as f32 / split_number as f32;
            if circular {
                stroke_circle(
                    canvas,
                    cx,
                    cy,
                    split_radius,
                    split_line_color,
                    split_line_width,
                );
            } else {
                let path = radar_polygon_path((cx, cy), split_radius, count, start);
                stroke_path(canvas, &path, split_line_color, split_line_width);
            }
        }
        for dimension in 0..count {
            let angle = start + TAU * dimension as f32 / count as f32;
            let edge_x = cx + angle.cos() * radius;
            let edge_y = cy + angle.sin() * radius;
            if axis_line_show {
                stroke_line(
                    canvas,
                    cx,
                    cy,
                    edge_x,
                    edge_y,
                    axis_line_color,
                    axis_line_width,
                );
            }
            if axis_name_show {
                if let Some(indicator) = radar.and_then(|radar| radar.indicators.get(dimension)) {
                    let name = axis_name_formatter
                        .unwrap_or("{value}")
                        .replace("{value}", &indicator.name);
                    draw_text(
                        canvas,
                        &name,
                        cx + angle.cos() * (radius + 13.0) - 12.0,
                        cy + angle.sin() * (radius + 13.0) + 5.0,
                        axis_name_size,
                        indicator.color.or(axis_name_color).unwrap_or(0xFF333333),
                        400,
                    );
                }
            }
        }
    }

    for (data_index, point) in series.data.iter().enumerate() {
        let Some(values) = (0..count)
            .map(|dimension| point.number_opt(dimension))
            .collect::<Option<Vec<_>>>()
        else {
            continue;
        };
        let mut vertices = Vec::with_capacity(count);
        let mut path = Path::new();
        for (dimension, (min, max)) in maxima.iter().copied().enumerate().take(count) {
            let normalized =
                ((values[dimension] - min) / (max - min).max(1e-12)).clamp(0.0, 1.0) as f32;
            let angle = start + TAU * dimension as f32 / count as f32;
            let x = cx + angle.cos() * radius * normalized;
            let y = cy + angle.sin() * radius * normalized;
            vertices.push((x, y));
            if dimension == 0 {
                path.move_to(x, y);
            } else {
                path.line_to(x, y);
            }
        }
        path.close();
        if let Some(canvas) = canvas {
            let data_color = item_color(series, Some(point), palette, series_index + data_index);
            if let Some(fill) = area_color(series, palette, series_index + data_index) {
                fill_path(canvas, &path, fill);
            }
            stroke_path(
                canvas,
                &path,
                line_color(series, palette, series_index + data_index),
                series.options.line_style.width,
            );
            if series.options.show_symbol {
                let symbol = resolve_symbol(series, point, None);
                for (x, y) in &vertices {
                    draw_symbol(
                        canvas,
                        &symbol,
                        *x,
                        *y,
                        data_color,
                        border(series, Some(point)),
                    );
                }
            }
        }
        let symbol = resolve_symbol(series, point, None);
        for (x, y) in vertices {
            hits.push(point_hit(
                "radar",
                series_index,
                data_index,
                series.name.clone(),
                point,
                (x, y),
                symbol.hit_radius().max(8.0),
            ));
        }
    }
}

fn radar_polygon_path(center: (f32, f32), radius: f32, count: usize, start: f32) -> Path {
    let mut path = Path::new();
    for dimension in 0..count {
        let angle = start + TAU * dimension as f32 / count as f32;
        let x = center.0 + angle.cos() * radius;
        let y = center.1 + angle.sin() * radius;
        if dimension == 0 {
            path.move_to(x, y);
        } else {
            path.line_to(x, y);
        }
    }
    path.close();
    path
}

fn nested_value<'a>(
    value: Option<&'a std::collections::BTreeMap<String, serde_json::Value>>,
    path: &[&str],
) -> Option<&'a serde_json::Value> {
    let first = path.first()?;
    path[1..]
        .iter()
        .try_fold(value?.get(*first)?, |value, key| {
            value.as_object().and_then(|value| value.get(*key))
        })
}

fn nested_bool(
    value: Option<&std::collections::BTreeMap<String, serde_json::Value>>,
    path: &[&str],
    default: bool,
) -> bool {
    nested_value(value, path)
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(default)
}

fn nested_number(
    value: Option<&std::collections::BTreeMap<String, serde_json::Value>>,
    path: &[&str],
    default: f64,
) -> f64 {
    nested_value(value, path)
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(default)
}

fn nested_string<'a>(
    value: Option<&'a std::collections::BTreeMap<String, serde_json::Value>>,
    path: &[&str],
) -> Option<&'a str> {
    nested_value(value, path).and_then(serde_json::Value::as_str)
}

fn nested_color(
    value: Option<&std::collections::BTreeMap<String, serde_json::Value>>,
    path: &[&str],
) -> Option<u32> {
    nested_value(value, path).and_then(crate::parser::parse_color)
}

fn nested_colors(
    value: Option<&std::collections::BTreeMap<String, serde_json::Value>>,
    path: &[&str],
) -> Option<Vec<u32>> {
    let value = nested_value(value, path)?;
    match value {
        serde_json::Value::Array(values) => {
            let colors = values
                .iter()
                .filter_map(crate::parser::parse_color)
                .collect::<Vec<_>>();
            (!colors.is_empty()).then_some(colors)
        }
        value => crate::parser::parse_color(value).map(|color| vec![color]),
    }
}
