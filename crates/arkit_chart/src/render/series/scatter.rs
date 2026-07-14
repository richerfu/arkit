use ohos_drawing_binding::{Canvas, Rect};

use super::super::compat;
use super::super::label_layout::draw_rotated_text;
use super::super::prelude::*;
use super::super::symbol::{draw_symbol, resolve_symbol, SymbolSpec};

pub(super) fn render(series: &BasicSeries, context: &mut CartesianRenderContext<'_>) {
    render_impl(series, context, false);
}

pub(super) fn render_effect(series: &BasicSeries, context: &mut CartesianRenderContext<'_>) {
    render_impl(series, context, true);
}

pub(super) fn render_free(series: &BasicSeries, context: &mut FreeRenderContext<'_>, effect: bool) {
    let coordinate_system = series
        .options
        .extra
        .get("coordinateSystem")
        .and_then(serde_json::Value::as_str);
    match coordinate_system {
        Some("singleAxis") => render_single_axis(series, context, effect),
        Some("geo") => render_geo(series, context, effect),
        _ => render_polar(series, context, effect),
    }
}

fn render_geo(series: &BasicSeries, context: &mut FreeRenderContext<'_>, effect: bool) {
    let series_index = context.series_index;
    let geo_index = super::geo_index(&context.option.series[series_index]).unwrap_or(0);
    let Some(transform) =
        super::map::transform_from_geo_component(context.option, context.plot, geo_index)
    else {
        return;
    };
    if super::should_draw_geo_base(context.option, series_index, geo_index) {
        super::map::draw_geo_component(context.option, context.plot, geo_index, context.canvas);
    }
    let animation_time = effect.then(crate::animation::animation_time_seconds);
    for (index, point) in series.data.iter().enumerate() {
        let (Some(longitude), Some(latitude)) = (point.number_opt(0), point.number_opt(1)) else {
            continue;
        };
        let position = transform.project((longitude, latitude));
        let (visual_size, visual_color) =
            visual_encoding(point, context.option.visual_map_for_series(series_index));
        let symbol = resolve_symbol(series, point, visual_size);
        let fill = effective_item_style(series, Some(point))
            .color
            .map(|_| item_color(series, Some(point), context.palette, series_index))
            .or(visual_color)
            .unwrap_or_else(|| item_color(series, Some(point), context.palette, series_index));
        if let Some(canvas) = context.canvas {
            if effect {
                draw_ripples(
                    canvas,
                    series,
                    &symbol,
                    position.0,
                    position.1,
                    fill,
                    animation_time.unwrap_or_default(),
                );
            }
            draw_symbol(
                canvas,
                &symbol,
                position.0,
                position.1,
                fill,
                border(series, Some(point)),
            );
        }
        context.hits.push(point_hit(
            if effect { "effectScatter" } else { "scatter" },
            series_index,
            index,
            series.name.clone(),
            point,
            symbol.center(position.0, position.1),
            symbol.hit_radius().max(9.0),
        ));
    }
}

pub(super) fn render_polar(
    series: &BasicSeries,
    context: &mut FreeRenderContext<'_>,
    effect: bool,
) {
    let series_index = context.series_index;
    let polar_index = series
        .options
        .extra
        .get("polarIndex")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0) as usize;
    let config =
        super::line::PolarConfig::from_option(context.option, polar_index, context.plot, series);
    let animation_time = effect.then(crate::animation::animation_time_seconds);
    if let Some(canvas) = context.canvas {
        config.draw_axes(canvas, context.option.visual_style.text_color);
    }
    for (index, point) in series.data.iter().enumerate() {
        let paired = point.values.len() > 1;
        let Some(radius_value) = point.number_opt(0) else {
            continue;
        };
        let angle_value = paired.then(|| point.number_opt(1)).flatten();
        let radius = config.radius_for(radius_value, index);
        let angle = config.angle_for(angle_value, index);
        let position = config.project(radius, angle);
        let (visual_size, visual_color) =
            visual_encoding(point, context.option.visual_map_for_series(series_index));
        let symbol = resolve_symbol(series, point, visual_size);
        let fill = effective_item_style(series, Some(point))
            .color
            .map(|_| item_color(series, Some(point), context.palette, series_index))
            .or(visual_color)
            .unwrap_or_else(|| item_color(series, Some(point), context.palette, series_index));
        if let Some(canvas) = context.canvas {
            if effect {
                draw_ripples(
                    canvas,
                    series,
                    &symbol,
                    position.0,
                    position.1,
                    fill,
                    animation_time.unwrap_or_default(),
                );
            }
            draw_symbol(
                canvas,
                &symbol,
                position.0,
                position.1,
                fill,
                border(series, Some(point)),
            );
            let label = effective_label(series, point);
            if label.show {
                set_next_data_index(index);
                draw_label(
                    canvas,
                    &format_label(&label, series, point, index),
                    &label,
                    &symbol,
                    position.0,
                    position.1,
                );
            }
        }
        context.hits.push(point_hit(
            if effect { "effectScatter" } else { "scatter" },
            series_index,
            index,
            series.name.clone(),
            point,
            symbol.center(position.0, position.1),
            symbol.hit_radius().max(9.0),
        ));
    }
}

