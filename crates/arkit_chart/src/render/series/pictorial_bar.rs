use super::super::prelude::*;

pub(super) fn render(series: &BasicSeries, context: &mut CartesianRenderContext<'_>) {
    let series_index = context.series_index;
    let plot = context.plot;
    let layout = context.layout;
    let palette = context.palette;
    let canvas = context.canvas;
    let hits = &mut *context.hits;
    let count = series.data.len().max(1);
    let slot = layout.x.band_width(plot, false, count);
    let symbol_size = series.options.symbol_size.max(4.0).min(slot * 0.8);
    let repeat = series
        .options
        .extra
        .get("symbolRepeat")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);

    for (index, point) in series.data.iter().enumerate() {
        let value = point.number(0);
        if !layout.x.contains(None, index)
            || (!layout.y.contains(Some(value), index) && !layout.y.contains(Some(0.0), index))
        {
            continue;
        }
        let x = layout.x.position(plot, None, index, false);
        let baseline = layout.y.zero_position(plot, true);
        let end = layout.y.position(plot, Some(value), index, true);
        let top = baseline.min(end);
        let height = (baseline - end).abs().max(1.0);
        let fill = item_color(series, Some(point), palette, series_index);
        if let Some(canvas) = canvas {
            if repeat {
                let symbol_count = (height / symbol_size.max(1.0)).ceil().max(1.0) as usize;
                for symbol in 0..symbol_count {
                    let ratio = (symbol as f32 + 0.5) / symbol_count as f32;
                    fill_circle(
                        canvas,
                        x,
                        baseline + (end - baseline) * ratio,
                        symbol_size * 0.38,
                        fill,
                    );
                }
            } else {
                fill_rect(
                    canvas,
                    x - symbol_size / 2.0,
                    top,
                    symbol_size,
                    height,
                    fill,
                );
                fill_circle(canvas, x, top, symbol_size / 2.0, fill);
            }
        }
        hits.push(rect_hit(
            "pictorialBar",
            series_index,
            index,
            series.name.clone(),
            point,
            (x - symbol_size / 2.0, top, symbol_size, height),
        ));
    }
}
