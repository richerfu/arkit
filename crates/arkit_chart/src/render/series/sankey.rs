use super::super::compat;
use super::super::layout::sankey_layout;
use super::super::prelude::*;

pub(super) fn render(series: &SankeySeries, context: &mut FreeRenderContext<'_>) {
    let series_index = context.series_index;
    let full_plot = context.plot;
    let plot = compat::inset_plot(
        &series.options.extra,
        full_plot,
        [
            full_plot.width * 0.05,
            full_plot.height * 0.05,
            full_plot.width * 0.20,
            full_plot.height * 0.05,
        ],
    );
    let palette = context.palette;
    let canvas = context.canvas;
    let hits = &mut *context.hits;
    let node_width = compat::number(&series.options.extra, "nodeWidth", 20.0).max(1.0) as f32;
    let node_gap = compat::number(&series.options.extra, "nodeGap", 8.0).max(0.0) as f32;
    let orient = compat::string(&series.options.extra, "orient").unwrap_or("horizontal");
    let logical_plot = if orient == "vertical" {
        super::super::geometry::Plot {
            x: 0.0,
            y: 0.0,
            width: plot.height,
            height: plot.width,
        }
    } else {
        plot
    };
    let layout = sankey_layout(
        &series.nodes,
        &series.links,
        logical_plot,
        node_width,
        node_gap,
    );
    let transform_point = |point: (f32, f32)| {
        if orient == "vertical" {
            (plot.x + point.1, plot.y + point.0)
        } else {
            point
        }
    };

    if let Some(canvas) = canvas {
        for (index, link) in layout.links.iter().enumerate() {
            if link.width <= 0.0 {
                continue;
            }
            let source = transform_point(link.source);
            let target = transform_point(link.target);
            let color = series
                .options
                .line_style
                .color
                .unwrap_or_else(|| color(palette, series.links[index].source));
            fill_sankey_ribbon(
                canvas,
                source,
                target,
                link.width,
                with_opacity(color, series.options.line_style.opacity.min(0.5)),
            );
        }
    }

    for (index, (node, area)) in series.nodes.iter().zip(layout.nodes).enumerate() {
        let (x, y, width, height) = if orient == "vertical" {
            (plot.x + area.y, plot.y + area.x, area.height, area.width)
        } else {
            (area.x, area.y, area.width, area.height)
        };
        let style = merge_item_style(&series.options.item_style, &node.item_style);
        let fill = with_opacity(
            style
                .color
                .unwrap_or_else(|| color(palette, node.category.unwrap_or(index))),
            style.opacity,
        );
        if let Some(canvas) = canvas {
            fill_rounded_rect(canvas, x, y, width, height, style.border_radius, fill);
            if let Some(border_color) = style.border_color.filter(|_| style.border_width > 0.0) {
                stroke_rect(
                    canvas,
                    x,
                    y,
                    width,
                    height,
                    border_color,
                    style.border_width,
                );
            }
            let label = merge_label_style(&series.options.label, &node.label);
            if label.show || !series.nodes.is_empty() {
                set_next_data_index(index);
                draw_text(
                    canvas,
                    &node.name,
                    x + width + 4.0,
                    y + height / 2.0 + 4.0,
                    label.font_size as f64,
                    label.color.unwrap_or(0xFF333333),
                    label.font_weight,
                );
            }
        }
        hits.push(HitRegion {
            shape: HitShape::Rect {
                x,
                y,
                width,
                height,
            },
            event: ChartEvent {
                series_index,
                data_index: index,
                series_name: series.name.clone(),
                name: Some(node.name.clone()),
                value: vec![node.value],
                x: x + width / 2.0,
                y: y + height / 2.0,
                component_type: String::from("sankey"),
            },
        });
    }
}

fn fill_sankey_ribbon(
    canvas: &ohos_drawing_binding::Canvas,
    source: (f32, f32),
    target: (f32, f32),
    width: f32,
    color: u32,
) {
    let mut upper = Vec::with_capacity(13);
    let mut lower = Vec::with_capacity(13);
    for step in 0..=12 {
        let t = step as f32 / 12.0;
        let eased = t * t * (3.0 - 2.0 * t);
        let x = source.0 + (target.0 - source.0) * t;
        let y = source.1 + (target.1 - source.1) * eased;
        let dx = target.0 - source.0;
        let dy = (target.1 - source.1) * 6.0 * t * (1.0 - t);
        let length = (dx * dx + dy * dy).sqrt().max(1e-6);
        let normal = (-dy / length * width / 2.0, dx / length * width / 2.0);
        upper.push((x + normal.0, y + normal.1));
        lower.push((x - normal.0, y - normal.1));
    }
    let mut path = Path::new();
    if let Some((x, y)) = upper.first() {
        path.move_to(*x, *y);
    }
    for (x, y) in upper.iter().skip(1) {
        path.line_to(*x, *y);
    }
    for (x, y) in lower.iter().rev() {
        path.line_to(*x, *y);
    }
    path.close();
    fill_path(canvas, &path, color);
}
