use ohos_drawing_binding::Canvas;

use super::super::compat;
use super::super::geometry::Plot;
use super::super::label_layout::{
    draw_rotated_text, set_next_data_index, set_next_label_line_points, take_last_label_line_points,
};
use super::super::prelude::*;
use super::super::surface::stroke_path_style;

struct PieSector<'a> {
    index: usize,
    point: &'a DataPoint,
    percentage: f64,
    center: (f32, f32),
    inner: f32,
    outer: f32,
    start: f32,
    sweep: f32,
    mid: f32,
    fill: u32,
}

struct OutsideLabel {
    data_index: usize,
    text: String,
    label: LabelStyle,
    side: f32,
    edge: (f32, f32),
    bend: (f32, f32),
    end_x: f32,
    y: f32,
    guide_color: u32,
    guide_width: f32,
    guide_type: String,
    show_guide: bool,
}

pub(super) fn render(series: &BasicSeries, context: &mut FreeRenderContext<'_>) {
    let series_index = context.series_index;
    let plot = context.plot;
    let palette = context.palette;
    let canvas = context.canvas;
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
    let radius = options.get("radius");
    let (inner, outer) = radius
        .and_then(serde_json::Value::as_array)
        .and_then(|values| Some((values.first()?, values.get(1)?)))
        .map(|(inner, outer)| {
            (
                compat::length(Some(inner), radius_base, 0.0),
                compat::length(Some(outer), radius_base, radius_base * 0.75),
            )
        })
        .unwrap_or_else(|| (0.0, compat::length(radius, radius_base, radius_base * 0.75)));
    let (inner, outer) = if inner <= outer {
        (inner, outer.max(inner + 1.0))
    } else {
        (outer, inner.max(outer + 1.0))
    };
    let values = series
        .data
        .iter()
        .enumerate()
        .filter_map(|(index, point)| Some((index, point, point.number_opt(0)?.max(0.0))))
        .collect::<Vec<_>>();
    if values.is_empty() {
        return;
    }
    let total = values.iter().map(|(_, _, value)| *value).sum::<f64>();
    let still_show_zero_sum = compat::boolean(options, "stillShowZeroSum", true);
    if total <= f64::EPSILON && !still_show_zero_sum {
        return;
    }
    let max_value = values
        .iter()
        .map(|(_, _, value)| *value)
        .reduce(f64::max)
        .unwrap_or(1.0)
        .max(1.0);
    let clockwise = compat::boolean(options, "clockwise", true);
    let direction = if clockwise { 1.0 } else { -1.0 };
    let start_degrees = compat::number(options, "startAngle", 90.0);
    let start = -(start_degrees as f32).to_radians();
    let span = pie_span(start_degrees, options.get("endAngle"), clockwise);
    if span <= f32::EPSILON {
        return;
    }
    let pad = (compat::number(options, "padAngle", 0.0) as f32)
        .to_radians()
        .max(0.0);
    let min_angle = (compat::number(options, "minAngle", 0.0) as f32)
        .to_radians()
        .max(0.0);
    let min_show_label_angle = compat::number(options, "minShowLabelAngle", 0.0) as f32;
    let rose_type = compat::string(options, "roseType");
    let weights = values
        .iter()
        .map(|(_, _, value)| {
            if rose_type == Some("area") || total <= f64::EPSILON {
                1.0
            } else {
                *value
            }
        })
        .collect::<Vec<_>>();
    let angles = allocate_angles(&weights, span, min_angle);
    let selected_offset = compat::number(options, "selectedOffset", 10.0).max(0.0) as f32;
    let percent_precision =
        compat::number(options, "percentPrecision", 2.0).clamp(0.0, 20.0) as usize;
    let mut cursor = start;
    let mut sectors = Vec::with_capacity(values.len());

    for (((index, point, value), allocated), weight_index) in values
        .iter()
        .zip(angles.iter().copied())
        .zip(0..values.len())
    {
        let raw_sweep = allocated * direction;
        let sweep = (allocated - pad).max(0.0) * direction;
        let sector_start = cursor + pad.min(allocated) * 0.5 * direction;
        let mid = cursor + raw_sweep * 0.5;
        let item_outer = match rose_type {
            Some("radius") => inner + (outer - inner) * (*value / max_value) as f32,
            Some("area") => inner + (outer - inner) * (*value / max_value).sqrt() as f32,
            _ => outer,
        };
        let selected = context.selected_items.contains(&(series_index, *index));
        let offset = if selected { selected_offset } else { 0.0 };
        let item_center = (cx + mid.cos() * offset, cy + mid.sin() * offset);
        sectors.push(PieSector {
            index: *index,
            point,
            percentage: if total > f64::EPSILON {
                *value / total * 100.0
            } else {
                100.0 / values.len() as f64
            },
            center: item_center,
            inner,
            outer: item_outer,
            start: sector_start,
            sweep,
            mid,
            fill: item_color(series, Some(point), palette, weight_index),
        });
        cursor += raw_sweep;
    }

    let mut outside_labels = Vec::new();
    for sector in &sectors {
        if let Some(canvas) = canvas {
            fill_ring_sector(
                canvas,
                sector.center,
                (sector.inner, sector.outer),
                sector.start,
                sector.sweep,
                sector.fill,
            );
            if let Some((border_color, border_width)) = border(series, Some(sector.point)) {
                stroke_ring_sector(
                    canvas,
                    sector.center,
                    (sector.inner, sector.outer),
                    sector.start,
                    sector.sweep,
                    border_color,
                    border_width,
                );
            }
            let label = effective_label(series, sector.point);
            if label.show && sector.sweep.abs().to_degrees() >= min_show_label_angle {
                let text = format_pie_label(
                    &label,
                    series,
                    sector.point,
                    sector.index,
                    sector.percentage,
                    percent_precision,
                );
                if matches!(label.position.as_str(), "inside" | "inner" | "center") {
                    set_next_data_index(sector.index);
                    draw_inside_label(canvas, sector, &label, &text, (cx, cy));
                } else {
                    outside_labels.push(outside_label(series, sector, label, text));
                }
            }
        }
        let event_radius = (sector.inner + sector.outer) / 2.0;
        let event_x = sector.center.0 + sector.mid.cos() * event_radius;
        let event_y = sector.center.1 + sector.mid.sin() * event_radius;
        context.hits.push(HitRegion {
            shape: HitShape::Sector {
                cx: sector.center.0,
                cy: sector.center.1,
                inner: sector.inner,
                outer: sector.outer,
                start: normalize_angle(sector.start),
                sweep: sector.sweep,
            },
            event: chart_event(
                "pie",
                series_index,
                sector.index,
                series.name.clone(),
                sector.point,
                event_x,
                event_y,
            ),
        });
    }
    if let Some(canvas) = canvas {
        if compat::boolean(options, "avoidLabelOverlap", true) {
            avoid_label_overlap(&mut outside_labels, plot);
        }
        for label in &outside_labels {
            draw_outside_label(canvas, label);
        }
    }
}

