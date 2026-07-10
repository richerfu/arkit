use serde_json::Value;

use super::super::compat;
use super::super::prelude::*;

pub(super) fn render(series: &BasicSeries, context: &mut FreeRenderContext<'_>) {
    let series_index = context.series_index;
    let plot = context.plot;
    let palette = context.palette;
    let canvas = context.canvas;
    let hits = &mut *context.hits;
    let options = &series.options.extra;

    let center = compat::pair(options, "center");
    let cx = compat::position(
        center.map(|pair| pair[0]),
        plot.x,
        plot.width,
        plot.x + plot.width / 2.0,
    );
    let cy = compat::position(
        center.map(|pair| pair[1]),
        plot.y,
        plot.height,
        plot.y + plot.height / 2.0,
    );
    let radius_base = plot.width.min(plot.height) / 2.0;
    let radius = compat::length(options.get("radius"), radius_base, radius_base * 0.75);
    let min = compat::number(options, "min", 0.0);
    let max = compat::number(options, "max", 100.0).max(min + f64::EPSILON);
    let start_angle = compat::number(options, "startAngle", 225.0) as f32;
    let end_angle = compat::number(options, "endAngle", -45.0) as f32;
    let clockwise = compat::boolean(options, "clockwise", true);
    let start = -start_angle.to_radians();
    let sweep = (start_angle - end_angle).abs().to_radians() * if clockwise { 1.0 } else { -1.0 };
    let value = series
        .data
        .first()
        .map(|point| point.number(0))
        .unwrap_or_default()
        .clamp(min, max);
    let normalized = ((value - min) / (max - min)).clamp(0.0, 1.0) as f32;
    let axis_width =
        nested_number(options.get("axisLine"), &["lineStyle", "width"], 10.0).max(1.0) as f32;
    let pointer_show = nested_bool(options.get("pointer"), &["show"], true);
    let progress_show = nested_bool(options.get("progress"), &["show"], false);
    let axis_colors = gauge_axis_colors(options.get("axisLine"));
    let item = item_color(series, series.data.first(), palette, series_index);

    if let Some(canvas) = canvas {
        let mut previous = 0.0;
        for (stop, color) in &axis_colors {
            let stop = stop.clamp(0.0, 1.0);
            stroke_arc(
                canvas,
                (cx, cy),
                radius,
                start + sweep * previous,
                sweep * (stop - previous),
                *color,
                axis_width,
            );
            previous = stop;
        }
        if progress_show {
            stroke_arc(
                canvas,
                (cx, cy),
                radius,
                start,
                sweep * normalized,
                item,
                axis_width,
            );
        }

        let split_number = compat::number(options, "splitNumber", 10.0).max(1.0) as usize;
        let tick_show = nested_bool(options.get("axisTick"), &["show"], true);
        let split_show = nested_bool(options.get("splitLine"), &["show"], true);
        let minor_ticks = 5;
        if tick_show || split_show {
            for tick in 0..=split_number * minor_ticks {
                let major = tick % minor_ticks == 0;
                if (!major && !tick_show) || (major && !split_show) {
                    continue;
                }
                let ratio = tick as f32 / (split_number * minor_ticks) as f32;
                let angle = start + sweep * ratio;
                let length = if major { 9.0 } else { 4.0 };
                stroke_line(
                    canvas,
                    cx + angle.cos() * (radius - axis_width / 2.0),
                    cy + angle.sin() * (radius - axis_width / 2.0),
                    cx + angle.cos() * (radius - axis_width / 2.0 - length),
                    cy + angle.sin() * (radius - axis_width / 2.0 - length),
                    0xFF6B7280,
                    if major { 1.5 } else { 1.0 },
                );
            }
        }

        let angle = start + sweep * normalized;
        if pointer_show {
            let pointer_length = compat::length(
                options
                    .get("pointer")
                    .and_then(Value::as_object)
                    .and_then(|value| value.get("length")),
                radius,
                radius * 0.8,
            );
            let pointer_width = nested_number(options.get("pointer"), &["width"], 6.0) as f32;
            stroke_line(
                canvas,
                cx,
                cy,
                cx + angle.cos() * pointer_length,
                cy + angle.sin() * pointer_length,
                item,
                pointer_width,
            );
            fill_circle(canvas, cx, cy, pointer_width.max(5.0), item);
        }

        if nested_bool(options.get("detail"), &["show"], true) {
            let formatter =
                nested_string(options.get("detail"), &["formatter"]).unwrap_or("{value}");
            let text = formatter.replace("{value}", &format_value(value));
            let font_size = nested_number(options.get("detail"), &["fontSize"], 15.0);
            let color = nested_value(options.get("detail"), &["color"])
                .and_then(crate::parser::parse_color)
                .unwrap_or(0xFF464646);
            draw_text(
                canvas,
                &text,
                cx - text.chars().count() as f32 * font_size as f32 * 0.28,
                cy + radius * 0.48,
                font_size,
                color,
                500,
            );
        }
        if let Some(point) = series.data.first() {
            if let Some(name) = point.name.as_deref().or(series.name.as_deref()) {
                draw_text(
                    canvas,
                    name,
                    cx - name.chars().count() as f32 * 3.0,
                    cy + radius * 0.7,
                    11.0,
                    0xFF6B7280,
                    400,
                );
            }
        }
    }

    if let Some(point) = series.data.first() {
        hits.push(HitRegion {
            shape: HitShape::Sector {
                cx,
                cy,
                inner: (radius - axis_width - 12.0).max(0.0),
                outer: radius + axis_width,
                start: normalize_angle(start),
                sweep,
            },
            event: chart_event("gauge", series_index, 0, series.name.clone(), point, cx, cy),
        });
    }
}

fn gauge_axis_colors(axis_line: Option<&Value>) -> Vec<(f32, u32)> {
    let values = nested_value(axis_line, &["lineStyle", "color"])
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(|value| {
                    let value = value.as_array()?;
                    Some((
                        value.first()?.as_f64()? as f32,
                        crate::parser::parse_color(value.get(1)?)?,
                    ))
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if values.is_empty() {
        vec![(1.0, 0xFFE6EBF8)]
    } else {
        values
    }
}

fn nested_value<'a>(value: Option<&'a Value>, path: &[&str]) -> Option<&'a Value> {
    path.iter().try_fold(value?, |value, key| {
        value.as_object().and_then(|value| value.get(*key))
    })
}

fn nested_number(value: Option<&Value>, path: &[&str], default: f64) -> f64 {
    nested_value(value, path)
        .and_then(Value::as_f64)
        .unwrap_or(default)
}

fn nested_bool(value: Option<&Value>, path: &[&str], default: bool) -> bool {
    nested_value(value, path)
        .and_then(Value::as_bool)
        .unwrap_or(default)
}

fn nested_string<'a>(value: Option<&'a Value>, path: &[&str]) -> Option<&'a str> {
    nested_value(value, path).and_then(Value::as_str)
}

fn format_value(value: f64) -> String {
    if (value - value.round()).abs() < 1e-8 {
        format!("{value:.0}")
    } else {
        format!("{value:.2}")
    }
}
