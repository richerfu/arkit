use super::super::compat;
use super::super::layout::tree_layout;
use super::super::prelude::*;

pub(super) fn render(series: &GraphSeries, context: &mut FreeRenderContext<'_>) {
    let series_index = context.series_index;
    let full_plot = context.plot;
    let plot = compat::inset_plot(
        &series.options.extra,
        full_plot,
        [
            full_plot.width * 0.07,
            full_plot.height * 0.07,
            full_plot.width * 0.20,
            full_plot.height * 0.07,
        ],
    );
    let palette = context.palette;
    let canvas = context.canvas;
    let hits = &mut *context.hits;
    let orientation = compat::string(&series.options.extra, "orient").unwrap_or("LR");
    let positions = tree_layout(series.nodes.len(), &series.links, plot, orientation);
    let edge_color = series.options.line_style.color.unwrap_or(0xFFB8C5D6);

    if let Some(canvas) = canvas {
        for link in &series.links {
            let (Some(source), Some(target)) =
                (positions.get(link.source), positions.get(link.target))
            else {
                continue;
            };
            if matches!(orientation, "TB" | "BT") {
                let middle = (source.1 + target.1) / 2.0;
                stroke_line(
                    canvas, source.0, source.1, source.0, middle, edge_color, 1.2,
                );
                stroke_line(canvas, source.0, middle, target.0, middle, edge_color, 1.2);
                stroke_line(
                    canvas, target.0, middle, target.0, target.1, edge_color, 1.2,
                );
            } else {
                let middle = (source.0 + target.0) / 2.0;
                stroke_line(
                    canvas, source.0, source.1, middle, source.1, edge_color, 1.2,
                );
                stroke_line(canvas, middle, source.1, middle, target.1, edge_color, 1.2);
                stroke_line(
                    canvas, middle, target.1, target.0, target.1, edge_color, 1.2,
                );
            }
        }
    }

    for (index, node) in series.nodes.iter().enumerate() {
        let Some((x, y)) = positions.get(index).copied() else {
            continue;
        };
        let radius = node
            .symbol_size
            .unwrap_or(series.options.symbol_size)
            .max(2.0)
            / 2.0;
        let fill = with_opacity(
            series
                .options
                .item_style
                .color
                .unwrap_or_else(|| color(palette, node.category.unwrap_or(series_index))),
            series.options.item_style.opacity,
        );
        if let Some(canvas) = canvas {
            fill_circle(canvas, x, y, radius, fill);
            if let (Some(border_color), border_width) = (
                series.options.item_style.border_color,
                series.options.item_style.border_width,
            ) {
                if border_width > 0.0 {
                    stroke_circle(canvas, x, y, radius, border_color, border_width);
                }
            }
            if series.options.label.show {
                let (label_x, label_y) = match orientation {
                    "RL" => (x - radius - node.name.chars().count() as f32 * 6.0, y + 4.0),
                    "TB" => (
                        x - node.name.chars().count() as f32 * 3.0,
                        y + radius + 14.0,
                    ),
                    "BT" => (x - node.name.chars().count() as f32 * 3.0, y - radius - 4.0),
                    _ => (x + radius + 4.0, y + 4.0),
                };
                draw_text(
                    canvas,
                    &node.name,
                    label_x,
                    label_y,
                    series.options.label.font_size as f64,
                    series.options.label.color.unwrap_or(0xFF333333),
                    series.options.label.font_weight,
                );
            }
        }
        hits.push(HitRegion {
            shape: HitShape::Point {
                x,
                y,
                radius: radius.max(12.0),
            },
            event: ChartEvent {
                series_index,
                data_index: index,
                series_name: series.name.clone(),
                name: Some(node.name.clone()),
                value: vec![node.value],
                x,
                y,
                component_type: String::from("tree"),
            },
        });
    }
}