fn pie_span(start_degrees: f64, end_angle: Option<&serde_json::Value>, clockwise: bool) -> f32 {
    let Some(end_degrees) = end_angle.and_then(serde_json::Value::as_f64) else {
        return TAU;
    };
    let raw = if clockwise {
        start_degrees - end_degrees
    } else {
        end_degrees - start_degrees
    };
    if raw.abs() >= 360.0 && raw.rem_euclid(360.0).abs() < 1e-9 {
        TAU
    } else {
        raw.rem_euclid(360.0).to_radians() as f32
    }
}

fn allocate_angles(weights: &[f64], span: f32, minimum: f32) -> Vec<f32> {
    if weights.is_empty() {
        return Vec::new();
    }
    let minimum = minimum.min(span / weights.len() as f32);
    let mut output = vec![0.0; weights.len()];
    let mut remaining = (0..weights.len()).collect::<Vec<_>>();
    let mut remaining_span = span;
    loop {
        let total_weight = remaining
            .iter()
            .map(|index| weights[*index].max(0.0))
            .sum::<f64>();
        let mut fixed = Vec::new();
        for index in &remaining {
            let angle = if total_weight <= f64::EPSILON {
                remaining_span / remaining.len().max(1) as f32
            } else {
                remaining_span * (weights[*index].max(0.0) / total_weight) as f32
            };
            if angle + f32::EPSILON < minimum {
                output[*index] = minimum;
                remaining_span = (remaining_span - minimum).max(0.0);
                fixed.push(*index);
            }
        }
        if fixed.is_empty() {
            let total_weight = remaining
                .iter()
                .map(|index| weights[*index].max(0.0))
                .sum::<f64>();
            for index in remaining {
                output[index] = if total_weight <= f64::EPSILON {
                    remaining_span / weights.len() as f32
                } else {
                    remaining_span * (weights[index].max(0.0) / total_weight) as f32
                };
            }
            break;
        }
        remaining.retain(|index| !fixed.contains(index));
        if remaining.is_empty() {
            break;
        }
    }
    output
}

