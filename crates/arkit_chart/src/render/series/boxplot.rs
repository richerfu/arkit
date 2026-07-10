use super::super::prelude::*;

pub(super) fn render(series: &BasicSeries, context: &mut CartesianRenderContext<'_>) {
    let series_index = context.series_index;
    let plot = context.plot;
    let layout = context.layout;
    let palette = context.palette;
    let canvas = context.canvas;
    let hits = &mut *context.hits;
    let count = series.data.len().max(1);
    let width = layout.x.band_width(plot, false, count) * 0.5;

    for (index, point) in series.data.iter().enumerate() {
        if point.values.len() < 5 {
            continue;
        }
        if !layout.x.contains(None, index) {
            continue;
        }
        let values = [
            point.number(0),
            point.number(1),
            point.number(2),
            point.number(3),
            point.number(4),
        ];
        if !layout.y.contains(Some(values[0]), index) && !layout.y.contains(Some(values[4]), index)
        {
            continue;
        }
        let x = layout.x.position(plot, None, index, false);
        let ys = values.map(|value| layout.y.position(plot, Some(value), index, true));
        let top = ys[4].min(ys[0]);
        let bottom = ys[4].max(ys[0]);
        let box_top = ys[3].min(ys[1]);
        let box_height = (ys[3] - ys[1]).abs().max(1.0);
        let stroke = series
            .options
            .item_style
            .border_color
            .unwrap_or_else(|| color(palette, series_index));
        let stroke_width = series.options.item_style.border_width.max(1.5);
        if let Some(canvas) = canvas {
            stroke_line(canvas, x, top, x, bottom, stroke, stroke_width);
            stroke_line(
                canvas,
                x - width * 0.3,
                ys[4],
                x + width * 0.3,
                ys[4],
                stroke,
                stroke_width,
            );
            stroke_line(
                canvas,
                x - width * 0.3,
                ys[0],
                x + width * 0.3,
                ys[0],
                stroke,
                stroke_width,
            );
            fill_rect(
                canvas,
                x - width / 2.0,
                box_top,
                width,
                box_height,
                series
                    .options
                    .item_style
                    .color
                    .map(|color| with_opacity(color, series.options.item_style.opacity))
                    .unwrap_or(0x22FFFFFF | (stroke & 0x00FFFFFF)),
            );
            stroke_rect(
                canvas,
                x - width / 2.0,
                box_top,
                width,
                box_height,
                stroke,
                stroke_width,
            );
            stroke_line(
                canvas,
                x - width / 2.0,
                ys[2],
                x + width / 2.0,
                ys[2],
                stroke,
                stroke_width,
            );
        }
        hits.push(rect_hit(
            "boxplot",
            series_index,
            index,
            series.name.clone(),
            point,
            (x - width / 2.0, top, width, bottom - top),
        ));
    }
}
