use super::super::prelude::*;

pub(super) fn render(series: &LinesSeries, context: &mut FreeRenderContext<'_>) {
    let series_index = context.series_index;
    let plot = context.plot;
    let palette = context.palette;
    let canvas = context.canvas;
    let hits = &mut *context.hits;
    let coordinates: Vec<(f64, f64)> = series
        .data
        .iter()
        .flat_map(|segment| [segment.from, segment.to])
        .collect();
    let min_x = coordinates
        .iter()
        .map(|point| point.0)
        .reduce(f64::min)
        .unwrap_or(0.0);
    let max_x = coordinates
        .iter()
        .map(|point| point.0)
        .reduce(f64::max)
        .unwrap_or(1.0);
    let min_y = coordinates
        .iter()
        .map(|point| point.1)
        .reduce(f64::min)
        .unwrap_or(0.0);
    let max_y = coordinates
        .iter()
        .map(|point| point.1)
        .reduce(f64::max)
        .unwrap_or(1.0);
    let project = |point: (f64, f64)| {
        (
            plot.x + ((point.0 - min_x) / (max_x - min_x).max(1e-12)) as f32 * plot.width,
            plot.y + plot.height
                - ((point.1 - min_y) / (max_y - min_y).max(1e-12)) as f32 * plot.height,
        )
    };
    let line_color = series
        .options
        .line_style
        .color
        .unwrap_or_else(|| color(palette, series_index));

    for (index, segment) in series.data.iter().enumerate() {
        let from = project(segment.from);
        let to = project(segment.to);
        if let Some(canvas) = canvas {
            stroke_line(
                canvas,
                from.0,
                from.1,
                to.0,
                to.1,
                with_opacity(line_color, series.options.line_style.opacity),
                series.options.line_style.width.max(1.0) * segment.value.max(0.5) as f32,
            );
            fill_circle(canvas, to.0, to.1, 3.0, line_color);
        }
        let min_x = from.0.min(to.0) - 6.0;
        let min_y = from.1.min(to.1) - 6.0;
        hits.push(HitRegion {
            shape: HitShape::Rect {
                x: min_x,
                y: min_y,
                width: (from.0 - to.0).abs() + 12.0,
                height: (from.1 - to.1).abs() + 12.0,
            },
            event: ChartEvent {
                series_index,
                data_index: index,
                series_name: series.name.clone(),
                name: segment.name.clone(),
                value: vec![segment.value],
                x: (from.0 + to.0) / 2.0,
                y: (from.1 + to.1) / 2.0,
                component_type: String::from("lines"),
            },
        });
    }
}
