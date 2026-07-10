use super::super::prelude::*;

pub(super) fn render(series: &BasicSeries, context: &mut CartesianRenderContext<'_>) {
    let series_index = context.series_index;
    let plot = context.plot;
    let layout = context.layout;
    let palette = context.palette;
    let canvas = context.canvas;
    let hits = &mut *context.hits;

    let max_x = layout.x.count().max(
        series
            .data
            .iter()
            .map(|point| point.number(0).max(0.0) as usize + 1)
            .max()
            .unwrap_or(1),
    );
    let max_y = layout.y.count().max(
        series
            .data
            .iter()
            .map(|point| point.number(1).max(0.0) as usize + 1)
            .max()
            .unwrap_or(1),
    );
    let cell_w = layout.x.band_width(plot, false, max_x);
    let cell_h = layout.y.band_width(plot, true, max_y);
    let data_min = series
        .data
        .iter()
        .map(|point| point.number(2))
        .reduce(f64::min)
        .unwrap_or(0.0);
    let data_max = series
        .data
        .iter()
        .map(|point| point.number(2))
        .reduce(f64::max)
        .unwrap_or(1.0);
    let (min_v, max_v, colors) = context
        .visual_map
        .map(|visual_map| (visual_map.min, visual_map.max, visual_map.colors.as_slice()))
        .unwrap_or((data_min, data_max, palette));
    for (index, point) in series.data.iter().enumerate() {
        let x_index = point.number(0).max(0.0) as usize;
        let y_index = point.number(1).max(0.0) as usize;
        let x_value = (!layout.x.is_category()).then(|| point.number(0));
        let y_value = (!layout.y.is_category()).then(|| point.number(1));
        if !layout.x.contains(x_value, x_index) || !layout.y.contains(y_value, y_index) {
            continue;
        }
        let x = layout.x.band_start(plot, x_value, x_index, false, max_x);
        let y = layout.y.band_start(plot, y_value, y_index, true, max_y);
        let normalized = (point.number(2) - min_v) / (max_v - min_v).max(1e-12);
        let fill = point
            .item_style
            .color
            .map(|color| with_opacity(color, point.item_style.opacity))
            .unwrap_or_else(|| gradient_color(colors, normalized));
        if let Some(canvas) = canvas {
            fill_rect(
                canvas,
                x,
                y,
                (cell_w - 1.0).max(1.0),
                (cell_h - 1.0).max(1.0),
                fill,
            );
            if let Some((border_color, border_width)) = border(series, Some(point)) {
                stroke_rect(canvas, x, y, cell_w, cell_h, border_color, border_width);
            }
            let label = effective_label(series, point);
            if label.show {
                draw_text(
                    canvas,
                    &format_label(label, series, point, index),
                    x + 4.0,
                    y + cell_h / 2.0 + label.font_size / 2.0,
                    label.font_size as f64,
                    label.color.unwrap_or(0xFF333333),
                    label.font_weight,
                );
            }
        }
        hits.push(rect_hit(
            "heatmap",
            series_index,
            index,
            series.name.clone(),
            point,
            (x, y, cell_w, cell_h),
        ));
    }
}