fn render_single_axis(series: &BasicSeries, context: &mut FreeRenderContext<'_>, effect: bool) {
    let axis_index = series
        .options
        .extra
        .get("singleAxisIndex")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0) as usize;
    let axis = option_component(&context.option.extra, "singleAxis", axis_index);
    let orient = axis
        .and_then(|axis| axis.get("orient"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or("horizontal");
    let full = context.plot;
    let left = axis
        .and_then(|axis| axis.get("left"))
        .map(|value| compat::position(Some(value), full.x, full.width, full.x + full.width * 0.1))
        .unwrap_or(full.x + full.width * 0.1);
    let top = axis
        .and_then(|axis| axis.get("top"))
        .map(|value| compat::position(Some(value), full.y, full.height, full.y + full.height * 0.5))
        .unwrap_or(full.y + full.height * 0.5);
    let width = axis
        .and_then(|axis| axis.get("width"))
        .map(|value| compat::length(Some(value), full.width, full.width * 0.8))
        .unwrap_or(full.width * 0.8);
    let height = axis
        .and_then(|axis| axis.get("height"))
        .map(|value| compat::length(Some(value), full.height, full.height * 0.8))
        .unwrap_or(full.height * 0.8);
    let labels = axis
        .and_then(|axis| axis.get("data"))
        .and_then(serde_json::Value::as_array)
        .map(|values| values.iter().map(ToString::to_string).collect::<Vec<_>>())
        .unwrap_or_default();
    let values = series
        .data
        .iter()
        .filter_map(|point| point.number_opt(0))
        .collect::<Vec<_>>();
    let mut min = values
        .iter()
        .copied()
        .reduce(f64::min)
        .unwrap_or(0.0)
        .min(0.0);
    let mut max = values
        .iter()
        .copied()
        .reduce(f64::max)
        .unwrap_or(1.0)
        .max(0.0);
    min = axis
        .and_then(|axis| axis.get("min"))
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(min);
    max = axis
        .and_then(|axis| axis.get("max"))
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(max);
    if (max - min).abs() < f64::EPSILON {
        max = min + 1.0;
    }
    let inverse = axis
        .and_then(|axis| axis.get("inverse"))
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    let position_for = |value: f64, index: usize| {
        let mut normalized = if labels.is_empty() {
            ((value - min) / (max - min)).clamp(0.0, 1.0) as f32
        } else {
            index as f32 / labels.len().saturating_sub(1).max(1) as f32
        };
        if inverse {
            normalized = 1.0 - normalized;
        }
        if orient == "vertical" {
            (left, top + height * (1.0 - normalized))
        } else {
            (left + width * normalized, top)
        }
    };
    let split_number = axis
        .and_then(|axis| axis.get("splitNumber"))
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(5)
        .max(1) as usize;
    if let Some(canvas) = context.canvas {
        if orient == "vertical" {
            stroke_line(canvas, left, top, left, top + height, 0xFF94A3B8, 1.0);
        } else {
            stroke_line(canvas, left, top, left + width, top, 0xFF94A3B8, 1.0);
        }
        let ticks = labels.len().max(split_number + 1);
        for index in 0..ticks {
            let value = min + (max - min) * index as f64 / ticks.saturating_sub(1).max(1) as f64;
            let position = position_for(value, index);
            if orient == "vertical" {
                stroke_line(
                    canvas,
                    position.0 - 4.0,
                    position.1,
                    position.0 + 4.0,
                    position.1,
                    0xFF94A3B8,
                    1.0,
                );
            } else {
                stroke_line(
                    canvas,
                    position.0,
                    position.1 - 4.0,
                    position.0,
                    position.1 + 4.0,
                    0xFF94A3B8,
                    1.0,
                );
            }
            let label = labels
                .get(index)
                .cloned()
                .unwrap_or_else(|| format_axis_value(value));
            draw_text(
                canvas,
                label.trim_matches('"'),
                if orient == "vertical" {
                    position.0 + 7.0
                } else {
                    position.0 - 8.0
                },
                if orient == "vertical" {
                    position.1 + 3.0
                } else {
                    position.1 + 17.0
                },
                10.0,
                0xFF64748B,
                400,
            );
        }
    }
    let animation_time = effect.then(crate::animation::animation_time_seconds);
    for (index, point) in series.data.iter().enumerate() {
        let Some(value) = point.number_opt(0) else {
            continue;
        };
        let position = position_for(value, index);
        let (visual_size, visual_color) = visual_encoding(
            point,
            context.option.visual_map_for_series(context.series_index),
        );
        let symbol = resolve_symbol(series, point, visual_size);
        let fill = visual_color.unwrap_or_else(|| {
            item_color(series, Some(point), context.palette, context.series_index)
        });
        if let Some(canvas) = context.canvas {
            if effect {
                draw_ripples(
                    canvas,
                    series,
                    &symbol,
                    position.0,
                    position.1,
                    fill,
                    animation_time.unwrap_or_default(),
                );
            }
            draw_symbol(
                canvas,
                &symbol,
                position.0,
                position.1,
                fill,
                border(series, Some(point)),
            );
        }
        context.hits.push(point_hit(
            if effect { "effectScatter" } else { "scatter" },
            context.series_index,
            index,
            series.name.clone(),
            point,
            symbol.center(position.0, position.1),
            symbol.hit_radius().max(9.0),
        ));
    }
}

fn option_component<'a>(
    extra: &'a std::collections::BTreeMap<String, serde_json::Value>,
    key: &str,
    index: usize,
) -> Option<&'a serde_json::Map<String, serde_json::Value>> {
    match extra.get(key)? {
        serde_json::Value::Array(values) => values.get(index)?.as_object(),
        value if index == 0 => value.as_object(),
        _ => None,
    }
}

