use std::time::{SystemTime, UNIX_EPOCH};

use super::super::prelude::*;
use super::super::surface::stroke_path_style;
use super::super::symbol::{draw_symbol, SymbolSpec};

pub(super) fn render(series: &LinesSeries, context: &mut FreeRenderContext<'_>) {
    let series_index = context.series_index;
    let plot = context.plot;
    let coordinates = series
        .data
        .iter()
        .flat_map(segment_coordinates)
        .collect::<Vec<_>>();
    let min_x = coordinates
        .iter()
        .map(|point| point.0)
        .reduce(f64::min)
        .unwrap_or(0.0);
    let max_x = coordinates
        .iter()
        .map(|point| point.0)
        .reduce(f64::max)
        .unwrap_or(1.0);
    let min_y = coordinates
        .iter()
        .map(|point| point.1)
        .reduce(f64::min)
        .unwrap_or(0.0);
    let max_y = coordinates
        .iter()
        .map(|point| point.1)
        .reduce(f64::max)
        .unwrap_or(1.0);
    let geo_index = super::geo_index(&context.option.series[series_index]);
    let geo_transform = geo_index
        .and_then(|index| super::map::transform_from_geo_component(context.option, plot, index));
    if let (Some(index), Some(_)) = (geo_index, geo_transform) {
        if super::should_draw_geo_base(context.option, series_index, index) {
            super::map::draw_geo_component(context.option, plot, index, context.canvas);
        }
    }
    let project = |point: (f64, f64)| {
        geo_transform.map_or_else(
            || {
                (
                    plot.x + ((point.0 - min_x) / (max_x - min_x).max(1e-12)) as f32 * plot.width,
                    plot.y + plot.height
                        - ((point.1 - min_y) / (max_y - min_y).max(1e-12)) as f32 * plot.height,
                )
            },
            |transform| transform.project(point),
        )
    };
    let line_color = series
        .options
        .line_style
        .color
        .unwrap_or_else(|| color(context.palette, series_index));
    let effect = series
        .options
        .extra
        .get("effect")
        .and_then(serde_json::Value::as_object);
    let effect_show = effect
        .and_then(|effect| effect.get("show"))
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    let animation_time = effect_show.then(|| {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0.0, |duration| duration.as_secs_f64())
    });

    for (index, segment) in series.data.iter().enumerate() {
        let points = segment_coordinates(segment)
            .map(project)
            .collect::<Vec<_>>();
        if points.len() < 2 {
            continue;
        }
        let width = series.options.line_style.width.max(1.0) * segment.value.max(0.25) as f32;
        if let Some(canvas) = context.canvas {
            let mut path = Path::new();
            path.move_to(points[0].0, points[0].1);
            for point in &points[1..] {
                path.line_to(point.0, point.1);
            }
            stroke_path_style(
                canvas,
                &path,
                with_opacity(line_color, series.options.line_style.opacity),
                width,
                &series.options.line_style.kind,
            );
            draw_end_symbol(canvas, series, &points, line_color);
            if effect_show {
                draw_effect(
                    canvas,
                    effect,
                    &points,
                    line_color,
                    animation_time.unwrap_or_default(),
                );
            }
        }
        let center = point_along(&points, 0.5).unwrap_or(points[0]);
        context.hits.push(HitRegion {
            shape: HitShape::MultiPolygon {
                polygons: stroke_polygons(&points, width.max(10.0)),
            },
            event: ChartEvent {
                series_index,
                data_index: index,
                series_name: series.name.clone(),
                name: segment.name.clone(),
                value: vec![segment.value],
                x: center.0,
                y: center.1,
                component_type: String::from("lines"),
            },
        });
    }
}

fn segment_coordinates(segment: &LineSegment) -> Box<dyn Iterator<Item = (f64, f64)> + '_> {
    if segment.coords.len() >= 2 {
        Box::new(segment.coords.iter().copied())
    } else {
        Box::new(std::iter::once(segment.from).chain(std::iter::once(segment.to)))
    }
}

