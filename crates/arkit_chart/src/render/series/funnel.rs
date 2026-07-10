use super::super::compat;
use super::super::prelude::*;

pub(super) fn render(series: &BasicSeries, context: &mut FreeRenderContext<'_>) {
    let series_index = context.series_index;
    let plot = context.plot;
    let palette = context.palette;
    let canvas = context.canvas;
    let hits = &mut *context.hits;
    let options = &series.options.extra;

    let left = compat::position(
        options.get("left"),
        plot.x,
        plot.width,
        plot.x + plot.width * 0.1,
    );
    let top = compat::position(
        options.get("top"),
        plot.y,
        plot.height,
        plot.y + plot.height * 0.1,
    );
    let width = compat::length(options.get("width"), plot.width, plot.width * 0.8);
    let height = compat::length(options.get("height"), plot.height, plot.height * 0.8);
    let gap = compat::number(options, "gap", 0.0).max(0.0) as f32;
    let orient = compat::string(options, "orient").unwrap_or("vertical");
    let align = compat::string(options, "funnelAlign").unwrap_or("center");
    let mut items: Vec<(usize, &DataPoint)> = series.data.iter().enumerate().collect();
    match compat::string(options, "sort").unwrap_or("descending") {
        "ascending" => items.sort_by(|left, right| left.1.number(0).total_cmp(&right.1.number(0))),
        "none" => {}
        _ => items.sort_by(|left, right| right.1.number(0).total_cmp(&left.1.number(0))),
    }
    let data_min = items
        .iter()
        .map(|(_, point)| point.number(0))
        .reduce(f64::min)
        .unwrap_or(0.0);
    let data_max = items
        .iter()
        .map(|(_, point)| point.number(0))
        .reduce(f64::max)
        .unwrap_or(1.0);
    let min = compat::number(options, "min", data_min);
    let max = compat::number(options, "max", data_max).max(min + f64::EPSILON);
    let cross_extent = if orient == "horizontal" {
        height
    } else {
        width
    };
    let min_size = compat::length(options.get("minSize"), cross_extent, 0.0);
    let max_size = compat::length(options.get("maxSize"), cross_extent, cross_extent);
    let count = items.len().max(1);
    let main_extent = if orient == "horizontal" {
        width
    } else {
        height
    };
    let item_extent =
        ((main_extent - gap * count.saturating_sub(1) as f32) / count as f32).max(1.0);

    let size_for = |point: &DataPoint| {
        let normalized = ((point.number(0) - min) / (max - min)).clamp(0.0, 1.0) as f32;
        min_size + (max_size - min_size) * normalized
    };
    for (order, (data_index, point)) in items.iter().enumerate() {
        let current = size_for(point);
        let next = items
            .get(order + 1)
            .map(|(_, point)| size_for(point))
            .unwrap_or(current);
        let main = order as f32 * (item_extent + gap);
        let (bounds, path) = if orient == "horizontal" {
            let x = left + main;
            let current_y = aligned_offset(top, height, current, align);
            let next_y = aligned_offset(top, height, next, align);
            let mut path = Path::new();
            path.move_to(x, current_y);
            path.line_to(x, current_y + current);
            path.line_to(x + item_extent, next_y + next);
            path.line_to(x + item_extent, next_y);
            path.close();
            (
                (x, current_y.min(next_y), item_extent, current.max(next)),
                path,
            )
        } else {
            let y = top + main;
            let current_x = aligned_offset(left, width, current, align);
            let next_x = aligned_offset(left, width, next, align);
            let mut path = Path::new();
            path.move_to(current_x, y);
            path.line_to(current_x + current, y);
            path.line_to(next_x + next, y + item_extent);
            path.line_to(next_x, y + item_extent);
            path.close();
            (
                (current_x.min(next_x), y, current.max(next), item_extent),
                path,
            )
        };
        if let Some(canvas) = canvas {
            fill_path(
                canvas,
                &path,
                item_color(series, Some(point), palette, *data_index),
            );
            let label = effective_label(series, point);
            if label.show {
                let inside = label.position.contains("inside");
                let label_x = if inside {
                    bounds.0 + bounds.2 / 2.0 - 18.0
                } else {
                    bounds.0 + bounds.2 + 7.0
                };
                let label_y = bounds.1 + bounds.3 / 2.0 + label.font_size / 2.0;
                draw_text(
                    canvas,
                    &format_label(label, series, point, *data_index),
                    label_x,
                    label_y,
                    label.font_size as f64,
                    label
                        .color
                        .unwrap_or(if inside { 0xFFFFFFFF } else { 0xFF333333 }),
                    label.font_weight,
                );
            }
        }
        hits.push(rect_hit(
            "funnel",
            series_index,
            *data_index,
            series.name.clone(),
            point,
            bounds,
        ));
    }
}

fn aligned_offset(start: f32, extent: f32, size: f32, align: &str) -> f32 {
    match align {
        "left" | "top" => start,
        "right" | "bottom" => start + extent - size,
        _ => start + (extent - size) / 2.0,
    }
}
