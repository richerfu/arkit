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
    let radius = options.get("radius");
    let (inner, outer) = radius
        .and_then(|value| value.as_array())
        .and_then(|values| Some((values.first()?, values.get(1)?)))
        .map(|(inner, outer)| {
            (
                compat::length(Some(inner), radius_base, 0.0),
                compat::length(Some(outer), radius_base, radius_base * 0.75),
            )
        })
        .unwrap_or_else(|| (0.0, compat::length(radius, radius_base, radius_base * 0.75)));
    let outer = outer.max(inner + 1.0);
    let total: f64 = series
        .data
        .iter()
        .map(|point| point.number(0).max(0.0))
        .sum::<f64>()
        .max(1.0);
    let max_value = series
        .data
        .iter()
        .map(|point| point.number(0).max(0.0))
        .reduce(f64::max)
        .unwrap_or(1.0)
        .max(1.0);
    let clockwise = compat::boolean(options, "clockwise", true);
    let direction = if clockwise { 1.0 } else { -1.0 };
    let mut start = -(compat::number(options, "startAngle", 90.0) as f32).to_radians();
    let pad = (compat::number(options, "padAngle", 0.0) as f32)
        .to_radians()
        .max(0.0);
    let rose_type = compat::string(options, "roseType");

    for (index, point) in series.data.iter().enumerate() {
        let raw_sweep = (point.number(0).max(0.0) / total) as f32 * TAU * direction;
        let sweep = (raw_sweep.abs() - pad).max(0.0) * direction;
        let sector_start = start + pad * 0.5 * direction;
        let item_outer = match rose_type {
            Some("radius") => {
                inner + (outer - inner) * (point.number(0).max(0.0) / max_value) as f32
            }
            Some("area") => {
                inner + (outer - inner) * (point.number(0).max(0.0) / max_value).sqrt() as f32
            }
            _ => outer,
        };
        if let Some(canvas) = canvas {
            fill_ring_sector(
                canvas,
                (cx, cy),
                (inner, item_outer),
                sector_start,
                sweep,
                item_color(series, Some(point), palette, index),
            );
            let label = effective_label(series, point);
            if label.show {
                let mid = sector_start + sweep / 2.0;
                let inside = matches!(label.position.as_str(), "inside" | "inner" | "center");
                let label_radius = if inside {
                    (inner + item_outer) / 2.0
                } else {
                    item_outer + 14.0
                };
                let label_x = cx + mid.cos() * label_radius;
                let label_y = cy + mid.sin() * label_radius;
                if !inside {
                    stroke_line(
                        canvas,
                        cx + mid.cos() * item_outer,
                        cy + mid.sin() * item_outer,
                        label_x,
                        label_y,
                        label.color.unwrap_or(0xFF6B7280),
                        1.0,
                    );
                }
                draw_text(
                    canvas,
                    &format_label(label, series, point, index),
                    label_x + if mid.cos() >= 0.0 { 3.0 } else { -34.0 },
                    label_y + label.font_size / 2.0,
                    label.font_size as f64,
                    label.color.unwrap_or(0xFF333333),
                    label.font_weight,
                );
            }
        }
        hits.push(HitRegion {
            shape: HitShape::Sector {
                cx,
                cy,
                inner,
                outer: item_outer,
                start: normalize_angle(sector_start),
                sweep,
            },
            event: chart_event(
                "pie",
                series_index,
                index,
                series.name.clone(),
                point,
                cx,
                cy,
            ),
        });
        start += raw_sweep;
    }
}