fn draw_inside_label(
    canvas: &Canvas,
    sector: &PieSector<'_>,
    label: &LabelStyle,
    text: &str,
    chart_center: (f32, f32),
) {
    let text_width = text.chars().count() as f32 * label.font_size * 0.55;
    let (x, y) = if label.position == "center" {
        chart_center
    } else {
        let radius = (sector.inner + sector.outer) / 2.0;
        (
            sector.center.0 + sector.mid.cos() * radius,
            sector.center.1 + sector.mid.sin() * radius,
        )
    };
    draw_rotated_text(
        canvas,
        text,
        x - text_width / 2.0 + label.offset[0],
        y + label.font_size * 0.35 + label.offset[1],
        x,
        y,
        label.rotate,
        label.font_size as f64,
        label.color.unwrap_or(0xFFFFFFFF),
        label.font_weight,
    );
}

fn outside_label(
    series: &BasicSeries,
    sector: &PieSector<'_>,
    label: LabelStyle,
    text: String,
) -> OutsideLabel {
    let (show_guide, length, length2, guide_color, guide_width, guide_type) =
        label_line(series, sector.point, sector.fill);
    let side = if sector.mid.cos() >= 0.0 { 1.0 } else { -1.0 };
    let edge = (
        sector.center.0 + sector.mid.cos() * sector.outer,
        sector.center.1 + sector.mid.sin() * sector.outer,
    );
    let bend = (
        sector.center.0 + sector.mid.cos() * (sector.outer + length),
        sector.center.1 + sector.mid.sin() * (sector.outer + length),
    );
    OutsideLabel {
        data_index: sector.index,
        text,
        label,
        side,
        edge,
        bend,
        end_x: bend.0 + side * length2,
        y: bend.1,
        guide_color,
        guide_width,
        guide_type,
        show_guide,
    }
}

fn label_line(
    series: &BasicSeries,
    point: &DataPoint,
    fallback_color: u32,
) -> (bool, f32, f32, u32, f32, String) {
    let series_line = series
        .options
        .extra
        .get("labelLine")
        .and_then(serde_json::Value::as_object);
    let point_line = point
        .extra
        .get("labelLine")
        .and_then(serde_json::Value::as_object);
    let value = |key: &str| {
        point_line
            .and_then(|line| line.get(key))
            .or_else(|| series_line.and_then(|line| line.get(key)))
    };
    let series_style = series_line
        .and_then(|line| line.get("lineStyle"))
        .and_then(serde_json::Value::as_object);
    let point_style = point_line
        .and_then(|line| line.get("lineStyle"))
        .and_then(serde_json::Value::as_object);
    let style = |key: &str| {
        point_style
            .and_then(|line| line.get(key))
            .or_else(|| series_style.and_then(|line| line.get(key)))
    };
    (
        value("show")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(true),
        value("length")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(15.0) as f32,
        value("length2")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(15.0) as f32,
        style("color")
            .and_then(crate::parser::parse_color)
            .unwrap_or(fallback_color),
        style("width")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(1.0) as f32,
        style("type")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("solid")
            .to_string(),
    )
}

