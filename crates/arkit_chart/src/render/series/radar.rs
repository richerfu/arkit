use super::super::compat;
use super::super::prelude::*;

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
    let maxima: Vec<(f64, f64)> = (0..count)
        .map(|dimension| {
            radar
                .and_then(|radar| radar.indicators.get(dimension))
                .map(|indicator| (indicator.min, indicator.max))
                .unwrap_or_else(|| {
                    let max = series
                        .data
                        .iter()
                        .map(|point| point.number(dimension))
                        .reduce(f64::max)
                        .unwrap_or(1.0)
                        .max(1.0);
                    (0.0, max)
                })
        })
        .collect();

    if let Some(canvas) = canvas {
        for split in 1..=split_number {
            let split_radius = radius * split as f32 / split_number as f32;
            if circular {
                stroke_circle(canvas, cx, cy, split_radius, 0xFFE5E7EB, 1.0);
            } else {
                let mut path = Path::new();
                for dimension in 0..count {
                    let angle = start + TAU * dimension as f32 / count as f32;
                    let x = cx + angle.cos() * split_radius;
                    let y = cy + angle.sin() * split_radius;
                    if dimension == 0 {
                        path.move_to(x, y);
                    } else {
                        path.line_to(x, y);
                    }
                }
                path.close();
                stroke_path(canvas, &path, 0xFFE5E7EB, 1.0);
            }
        }
        for dimension in 0..count {
            let angle = start + TAU * dimension as f32 / count as f32;
            let edge_x = cx + angle.cos() * radius;
            let edge_y = cy + angle.sin() * radius;
            stroke_line(canvas, cx, cy, edge_x, edge_y, 0xFFD1D5DB, 1.0);
            if let Some(indicator) = radar.and_then(|radar| radar.indicators.get(dimension)) {
                draw_text(
                    canvas,
                    &indicator.name,
                    cx + angle.cos() * (radius + 13.0) - 12.0,
                    cy + angle.sin() * (radius + 13.0) + 5.0,
                    11.0,
                    indicator.color.unwrap_or(0xFF333333),
                    400,
                );
            }
        }
    }

    for (data_index, point) in series.data.iter().enumerate() {
        let mut vertices = Vec::with_capacity(count);
        let mut path = Path::new();
        for (dimension, (min, max)) in maxima.iter().copied().enumerate().take(count) {
            let normalized =
                ((point.number(dimension) - min) / (max - min).max(1e-12)).clamp(0.0, 1.0) as f32;
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
                for (x, y) in &vertices {
                    fill_circle(canvas, *x, *y, series.options.symbol_size / 2.0, data_color);
                }
            }
        }
        for (x, y) in vertices {
            hits.push(point_hit(
                "radar",
                series_index,
                data_index,
                series.name.clone(),
                point,
                (x, y),
                (series.options.symbol_size / 2.0).max(8.0),
            ));
        }
    }
}
