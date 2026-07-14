use serde_json::Value;

use super::super::compat;
use super::super::label_layout::draw_rotated_text;
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
    let values = series
        .data
        .iter()
        .enumerate()
        .filter_map(|(index, point)| {
            point
                .number_opt(0)
                .filter(|value| value.is_finite())
                .map(|value| (index, point, value))
        })
        .collect::<Vec<_>>();
    if values.is_empty() {
        return;
    }

    let axis_line = options.get("axisLine");
    let axis_width = nested_number(axis_line, &["lineStyle", "width"], 10.0).max(1.0) as f32;
    let axis_round_cap = nested_bool(axis_line, &["roundCap"], false);
    let axis_colors = gauge_axis_colors(axis_line);
    let pointer = options.get("pointer");
    let progress = options.get("progress");
    let pointer_show = nested_bool(pointer, &["show"], true);
    let progress_show = nested_bool(progress, &["show"], false);
    let progress_width = nested_number(progress, &["width"], axis_width as f64).max(1.0) as f32;
    let progress_round_cap = nested_bool(progress, &["roundCap"], false);
    let progress_overlap = nested_bool(progress, &["overlap"], true);
    let progress_clip = nested_bool(progress, &["clip"], true);

    if let Some(canvas) = canvas {
        if nested_bool(axis_line, &["show"], true) {
            draw_axis_line(
                canvas,
                (cx, cy),
                radius,
                start,
                sweep,
                axis_width,
                axis_round_cap,
                &axis_colors,
            );
        }

        if progress_show {
            for (order, (data_index, point, value)) in values.iter().enumerate() {
                let normalized = normalize_progress(*value, min, max, progress_clip);
                let width = if progress_overlap {
                    progress_width
                } else {
                    axis_width / values.len() as f32
                };
                let progress_radius = if progress_overlap {
                    radius - width / 2.0
                } else {
                    radius - (order as f32 + 0.5) * width
                };
                let point_progress = point.extra.get("progress");
                stroke_arc_with_cap(
                    canvas,
                    (cx, cy),
                    progress_radius,
                    start,
                    sweep * normalized,
                    nested_color(point_progress, &["itemStyle", "color"])
                        .or_else(|| nested_color(progress, &["itemStyle", "color"]))
                        .unwrap_or_else(|| {
                            item_color(series, Some(point), palette, series_index + *data_index)
                        }),
                    width,
                    progress_round_cap,
                );
            }
        }

        draw_ticks_and_labels(
            canvas,
            options,
            (cx, cy),
            radius,
            axis_width,
            start,
            sweep,
            min,
            max,
            &axis_colors,
        );

        let pointer_show_above = nested_bool(pointer, &["showAbove"], true);
        let anchor = options.get("anchor");
        let anchor_show_above = nested_bool(anchor, &["showAbove"], false);
        let draw_content = |canvas: &ohos_drawing_binding::Canvas| {
            for (data_index, point, value) in &values {
                let color = item_color(series, Some(point), palette, series_index + *data_index);
                draw_detail(
                    canvas,
                    point.extra.get("detail"),
                    options.get("detail"),
                    (cx, cy),
                    radius,
                    *value,
                    color,
                );
                draw_title(
                    canvas,
                    point.extra.get("title"),
                    options.get("title"),
                    (cx, cy),
                    radius,
                    point.name.as_deref().or(series.name.as_deref()),
                    color,
                );
            }
        };
        if pointer_show_above {
            draw_content(canvas);
        }
        if !anchor_show_above {
            draw_anchor(
                canvas,
                anchor,
                (cx, cy),
                radius,
                item_color(
                    series,
                    values.first().map(|value| value.1),
                    palette,
                    series_index,
                ),
            );
        }
        if pointer_show {
            for (data_index, point, value) in &values {
                let normalized = normalize_value(*value, min, max);
                let angle = start + sweep * normalized;
                draw_pointer(
                    canvas,
                    point.extra.get("pointer"),
                    pointer,
                    (cx, cy),
                    radius,
                    angle,
                    item_color(series, Some(point), palette, series_index + *data_index),
                );
            }
        }

        if anchor_show_above {
            draw_anchor(
                canvas,
                anchor,
                (cx, cy),
                radius,
                item_color(
                    series,
                    values.first().map(|value| value.1),
                    palette,
                    series_index,
                ),
            );
        }
        if !pointer_show_above {
            draw_content(canvas);
        }
    }

    for (order, (data_index, point, value)) in values.iter().enumerate() {
        let normalized = normalize_value(*value, min, max);
        let hit_width = if progress_show && !progress_overlap {
            axis_width / values.len() as f32
        } else {
            progress_width.max(axis_width)
        };
        let hit_radius = if progress_show && !progress_overlap {
            radius - (order as f32 + 0.5) * hit_width
        } else {
            radius - hit_width / 2.0
        };
        hits.push(HitRegion {
            shape: HitShape::Sector {
                cx,
                cy,
                inner: (hit_radius - hit_width / 2.0 - 8.0).max(0.0),
                outer: hit_radius + hit_width / 2.0 + 8.0,
                start: normalize_angle(start),
                sweep: sweep * normalized.max(0.025),
            },
            event: chart_event(
                "gauge",
                series_index,
                *data_index,
                series.name.clone(),
                point,
                cx,
                cy,
            ),
        });
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_axis_line(
    canvas: &ohos_drawing_binding::Canvas,
    center: (f32, f32),
    radius: f32,
    start: f32,
    sweep: f32,
    width: f32,
    round_cap: bool,
    colors: &[(f32, u32)],
) {
    let mut previous = 0.0;
    for (stop, color) in colors {
        let stop = stop.clamp(0.0, 1.0);
        stroke_arc_with_cap(
            canvas,
            center,
            radius - width / 2.0,
            start + sweep * previous,
            sweep * (stop - previous),
            *color,
            width,
            round_cap,
        );
        previous = stop;
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_ticks_and_labels(
    canvas: &ohos_drawing_binding::Canvas,
    options: &std::collections::BTreeMap<String, Value>,
    center: (f32, f32),
    radius: f32,
    axis_width: f32,
    start: f32,
    sweep: f32,
    min: f64,
    max: f64,
    axis_colors: &[(f32, u32)],
) {
    let (cx, cy) = center;
    let split_number = compat::number(options, "splitNumber", 10.0).max(1.0) as usize;
    let axis_tick = options.get("axisTick");
    let split_line = options.get("splitLine");
    let axis_label = options.get("axisLabel");
    let tick_show = nested_bool(axis_tick, &["show"], true);
    let split_show = nested_bool(split_line, &["show"], true);
    let label_show = nested_bool(axis_label, &["show"], true);
    let minor_ticks = nested_number(axis_tick, &["splitNumber"], 5.0).max(1.0) as usize;
    let total_ticks = split_number.saturating_mul(minor_ticks).max(1);

    if tick_show {
        let distance = nested_number(axis_tick, &["distance"], 10.0) as f32;
        let effective_distance = if distance.abs() > f32::EPSILON {
            distance + axis_width
        } else {
            axis_width
        };
        let length = nested_length(axis_tick, &["length"], radius, 6.0).max(0.0);
        let width = nested_number(axis_tick, &["lineStyle", "width"], 1.0).max(0.5) as f32;
        let configured = nested_color(axis_tick, &["lineStyle", "color"]);
        for tick in 0..=total_ticks {
            if tick % minor_ticks == 0 {
                continue;
            }
            let ratio = tick as f32 / total_ticks as f32;
            draw_radial_line(
                canvas,
                center,
                radius - effective_distance,
                length,
                start + sweep * ratio,
                configured.unwrap_or_else(|| color_at(axis_colors, ratio)),
                width,
            );
        }
    }

    for split in 0..=split_number {
        let ratio = split as f32 / split_number as f32;
        let angle = start + sweep * ratio;
        if split_show {
            let distance = nested_number(split_line, &["distance"], 10.0) as f32;
            let effective_distance = if distance.abs() > f32::EPSILON {
                distance + axis_width
            } else {
                axis_width
            };
            let length = nested_length(split_line, &["length"], radius, 10.0).max(0.0);
            let width = nested_number(split_line, &["lineStyle", "width"], 3.0).max(0.5) as f32;
            let configured = nested_color(split_line, &["lineStyle", "color"]);
            draw_radial_line(
                canvas,
                center,
                radius - effective_distance,
                length,
                angle,
                configured.unwrap_or_else(|| color_at(axis_colors, ratio)),
                width,
            );
        }
        if label_show {
            let label_distance = nested_number(axis_label, &["distance"], 15.0) as f32;
            let split_distance = nested_number(split_line, &["distance"], 10.0) as f32;
            let split_length = nested_length(split_line, &["length"], radius, 10.0);
            let label_radius = (radius - split_length - label_distance - split_distance).max(0.0);
            let font_size = nested_number(axis_label, &["fontSize"], 12.0).max(1.0);
            let color = nested_color(axis_label, &["color"])
                .unwrap_or_else(|| color_at(axis_colors, ratio));
            let value = split as f64 * (max - min) / split_number as f64 + min;
            let formatter = nested_string(axis_label, &["formatter"]).unwrap_or("{value}");
            let text = formatter.replace("{value}", &format_value(value));
            let x = cx + angle.cos() * label_radius;
            let y = cy + angle.sin() * label_radius;
            draw_axis_label(
                canvas,
                &text,
                x,
                y,
                angle,
                font_size,
                color,
                axis_label_rotate(axis_label, angle),
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_axis_label(
    canvas: &ohos_drawing_binding::Canvas,
    text: &str,
    x: f32,
    y: f32,
    angle: f32,
    font_size: f64,
    color: u32,
    rotation: f32,
) {
    if rotation.abs() > f32::EPSILON {
        let width = text.chars().count() as f32 * font_size as f32 * 0.56;
        draw_rotated_text(
            canvas,
            text,
            x - width / 2.0,
            y + font_size as f32 * 0.5,
            x,
            y,
            rotation.to_degrees(),
            font_size,
            color,
            400,
        );
        return;
    }
    let unit_x = angle.cos();
    let unit_y = angle.sin();
    let width = text.chars().count() as f32 * font_size as f32 * 0.56;
    let text_x = if unit_x < -0.4 {
        x
    } else if unit_x > 0.4 {
        x - width
    } else {
        x - width / 2.0
    };
    let text_y = if unit_y < -0.8 {
        y + font_size as f32
    } else if unit_y > 0.8 {
        y
    } else {
        y + font_size as f32 * 0.5
    };
    draw_text(canvas, text, text_x, text_y, font_size, color, 400);
}

fn axis_label_rotate(axis_label: Option<&Value>, angle: f32) -> f32 {
    match nested_value(axis_label, &["rotate"]) {
        Some(Value::Number(value)) => value.as_f64().unwrap_or_default().to_radians() as f32,
        Some(Value::String(value)) if value == "tangential" => -angle - std::f32::consts::FRAC_PI_2,
        Some(Value::String(value)) if value == "radial" => {
            let mut rotation = -angle + TAU;
            if rotation > std::f32::consts::FRAC_PI_2 {
                rotation += std::f32::consts::PI;
            }
            rotation
        }
        _ => 0.0,
    }
}

fn draw_radial_line(
    canvas: &ohos_drawing_binding::Canvas,
    center: (f32, f32),
    outer: f32,
    length: f32,
    angle: f32,
    color: u32,
    width: f32,
) {
    stroke_line(
        canvas,
        center.0 + angle.cos() * outer,
        center.1 + angle.sin() * outer,
        center.0 + angle.cos() * (outer - length),
        center.1 + angle.sin() * (outer - length),
        color,
        width,
    );
}

fn draw_pointer(
    canvas: &ohos_drawing_binding::Canvas,
    pointer: Option<&Value>,
    fallback_pointer: Option<&Value>,
    center: (f32, f32),
    radius: f32,
    angle: f32,
    fallback_color: u32,
) {
    let length = compat::length(
        nested_value_fallback(pointer, fallback_pointer, &["length"]),
        radius,
        radius * 0.6,
    );
    let width = nested_number_fallback(pointer, fallback_pointer, &["width"], 6.0).max(1.0) as f32;
    let offset = offset_center_fallback(pointer, fallback_pointer, radius, [0.0, 0.0]);
    let center = (center.0 + offset[0], center.1 + offset[1]);
    let color = nested_color_fallback(pointer, fallback_pointer, &["itemStyle", "color"])
        .unwrap_or(fallback_color);
    let backwards = length * 0.08;
    let perpendicular = (-angle.sin(), angle.cos());
    let base = (
        center.0 - angle.cos() * backwards,
        center.1 - angle.sin() * backwards,
    );
    let mut path = Path::new();
    path.move_to(
        base.0 + perpendicular.0 * width / 2.0,
        base.1 + perpendicular.1 * width / 2.0,
    );
    path.line_to(
        center.0 + angle.cos() * length,
        center.1 + angle.sin() * length,
    );
    path.line_to(
        base.0 - perpendicular.0 * width / 2.0,
        base.1 - perpendicular.1 * width / 2.0,
    );
    path.close();
    fill_path(canvas, &path, color);
    let border_width = nested_number_fallback(
        pointer,
        fallback_pointer,
        &["itemStyle", "borderWidth"],
        0.0,
    ) as f32;
    if border_width > 0.0 {
        let border_color =
            nested_color_fallback(pointer, fallback_pointer, &["itemStyle", "borderColor"])
                .unwrap_or(color);
        stroke_path(canvas, &path, border_color, border_width);
    }
}

fn draw_anchor(
    canvas: &ohos_drawing_binding::Canvas,
    anchor: Option<&Value>,
    center: (f32, f32),
    radius: f32,
    fallback_color: u32,
) {
    if !nested_bool(anchor, &["show"], false) {
        return;
    }
    let size = nested_number(anchor, &["size"], 6.0).max(0.0) as f32;
    let offset = offset_center(anchor, radius, [0.0, 0.0]);
    let center = (center.0 + offset[0], center.1 + offset[1]);
    let color = nested_color(anchor, &["itemStyle", "color"]).unwrap_or(fallback_color);
    fill_circle(canvas, center.0, center.1, size / 2.0, color);
    let border_width = nested_number(anchor, &["itemStyle", "borderWidth"], 0.0) as f32;
    if border_width > 0.0 {
        let border_color =
            nested_color(anchor, &["itemStyle", "borderColor"]).unwrap_or(fallback_color);
        stroke_circle(
            canvas,
            center.0,
            center.1,
            size / 2.0,
            border_color,
            border_width,
        );
    }
}

fn draw_detail(
    canvas: &ohos_drawing_binding::Canvas,
    detail: Option<&Value>,
    fallback_detail: Option<&Value>,
    center: (f32, f32),
    radius: f32,
    value: f64,
    auto_color: u32,
) {
    if !nested_bool_fallback(detail, fallback_detail, &["show"], true) {
        return;
    }
    let formatter =
        nested_string_fallback(detail, fallback_detail, &["formatter"]).unwrap_or("{value}");
    let text = formatter.replace("{value}", &format_value(value));
    let font_size = nested_number_fallback(detail, fallback_detail, &["fontSize"], 30.0).max(1.0);
    let color = nested_color_fallback(detail, fallback_detail, &["color"]).unwrap_or(auto_color);
    let weight = nested_font_weight_fallback(detail, fallback_detail, &["fontWeight"], 700);
    let offset = offset_center_fallback(detail, fallback_detail, radius, [0.0, radius * 0.4]);
    draw_centered_text(
        canvas,
        &text,
        center.0 + offset[0],
        center.1 + offset[1],
        font_size,
        color,
        weight,
    );
}

fn draw_title(
    canvas: &ohos_drawing_binding::Canvas,
    title: Option<&Value>,
    fallback_title: Option<&Value>,
    center: (f32, f32),
    radius: f32,
    name: Option<&str>,
    auto_color: u32,
) {
    if !nested_bool_fallback(title, fallback_title, &["show"], true) {
        return;
    }
    let Some(name) = name else { return };
    let font_size = nested_number_fallback(title, fallback_title, &["fontSize"], 16.0).max(1.0);
    let color = nested_color_fallback(title, fallback_title, &["color"]).unwrap_or(auto_color);
    let weight = nested_font_weight_fallback(title, fallback_title, &["fontWeight"], 400);
    let offset = offset_center_fallback(title, fallback_title, radius, [0.0, radius * 0.2]);
    draw_centered_text(
        canvas,
        name,
        center.0 + offset[0],
        center.1 + offset[1],
        font_size,
        color,
        weight,
    );
}

fn draw_centered_text(
    canvas: &ohos_drawing_binding::Canvas,
    text: &str,
    x: f32,
    y: f32,
    font_size: f64,
    color: u32,
    weight: i32,
) {
    draw_text(
        canvas,
        text,
        x - text.chars().count() as f32 * font_size as f32 * 0.28,
        y + font_size as f32 * 0.35,
        font_size,
        color,
        weight,
    );
}

fn gauge_axis_colors(axis_line: Option<&Value>) -> Vec<(f32, u32)> {
    let mut values = nested_value(axis_line, &["lineStyle", "color"])
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(|value| {
                    let value = value.as_array()?;
                    Some((
                        value.first()?.as_f64()?.clamp(0.0, 1.0) as f32,
                        crate::parser::parse_color(value.get(1)?)?,
                    ))
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if values.is_empty() {
        return vec![(1.0, 0xFFE6EBF8)];
    }
    values.sort_by(|left, right| left.0.total_cmp(&right.0));
    if values.last().is_some_and(|value| value.0 < 1.0) {
        let color = values.last().map(|value| value.1).unwrap_or(0xFFE6EBF8);
        values.push((1.0, color));
    }
    values
}

fn color_at(colors: &[(f32, u32)], ratio: f32) -> u32 {
    colors
        .iter()
        .find(|(stop, _)| ratio <= *stop + f32::EPSILON)
        .or_else(|| colors.last())
        .map(|value| value.1)
        .unwrap_or(0xFF6B7280)
}

fn normalize_value(value: f64, min: f64, max: f64) -> f32 {
    ((value - min) / (max - min)).clamp(0.0, 1.0) as f32
}

fn normalize_progress(value: f64, min: f64, max: f64, clip: bool) -> f32 {
    let normalized = (value - min) / (max - min);
    if clip {
        normalized.clamp(0.0, 1.0) as f32
    } else {
        normalized as f32
    }
}

fn offset_center(value: Option<&Value>, radius: f32, default: [f32; 2]) -> [f32; 2] {
    let Some(values) = nested_value(value, &["offsetCenter"]).and_then(Value::as_array) else {
        return default;
    };
    [
        compat::length(values.first(), radius, default[0]),
        compat::length(values.get(1), radius, default[1]),
    ]
}

fn offset_center_fallback(
    value: Option<&Value>,
    fallback: Option<&Value>,
    radius: f32,
    default: [f32; 2],
) -> [f32; 2] {
    let Some(values) =
        nested_value_fallback(value, fallback, &["offsetCenter"]).and_then(Value::as_array)
    else {
        return default;
    };
    [
        compat::length(values.first(), radius, default[0]),
        compat::length(values.get(1), radius, default[1]),
    ]
}

fn nested_value<'a>(value: Option<&'a Value>, path: &[&str]) -> Option<&'a Value> {
    path.iter().try_fold(value?, |value, key| {
        value.as_object().and_then(|value| value.get(*key))
    })
}

fn nested_value_fallback<'a>(
    value: Option<&'a Value>,
    fallback: Option<&'a Value>,
    path: &[&str],
) -> Option<&'a Value> {
    nested_value(value, path).or_else(|| nested_value(fallback, path))
}

fn nested_number(value: Option<&Value>, path: &[&str], default: f64) -> f64 {
    nested_value(value, path)
        .and_then(Value::as_f64)
        .unwrap_or(default)
}

fn nested_number_fallback(
    value: Option<&Value>,
    fallback: Option<&Value>,
    path: &[&str],
    default: f64,
) -> f64 {
    nested_value_fallback(value, fallback, path)
        .and_then(Value::as_f64)
        .unwrap_or(default)
}

fn nested_length(value: Option<&Value>, path: &[&str], base: f32, default: f32) -> f32 {
    compat::length(nested_value(value, path), base, default)
}

fn nested_bool(value: Option<&Value>, path: &[&str], default: bool) -> bool {
    nested_value(value, path)
        .and_then(Value::as_bool)
        .unwrap_or(default)
}

fn nested_bool_fallback(
    value: Option<&Value>,
    fallback: Option<&Value>,
    path: &[&str],
    default: bool,
) -> bool {
    nested_value_fallback(value, fallback, path)
        .and_then(Value::as_bool)
        .unwrap_or(default)
}

fn nested_string<'a>(value: Option<&'a Value>, path: &[&str]) -> Option<&'a str> {
    nested_value(value, path).and_then(Value::as_str)
}

fn nested_string_fallback<'a>(
    value: Option<&'a Value>,
    fallback: Option<&'a Value>,
    path: &[&str],
) -> Option<&'a str> {
    nested_value_fallback(value, fallback, path).and_then(Value::as_str)
}

fn nested_color(value: Option<&Value>, path: &[&str]) -> Option<u32> {
    nested_value(value, path).and_then(crate::parser::parse_color)
}

fn nested_color_fallback(
    value: Option<&Value>,
    fallback: Option<&Value>,
    path: &[&str],
) -> Option<u32> {
    nested_value_fallback(value, fallback, path).and_then(crate::parser::parse_color)
}

fn nested_font_weight_fallback(
    value: Option<&Value>,
    fallback: Option<&Value>,
    path: &[&str],
    default: i32,
) -> i32 {
    match nested_value_fallback(value, fallback, path) {
        Some(Value::Number(value)) => value.as_i64().unwrap_or(default as i64) as i32,
        Some(Value::String(value)) if value == "bold" || value == "bolder" => 700,
        Some(Value::String(value)) if value == "normal" || value == "lighter" => 400,
        _ => default,
    }
}

fn format_value(value: f64) -> String {
    if (value - value.round()).abs() < 1e-6 {
        format!("{value:.0}")
    } else {
        format!("{value:.2}")
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn gauge_axis_colors_are_sorted_and_cover_the_full_axis() {
        let axis_line = json!({
            "lineStyle": {"color": [[0.7, "#37a2da"], [0.3, "#67e0e3"]]}
        });
        let colors = gauge_axis_colors(Some(&axis_line));
        assert_eq!(
            colors.iter().map(|value| value.0).collect::<Vec<_>>(),
            vec![0.3, 0.7, 1.0]
        );
        assert_eq!(colors[1].1, colors[2].1);
    }

    #[test]
    fn gauge_offsets_accept_pixels_and_percentages() {
        let value = json!({"offsetCenter": [12, "40%"]});
        assert_eq!(offset_center(Some(&value), 100.0, [0.0, 0.0]), [12.0, 40.0]);
    }

    #[test]
    fn gauge_progress_clip_matches_echarts_semantics() {
        assert_eq!(normalize_progress(140.0, 0.0, 100.0, true), 1.0);
        assert!((normalize_progress(140.0, 0.0, 100.0, false) - 1.4).abs() < 1e-6);
    }

    #[test]
    fn data_item_options_inherit_missing_series_fields() {
        let item = json!({"fontSize": 18});
        let series = json!({"fontSize": 30, "color": "#5470c6"});
        assert_eq!(
            nested_number_fallback(Some(&item), Some(&series), &["fontSize"], 12.0),
            18.0
        );
        assert_eq!(
            nested_color_fallback(Some(&item), Some(&series), &["color"]),
            Some(0xFF5470C6)
        );
    }
}