fn format_axis_value(value: f64) -> String {
    if (value - value.round()).abs() < 1e-6 {
        format!("{value:.0}")
    } else {
        format!("{value:.2}")
    }
}

fn render_impl(series: &BasicSeries, context: &mut CartesianRenderContext<'_>, effect: bool) {
    let series_index = context.series_index;
    let plot = context.plot;
    let layout = context.layout;
    let palette = context.palette;
    let canvas = context.canvas;
    let visual_map = context.visual_map;
    let hits = &mut *context.hits;
    let animation_time = effect.then(crate::animation::animation_time_seconds);

    if let Some(canvas) = canvas {
        if series.options.clip {
            begin_clip(canvas, plot);
        }
    }
    for (index, point) in series.data.iter().enumerate() {
        let paired = point.values.len() > 1;
        let x_value = if paired {
            let Some(value) = point.number_opt(0) else {
                continue;
            };
            Some(value)
        } else {
            None
        };
        let y_value = if paired {
            point.number_opt(1)
        } else {
            point.number_opt(0)
        };
        let Some(y_value) = y_value else { continue };
        if !layout.x.contains(x_value, index) || !layout.y.contains(Some(y_value), index) {
            continue;
        }
        let x = layout.x.position(plot, x_value, index, false);
        let y = layout.y.position(plot, Some(y_value), index, true);
        let (visual_size, visual_color) = visual_encoding(point, visual_map);
        let symbol = resolve_symbol(series, point, visual_size);
        let fill = effective_item_style(series, Some(point))
            .color
            .map(|_| item_color(series, Some(point), palette, series_index))
            .or(visual_color)
            .unwrap_or_else(|| item_color(series, Some(point), palette, series_index));
        if let Some(canvas) = canvas {
            if effect {
                draw_ripples(
                    canvas,
                    series,
                    &symbol,
                    x,
                    y,
                    fill,
                    animation_time.unwrap_or_default(),
                );
            }
            draw_symbol(canvas, &symbol, x, y, fill, border(series, Some(point)));
            let label = effective_label(series, point);
            if label.show {
                set_next_data_index(index);
                draw_label(
                    canvas,
                    &format_label(&label, series, point, index),
                    &label,
                    &symbol,
                    x,
                    y,
                );
            }
        }
        hits.push(point_hit(
            if effect { "effectScatter" } else { "scatter" },
            series_index,
            index,
            series.name.clone(),
            point,
            symbol.center(x, y),
            symbol.hit_radius().max(9.0),
        ));
    }
    if let Some(canvas) = canvas {
        if series.options.clip {
            canvas.restore();
        }
    }
}

fn visual_encoding(
    point: &DataPoint,
    visual_map: Option<&VisualMap>,
) -> (Option<[f32; 2]>, Option<u32>) {
    let Some(visual_map) = visual_map else {
        return (None, None);
    };
    let dimension = visual_map
        .dimension
        .unwrap_or_else(|| point.values.len().saturating_sub(1));
    let Some(value) = point.number_opt(dimension) else {
        return (None, None);
    };
    let size = visual_map_symbol_size(visual_map, value);
    let color = (!visual_map.colors.is_empty() || !visual_map.pieces.is_empty())
        .then(|| visual_map_color(visual_map, value));
    (size, color)
}

