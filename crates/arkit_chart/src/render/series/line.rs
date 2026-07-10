use super::super::prelude::*;

pub(super) fn render(series: &BasicSeries, context: &mut CartesianRenderContext<'_>) {
    let series_index = context.series_index;
    let plot = context.plot;
    let layout = context.layout;
    let palette = context.palette;
    let canvas = context.canvas;
    let hits = &mut *context.hits;

    let points: Vec<(usize, &DataPoint, f32, f32)> = series
        .data
        .iter()
        .enumerate()
        .filter_map(|(index, point)| {
            let paired = point.values.len() > 1;
            let x_value = (!layout.x.is_category() && paired).then(|| point.number(0));
            if !layout.x.contains(x_value, index) {
                return None;
            }
            let x = layout.x.position(plot, x_value, index, false);
            let raw_y = if paired {
                point.number(1)
            } else {
                point.number(0)
            };
            let y_value = context
                .stack
                .and_then(|stack| stack.get(index))
                .map(|(_, end)| *end)
                .unwrap_or(raw_y);
            if !layout.y.contains(Some(y_value), index) {
                return None;
            }
            let y = layout.y.position(plot, Some(y_value), index, true);
            Some((index, point, x, y))
        })
        .collect();
    let curve_points: Vec<(f32, f32)> = points.iter().map(|(_, _, x, y)| (*x, *y)).collect();
    let curve = smooth_polyline(&curve_points, series.options.smooth);
    let baseline_points: Vec<(f32, f32)> = points
        .iter()
        .map(|(index, _, x, _)| {
            let baseline = context
                .stack
                .and_then(|stack| stack.get(*index))
                .map(|(base, _)| layout.y.position(plot, Some(*base), *index, true))
                .unwrap_or_else(|| layout.y.zero_position(plot, true));
            (*x, baseline)
        })
        .collect();
    let baseline_curve = smooth_polyline(&baseline_points, series.options.smooth);

    if let Some(canvas) = canvas {
        if let (Some(fill), Some(first)) =
            (area_color(series, palette, series_index), curve.first())
        {
            let mut area = Path::new();
            let first_baseline = baseline_curve
                .first()
                .map(|point| point.1)
                .unwrap_or_else(|| layout.y.zero_position(plot, true));
            area.move_to(first.0, first_baseline);
            for (x, y) in &curve {
                area.line_to(*x, *y);
            }
            for (x, y) in baseline_curve.iter().rev() {
                area.line_to(*x, *y);
            }
            area.close();
            fill_path(canvas, &area, fill);
        }

        let mut path = Path::new();
        for (index, (x, y)) in curve.iter().enumerate() {
            if index == 0 {
                path.move_to(*x, *y);
            } else {
                path.line_to(*x, *y);
            }
        }
        stroke_path(
            canvas,
            &path,
            line_color(series, palette, series_index),
            series.options.line_style.width,
        );
    }

    for (index, point, x, y) in points {
        if let Some(canvas) = canvas {
            if series.options.show_symbol {
                let radius = series.options.symbol_size / 2.0;
                fill_circle(
                    canvas,
                    x,
                    y,
                    radius,
                    item_color(series, Some(point), palette, series_index),
                );
                if let Some((border_color, border_width)) = border(series, Some(point)) {
                    stroke_circle(canvas, x, y, radius, border_color, border_width);
                }
            }
            let label = effective_label(series, point);
            if label.show {
                draw_text(
                    canvas,
                    &format_label(label, series, point, index),
                    x + 4.0,
                    y - 6.0,
                    label.font_size as f64,
                    label.color.unwrap_or(0xFF333333),
                    label.font_weight,
                );
            }
        }
        hits.push(point_hit(
            "line",
            series_index,
            index,
            series.name.clone(),
            point,
            (x, y),
            (series.options.symbol_size / 2.0).max(8.0),
        ));
    }
}
