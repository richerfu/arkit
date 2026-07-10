use super::super::prelude::*;

pub(super) fn render(series: &MapSeries, context: &mut FreeRenderContext<'_>) {
    let series_index = context.series_index;
    let plot = context.plot;
    let palette = context.palette;
    let canvas = context.canvas;
    let hits = &mut *context.hits;

    let bounds = map_bounds(&series.features).unwrap_or((0.0, 0.0, 1.0, 1.0));
    let geo_width = (bounds.2 - bounds.0).max(1e-9) as f32;
    let geo_height = (bounds.3 - bounds.1).max(1e-9) as f32;
    let scale = (plot.width / geo_width).min(plot.height / geo_height);
    let offset_x = plot.x + (plot.width - geo_width * scale) / 2.0;
    let offset_y = plot.y + (plot.height - geo_height * scale) / 2.0;
    let data_min = series
        .features
        .iter()
        .map(|feature| feature.value)
        .reduce(f64::min)
        .unwrap_or(0.0);
    let data_max = series
        .features
        .iter()
        .map(|feature| feature.value)
        .reduce(f64::max)
        .unwrap_or(1.0);
    let (visual_min, visual_max, colors) = context
        .option
        .visual_map
        .as_ref()
        .map(|visual_map| (visual_map.min, visual_map.max, visual_map.colors.as_slice()))
        .unwrap_or((data_min, data_max, palette));

    for (index, feature) in series.features.iter().enumerate() {
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;
        let normalized = (feature.value - visual_min) / (visual_max - visual_min).max(1e-12);
        let fill = series
            .options
            .item_style
            .color
            .map(|color| with_opacity(color, series.options.item_style.opacity))
            .unwrap_or_else(|| gradient_color(colors, normalized));
        for polygon in &feature.polygons {
            let mut path = Path::new();
            for (point_index, point) in polygon.iter().enumerate() {
                let x = offset_x + (point.0 - bounds.0) as f32 * scale;
                let y = offset_y + (bounds.3 - point.1) as f32 * scale;
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x);
                max_y = max_y.max(y);
                if point_index == 0 {
                    path.move_to(x, y);
                } else {
                    path.line_to(x, y);
                }
            }
            path.close();
            if let Some(canvas) = canvas {
                fill_path(canvas, &path, fill);
                stroke_path(
                    canvas,
                    &path,
                    series.options.item_style.border_color.unwrap_or(0xFFFFFFFF),
                    series.options.item_style.border_width.max(1.0),
                );
            }
        }
        if let Some(canvas) = canvas {
            if series.options.label.show {
                draw_text(
                    canvas,
                    &feature.name,
                    (min_x + max_x) / 2.0 - feature.name.chars().count() as f32 * 3.0,
                    (min_y + max_y) / 2.0 + 4.0,
                    series.options.label.font_size as f64,
                    series.options.label.color.unwrap_or(0xFF333333),
                    series.options.label.font_weight,
                );
            }
        }
        if min_x.is_finite() {
            let point = DataPoint::named(feature.name.clone(), feature.value);
            hits.push(rect_hit(
                "map",
                series_index,
                index,
                series.name.clone(),
                &point,
                (min_x, min_y, max_x - min_x, max_y - min_y),
            ));
        }
    }
}

pub(crate) fn map_bounds(features: &[MapFeature]) -> Option<(f64, f64, f64, f64)> {
    let mut min_x = f64::MAX;
    let mut min_y = f64::MAX;
    let mut max_x = f64::MIN;
    let mut max_y = f64::MIN;
    let mut any = false;
    for feature in features {
        for polygon in &feature.polygons {
            for (x, y) in polygon {
                any = true;
                min_x = min_x.min(*x);
                min_y = min_y.min(*y);
                max_x = max_x.max(*x);
                max_y = max_y.max(*y);
            }
        }
    }
    any.then_some((min_x, min_y, max_x, max_y))
}
