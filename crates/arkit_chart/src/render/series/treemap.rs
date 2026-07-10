use super::super::compat;
use super::super::layout::squarify;
use super::super::prelude::*;

pub(super) fn render(series: &BasicSeries, context: &mut FreeRenderContext<'_>) {
    let series_index = context.series_index;
    let plot = context.plot;
    let palette = context.palette;
    let canvas = context.canvas;
    let hits = &mut *context.hits;
    let visible_min = compat::number(&series.options.extra, "visibleMin", 10.0).max(0.0) as f32;
    let gap = series.options.item_style.border_width.max(1.0);
    let weights: Vec<f64> = series
        .data
        .iter()
        .map(|point| point.number(0).max(0.0))
        .collect();
    let areas = squarify(&weights, plot);

    for (index, (point, area)) in series.data.iter().zip(areas).enumerate() {
        if area.width * area.height < visible_min {
            continue;
        }
        let x = area.x + gap / 2.0;
        let y = area.y + gap / 2.0;
        let width = (area.width - gap).max(1.0);
        let height = (area.height - gap).max(1.0);
        if let Some(canvas) = canvas {
            fill_rect(
                canvas,
                x,
                y,
                width,
                height,
                item_color(series, Some(point), palette, index),
            );
            if let Some((border_color, border_width)) = border(series, Some(point)) {
                stroke_rect(canvas, x, y, width, height, border_color, border_width);
            }
            let label = effective_label(series, point);
            if label.show && width >= 28.0 && height >= label.font_size + 8.0 {
                draw_text(
                    canvas,
                    &format_label(label, series, point, index),
                    x + 5.0,
                    y + label.font_size + 5.0,
                    label.font_size as f64,
                    label.color.unwrap_or(0xFFFFFFFF),
                    label.font_weight.max(500),
                );
            }
        }
        hits.push(rect_hit(
            "treemap",
            series_index,
            index,
            series.name.clone(),
            point,
            (x, y, width, height),
        ));
    }
}