fn avoid_label_overlap(labels: &mut [OutsideLabel], plot: Plot) {
    for side in [-1.0_f32, 1.0] {
        let mut indices = labels
            .iter()
            .enumerate()
            .filter_map(|(index, label)| (label.side == side).then_some(index))
            .collect::<Vec<_>>();
        indices.sort_by(|left, right| labels[*left].y.total_cmp(&labels[*right].y));
        let mut cursor = plot.y + 4.0;
        for index in &indices {
            let height = labels[*index].label.font_size + 2.0;
            labels[*index].y = labels[*index].y.max(cursor + height * 0.5);
            cursor = labels[*index].y + height * 0.5;
        }
        let mut cursor = plot.y + plot.height - 4.0;
        for index in indices.into_iter().rev() {
            let height = labels[index].label.font_size + 2.0;
            labels[index].y = labels[index].y.min(cursor - height * 0.5);
            cursor = labels[index].y - height * 0.5;
        }
    }
}

fn draw_outside_label(canvas: &Canvas, value: &OutsideLabel) {
    let default_line_points = vec![
        [value.edge.0, value.edge.1],
        [value.bend.0, value.bend.1],
        [value.end_x, value.y],
    ];
    set_next_data_index(value.data_index);
    set_next_label_line_points(default_line_points.clone());
    let text_width = value.text.chars().count() as f32 * value.label.font_size * 0.55;
    let x = if value.side > 0.0 {
        value.end_x + value.label.distance
    } else {
        value.end_x - value.label.distance - text_width
    } + value.label.offset[0];
    let y = value.y + value.label.font_size * 0.35 + value.label.offset[1];
    draw_rotated_text(
        canvas,
        &value.text,
        x,
        y,
        x,
        y,
        value.label.rotate,
        value.label.font_size as f64,
        value.label.color.unwrap_or(0xFF333333),
        value.label.font_weight,
    );
    if value.show_guide {
        let points = take_last_label_line_points().unwrap_or(default_line_points);
        if let Some(first) = points.first() {
            let mut path = Path::new();
            path.move_to(first[0], first[1]);
            for point in points.iter().skip(1) {
                path.line_to(point[0], point[1]);
            }
            stroke_path_style(
                canvas,
                &path,
                value.guide_color,
                value.guide_width,
                &value.guide_type,
            );
        }
    }
}

fn format_pie_label(
    label: &LabelStyle,
    series: &BasicSeries,
    point: &DataPoint,
    index: usize,
    percentage: f64,
    precision: usize,
) -> String {
    format_label(label, series, point, index).replace(
        "{d}",
        &format!("{percentage:.precision$}", precision = precision),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minimum_angle_is_reserved_and_total_span_is_preserved() {
        let angles = allocate_angles(&[99.0, 1.0], TAU, 10_f32.to_radians());
        assert!((angles[1] - 10_f32.to_radians()).abs() < 1e-5);
        assert!((angles.iter().sum::<f32>() - TAU).abs() < 1e-5);
    }

    #[test]
    fn explicit_end_angle_limits_clockwise_span() {
        assert!((pie_span(90.0, Some(&serde_json::json!(0)), true) - TAU / 4.0).abs() < 1e-5);
        assert!((pie_span(90.0, Some(&serde_json::json!(-270)), true) - TAU).abs() < 1e-5);
    }

    #[test]
    fn percentage_formatter_uses_configured_precision() {
        let series = BasicSeries::data("pie", [DataPoint::named("A", 1.0)]);
        let point = &series.data[0];
        let label = LabelStyle {
            formatter: Some(String::from("{b} {d}%")),
            ..LabelStyle::default()
        };
        assert_eq!(
            format_pie_label(&label, &series, point, 0, 12.345, 1),
            "A 12.3%"
        );
    }
}
