use super::super::prelude::*;

pub(super) fn render(series: &BasicSeries, context: &mut CartesianRenderContext<'_>) {
    let series_index = context.series_index;
    let plot = context.plot;
    let layout = context.layout;
    let canvas = context.canvas;
    let hits = &mut *context.hits;

    let count = series.data.len().max(1);
    let slot = layout.x.band_width(plot, false, count);
    let width = series
        .options
        .bar_width
        .map(|width| width.resolve(slot))
        .unwrap_or(slot * 0.5)
        .clamp(1.0, slot);
    for (index, point) in series.data.iter().enumerate() {
        let (Some(open), Some(close), Some(low), Some(high)) = (
            point.number_opt(0),
            point.number_opt(1),
            point.number_opt(2),
            point.number_opt(3),
        ) else {
            continue;
        };
        if !layout.x.contains(None, index) {
            continue;
        }
        let x = layout.x.position(plot, None, index, false);
        if !layout.y.contains(Some(low), index) && !layout.y.contains(Some(high), index) {
            continue;
        }
        let rising = close >= open;
        let style = effective_item_style(series, Some(point));
        let color_value = if rising {
            style.color.unwrap_or(0xFFEC0000)
        } else {
            style.color0.unwrap_or(0xFF00DA3C)
        };
        let border_color = if rising {
            style.border_color.unwrap_or(color_value)
        } else {
            style.border_color0.unwrap_or(color_value)
        };
        let color_value = with_opacity(color_value, style.opacity);
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
                style.border_width.max(1.0),
            );
            let label = effective_label(series, point);
            if label.show {
                set_next_data_index(index);
                draw_text(
                    canvas,
                    &format_label(&label, series, point, index),
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
