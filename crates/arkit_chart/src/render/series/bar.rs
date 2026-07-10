use super::super::prelude::*;

pub(super) fn render(series: &BasicSeries, context: &mut CartesianRenderContext<'_>) {
    let series_index = context.series_index;
    let plot = context.plot;
    let layout = context.layout;
    let (bar_offset, bar_count) = context
        .bar_layout
        .expect("bar renderer requires bar layout");
    let palette = context.palette;
    let canvas = context.canvas;
    let hits = &mut *context.hits;

    let count = series.data.len().max(1);
    let slot = layout.x.band_width(plot, false, count);
    let auto_width = slot * 0.7 / bar_count as f32;
    let width = series
        .options
        .bar_width
        .unwrap_or(auto_width)
        .min(slot * 0.95);
    for (index, point) in series.data.iter().enumerate() {
        let paired = point.values.len() > 1;
        let value = if paired {
            point.number(1)
        } else {
            point.number(0)
        };
        let x_value = (!layout.x.is_category() && paired).then(|| point.number(0));
        if !layout.x.contains(x_value, index) {
            continue;
        }
        let center = layout.x.position(plot, x_value, index, false);
        let (base_value, end_value) = context
            .stack
            .and_then(|stack| stack.get(index))
            .copied()
            .unwrap_or((0.0, value));
        if !layout.y.contains(Some(end_value), index) && !layout.y.contains(Some(base_value), index)
        {
            continue;
        }
        let base = layout.y.position(plot, Some(base_value), index, true);
        let y = layout.y.position(plot, Some(end_value), index, true);
        let group_width = width * bar_count as f32;
        let x = center - group_width / 2.0 + width * bar_offset as f32;
        let top = y.min(base);
        let height = (base - y).abs().max(1.0);
        if let Some(canvas) = canvas {
            fill_rect(
                canvas,
                x,
                top,
                width,
                height,
                item_color(series, Some(point), palette, series_index),
            );
            if let Some((border_color, border_width)) = border(series, Some(point)) {
                stroke_rect(canvas, x, top, width, height, border_color, border_width);
            }
            let label = effective_label(series, point);
            if label.show {
                draw_text(
                    canvas,
                    &format_label(label, series, point, index),
                    x + 2.0,
                    top - 4.0,
                    label.font_size as f64,
                    label.color.unwrap_or(0xFF333333),
                    label.font_weight,
                );
            }
        }
        hits.push(rect_hit(
            "bar",
            series_index,
            index,
            series.name.clone(),
            point,
            (x, top, width, height),
        ));
    }
}
