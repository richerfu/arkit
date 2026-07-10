use super::super::prelude::*;

pub(super) fn render(series: &BasicSeries, context: &mut CartesianRenderContext<'_>) {
    render_impl(series, context, false);
}

pub(super) fn render_effect(series: &BasicSeries, context: &mut CartesianRenderContext<'_>) {
    render_impl(series, context, true);
}

fn render_impl(series: &BasicSeries, context: &mut CartesianRenderContext<'_>, effect: bool) {
    let series_index = context.series_index;
    let plot = context.plot;
    let layout = context.layout;
    let palette = context.palette;
    let canvas = context.canvas;
    let hits = &mut *context.hits;

    for (index, point) in series.data.iter().enumerate() {
        let paired = point.values.len() > 1;
        let x_value = paired.then(|| point.number(0));
        let y_value = if paired {
            point.number(1)
        } else {
            point.number(0)
        };
        if !layout.x.contains(x_value, index) || !layout.y.contains(Some(y_value), index) {
            continue;
        }
        let x = layout.x.position(plot, x_value, index, false);
        let y = layout.y.position(plot, Some(y_value), index, true);
        if let Some(canvas) = canvas {
            let radius = series.options.symbol_size.max(1.0) / 2.0;
            fill_circle(
                canvas,
                x,
                y,
                radius,
                item_color(series, Some(point), palette, series_index),
            );
            if effect {
                stroke_circle(
                    canvas,
                    x,
                    y,
                    radius + 4.0,
                    with_opacity(item_color(series, Some(point), palette, series_index), 0.45),
                    2.0,
                );
            }
            if let Some((border_color, border_width)) = border(series, Some(point)) {
                stroke_circle(canvas, x, y, radius, border_color, border_width);
            }
            let label = effective_label(series, point);
            if label.show {
                draw_text(
                    canvas,
                    &format_label(label, series, point, index),
                    x + radius + 2.0,
                    y,
                    label.font_size as f64,
                    label.color.unwrap_or(0xFF333333),
                    label.font_weight,
                );
            }
        }
        hits.push(point_hit(
            if effect { "effectScatter" } else { "scatter" },
            series_index,
            index,
            series.name.clone(),
            point,
            (x, y),
            (series.options.symbol_size / 2.0).max(9.0),
        ));
    }
}
