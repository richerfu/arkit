use super::super::prelude::*;

pub(super) fn render(series: &BasicSeries, context: &mut FreeRenderContext<'_>) {
    let series_index = context.series_index;
    let plot = context.plot;
    let palette = context.palette;
    let canvas = context.canvas;
    let hits = &mut *context.hits;
    let dimension_count = series
        .data
        .iter()
        .map(|point| point.values.len())
        .max()
        .unwrap_or(0);
    if dimension_count < 2 {
        return;
    }
    let axis_options = context
        .option
        .extra
        .get("parallelAxis")
        .and_then(serde_json::Value::as_array);
    let extents: Vec<(f64, f64)> = (0..dimension_count)
        .map(|dimension| {
            let values: Vec<f64> = series
                .data
                .iter()
                .filter_map(|point| point.number_opt(dimension))
                .collect();
            let data_min = values.iter().copied().reduce(f64::min).unwrap_or(0.0);
            let data_max = values.iter().copied().reduce(f64::max).unwrap_or(1.0);
            let options = axis_options.and_then(|options| options.get(dimension));
            (
                options
                    .and_then(serde_json::Value::as_object)
                    .and_then(|options| options.get("min"))
                    .and_then(serde_json::Value::as_f64)
                    .unwrap_or(data_min),
                options
                    .and_then(serde_json::Value::as_object)
                    .and_then(|options| options.get("max"))
                    .and_then(serde_json::Value::as_f64)
                    .unwrap_or(data_max.max(data_min + 1.0)),
            )
        })
        .collect();

    if let Some(canvas) = canvas {
        for dimension in 0..dimension_count {
            let x = plot.x + plot.width * dimension as f32 / (dimension_count - 1) as f32;
            stroke_line(
                canvas,
                x,
                plot.y + 20.0,
                x,
                plot.y + plot.height - 20.0,
                0xFF9CA3AF,
                1.0,
            );
            let name = axis_options
                .and_then(|options| options.get(dimension))
                .and_then(serde_json::Value::as_object)
                .and_then(|options| options.get("name"))
                .and_then(serde_json::Value::as_str)
                .map(ToOwned::to_owned)
                .unwrap_or_else(|| format!("dim {dimension}"));
            draw_text(
                canvas,
                &name,
                x - name.chars().count() as f32 * 3.0,
                plot.y + 14.0,
                10.0,
                0xFF4B5563,
                400,
            );
        }
    }

    for (data_index, point) in series.data.iter().enumerate() {
        let Some(values) = (0..dimension_count)
            .map(|dimension| point.number_opt(dimension))
            .collect::<Option<Vec<_>>>()
        else {
            continue;
        };
        let mut path = Path::new();
        for (dimension, (min, max)) in extents.iter().copied().enumerate() {
            let x = plot.x + plot.width * dimension as f32 / (dimension_count - 1) as f32;
            let normalized =
                ((values[dimension] - min) / (max - min).max(1e-12)).clamp(0.0, 1.0) as f32;
            let y = plot.y + 20.0 + (plot.height - 40.0) * (1.0 - normalized);
            if dimension == 0 {
                path.move_to(x, y);
            } else {
                path.line_to(x, y);
            }
            hits.push(point_hit(
                "parallel",
                series_index,
                data_index,
                series.name.clone(),
                point,
                (x, y),
                7.0,
            ));
        }
        if let Some(canvas) = canvas {
            stroke_path(
                canvas,
                &path,
                item_color(series, Some(point), palette, data_index),
                series.options.line_style.width.max(1.0),
            );
        }
    }
}