fn draw_ripples(
    canvas: &Canvas,
    series: &BasicSeries,
    symbol: &SymbolSpec<'_>,
    x: f32,
    y: f32,
    fallback_color: u32,
    animation_time: f64,
) {
    let options = series
        .options
        .extra
        .get("rippleEffect")
        .and_then(serde_json::Value::as_object);
    let number = options
        .and_then(|value| value.get("number"))
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(3)
        .clamp(1, 10) as usize;
    let scale = options
        .and_then(|value| value.get("scale"))
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(2.5)
        .max(1.0) as f32;
    let period = options
        .and_then(|value| value.get("period"))
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(4.0)
        .max(0.1);
    let brush_type = options
        .and_then(|value| value.get("brushType"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or("fill");
    let color = options
        .and_then(|value| value.get("color"))
        .and_then(crate::parser::parse_color)
        .unwrap_or(fallback_color);
    let center = symbol.center(x, y);
    let phase = (animation_time / period).fract() as f32;
    for ring in 0..number {
        let progress = (phase + ring as f32 / number as f32).fract();
        let factor = 1.0 + (scale - 1.0) * progress;
        let width = symbol.size[0] * factor;
        let height = symbol.size[1] * factor;
        let alpha = if brush_type == "stroke" {
            0.42 * (1.0 - progress * 0.65)
        } else {
            0.16 * (1.0 - progress * 0.7)
        };
        let color = with_opacity(color, alpha);
        if brush_type == "stroke" {
            stroke_oval(
                canvas,
                center.0 - width / 2.0,
                center.1 - height / 2.0,
                width,
                height,
                color,
                1.5,
            );
        } else {
            fill_oval(
                canvas,
                center.0 - width / 2.0,
                center.1 - height / 2.0,
                width,
                height,
                color,
            );
        }
    }
}

fn draw_label(
    canvas: &Canvas,
    text: &str,
    label: &LabelStyle,
    symbol: &SymbolSpec<'_>,
    x: f32,
    y: f32,
) {
    let (x, y) = symbol.center(x, y);
    let text_width = text.chars().count() as f32 * label.font_size * 0.55;
    let half_width = symbol.size[0] / 2.0;
    let half_height = symbol.size[1] / 2.0;
    let (mut text_x, mut text_y) = match label.position.as_str() {
        "bottom" => (
            x - text_width / 2.0,
            y + half_height + label.distance + label.font_size,
        ),
        "left" => (
            x - half_width - label.distance - text_width,
            y + label.font_size * 0.35,
        ),
        "right" => (x + half_width + label.distance, y + label.font_size * 0.35),
        "inside" => (x - text_width / 2.0, y + label.font_size * 0.35),
        _ => (x - text_width / 2.0, y - half_height - label.distance),
    };
    text_x += label.offset[0];
    text_y += label.offset[1];
    draw_rotated_text(
        canvas,
        text,
        text_x,
        text_y,
        text_x,
        text_y,
        label.rotate,
        label.font_size as f64,
        label.color.unwrap_or(0xFF333333),
        label.font_weight,
    );
}

fn begin_clip(canvas: &Canvas, plot: &crate::render::geometry::Plot) {
    canvas.save();
    let rect = Rect::new(plot.x, plot.y, plot.x + plot.width, plot.y + plot.height);
    // SAFETY: canvas and rect are live for the synchronous clip call; the
    // caller restores the saved canvas state after rendering the series.
    unsafe {
        ohos_native_drawing_sys::OH_Drawing_CanvasClipRect(
            canvas.as_ptr(),
            rect.as_ptr(),
            ohos_native_drawing_sys::OH_Drawing_CanvasClipOp_INTERSECT,
            true,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visual_map_resolves_scatter_color_and_symbol_size_from_dimension() {
        let point = DataPoint::values([2.0, 4.0, 75.0]);
        let visual_map = VisualMap {
            show: false,
            min: 0.0,
            max: 100.0,
            colors: vec![0xFF000000, 0xFFFFFFFF],
            dimension: Some(2),
            symbol_size_range: Some([10.0, 30.0]),
            pieces: Vec::new(),
            series_indices: Vec::new(),
        };
        let (size, color) = visual_encoding(&point, Some(&visual_map));
        assert_eq!(size, Some([25.0, 25.0]));
        assert!(color.is_some());
    }
}