fn draw_end_symbol(
    canvas: &ohos_drawing_binding::Canvas,
    series: &LinesSeries,
    points: &[(f32, f32)],
    color: u32,
) {
    let symbol = match series.options.extra.get("symbol") {
        Some(serde_json::Value::Array(values)) => values.get(1).and_then(serde_json::Value::as_str),
        Some(value) => value.as_str(),
        None => None,
    }
    .unwrap_or("none");
    if symbol == "none" {
        return;
    }
    let end = points[points.len() - 1];
    let previous = points[points.len() - 2];
    let angle = (end.1 - previous.1).atan2(end.0 - previous.0).to_degrees() + 90.0;
    let size = series.options.symbol_size.max(4.0);
    draw_symbol(
        canvas,
        &SymbolSpec {
            name: symbol,
            size: series
                .options
                .symbol_size_dimensions
                .unwrap_or([size, size]),
            rotate: angle,
            offset: [0.0, 0.0],
        },
        end.0,
        end.1,
        color,
        None,
    );
}

fn draw_effect(
    canvas: &ohos_drawing_binding::Canvas,
    effect: Option<&serde_json::Map<String, serde_json::Value>>,
    points: &[(f32, f32)],
    fallback_color: u32,
    animation_time: f64,
) {
    let period = effect
        .and_then(|effect| effect.get("period"))
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(4.0)
        .max(0.1);
    let progress = (animation_time / period).fract() as f32;
    let Some(position) = point_along(points, progress) else {
        return;
    };
    let symbol = effect
        .and_then(|effect| effect.get("symbol"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or("circle");
    let size = effect
        .and_then(|effect| effect.get("symbolSize"))
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(4.0)
        .max(1.0) as f32;
    let color = effect
        .and_then(|effect| effect.get("color"))
        .and_then(crate::parser::parse_color)
        .unwrap_or(fallback_color);
    draw_symbol(
        canvas,
        &SymbolSpec {
            name: symbol,
            size: [size, size],
            rotate: 0.0,
            offset: [0.0, 0.0],
        },
        position.0,
        position.1,
        color,
        None,
    );
}

fn point_along(points: &[(f32, f32)], progress: f32) -> Option<(f32, f32)> {
    let lengths = points
        .windows(2)
        .map(|pair| ((pair[1].0 - pair[0].0).powi(2) + (pair[1].1 - pair[0].1).powi(2)).sqrt())
        .collect::<Vec<_>>();
    let total = lengths.iter().sum::<f32>();
    if total <= f32::EPSILON {
        return points.first().copied();
    }
    let mut remaining = total * progress.clamp(0.0, 1.0);
    for (pair, length) in points.windows(2).zip(lengths) {
        if remaining <= length {
            let ratio = remaining / length.max(1e-6);
            return Some((
                pair[0].0 + (pair[1].0 - pair[0].0) * ratio,
                pair[0].1 + (pair[1].1 - pair[0].1) * ratio,
            ));
        }
        remaining -= length;
    }
    points.last().copied()
}

fn stroke_polygons(points: &[(f32, f32)], width: f32) -> Vec<HitPolygon> {
    points
        .windows(2)
        .filter_map(|pair| {
            let dx = pair[1].0 - pair[0].0;
            let dy = pair[1].1 - pair[0].1;
            let length = (dx * dx + dy * dy).sqrt();
            if length <= f32::EPSILON {
                return None;
            }
            let normal = (-dy / length * width / 2.0, dx / length * width / 2.0);
            Some(HitPolygon {
                exterior: vec![
                    (pair[0].0 + normal.0, pair[0].1 + normal.1),
                    (pair[1].0 + normal.0, pair[1].1 + normal.1),
                    (pair[1].0 - normal.0, pair[1].1 - normal.1),
                    (pair[0].0 - normal.0, pair[0].1 - normal.1),
                ],
                holes: Vec::new(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn point_along_uses_total_polyline_length() {
        let points = [(0.0, 0.0), (10.0, 0.0), (10.0, 30.0)];
        assert_eq!(point_along(&points, 0.5), Some((10.0, 10.0)));
    }
}
