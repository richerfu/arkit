use super::super::prelude::*;

pub(super) fn render(series: &BasicSeries, context: &mut CartesianRenderContext<'_>) {
    let series_index = context.series_index;
    let plot = context.plot;
    let layout = context.layout;
    let canvas = context.canvas;
    let hits = &mut *context.hits;

    let count = series.data.len().max(1);
    let width = layout.x.band_width(plot, false, count) * 0.5;
    for (index, point) in series.data.iter().enumerate() {
        if !layout.x.contains(None, index) {
            continue;
        }
        let x = layout.x.position(plot, None, index, false);
        let open = point.number(0);
        let close = point.number(1);
        let low = point.number(2);
        let high = point.number(3);
        if !layout.y.contains(Some(low), index) && !layout.y.contains(Some(high), index) {
            continue;
        }
        let rising = close >= open;
        let color_value = if rising {
            series.options.item_style.color.unwrap_or(0xFFEC0000)
        } else {
            series.options.item_style.color0.unwrap_or(0xFF00DA3C)
        };
        let border_color = if rising {
            series
                .options
                .item_style
                .border_color
                .unwrap_or(color_value)
        } else {
            series
                .options
                .item_style
                .border_color0
                .unwrap_or(color_value)
        };
        let high_y = layout.y.position(plot, Some(high), index, true);
        let low_y = layout.y.position(plot, Some(low), index, true);
        let open_y = layout.y.position(plot, Some(open), index, true);
        let close_y = layout.y.position(plot, Some(close), index, true);
        if let Some(canvas) = canvas {
            stroke_line(canvas, x, high_y, x, low_y, color_value, 1.2);
            fill_rect(
                canvas,
                x - width / 2.0,
                open_y.min(close_y),
                width,
                (open_y - close_y).abs().max(1.0),
                color_value,
            );
            stroke_rect(
                canvas,
                x - width / 2.0,
                open_y.min(close_y),
                width,
                (open_y - close_y).abs().max(1.0),
                border_color,
                series.options.item_style.border_width.max(1.0),
            );
            let label = effective_label(series, point);
            if label.show {
                draw_text(
                    canvas,
                    &format_label(label, series, point, index),
                    x + width / 2.0 + 2.0,
                    high_y,
                    label.font_size as f64,
                    label.color.unwrap_or(0xFF333333),
                    label.font_weight,
                );
            }
        }
        hits.push(rect_hit(
            "candlestick",
            series_index,
            index,
            series.name.clone(),
            point,
            (x - width / 2.0, high_y, width, low_y - high_y),
        ));
    }
}
