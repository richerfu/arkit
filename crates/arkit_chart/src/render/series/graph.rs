use super::super::compat;
use super::super::layout::{circular_layout, force_layout, positioned_graph_layout};
use super::super::prelude::*;

pub(super) fn render(series: &GraphSeries, context: &mut FreeRenderContext<'_>) {
    let series_index = context.series_index;
    let plot = context.plot;
    let palette = context.palette;
    let canvas = context.canvas;
    let hits = &mut *context.hits;
    let layout = compat::string(&series.options.extra, "layout").unwrap_or("none");
    let positions = match layout {
        "force" => {
            let force = series
                .options
                .extra
                .get("force")
                .and_then(serde_json::Value::as_object);
            let repulsion = force
                .and_then(|value| value.get("repulsion"))
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(50.0) as f32;
            let gravity = force
                .and_then(|value| value.get("gravity"))
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(0.1) as f32;
            let edge_length = force
                .and_then(|value| value.get("edgeLength"))
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(30.0) as f32;
            force_layout(
                series.nodes.len(),
                &series.links,
                plot,
                repulsion,
                gravity,
                edge_length,
            )
        }
        "circular" => circular_layout(series.nodes.len(), plot),
        _ => positioned_graph_layout(&series.nodes, plot)
            .unwrap_or_else(|| circular_layout(series.nodes.len(), plot)),
    };
    let edge_color = series.options.line_style.color.unwrap_or(0xFFB8C5D6);

    if let Some(canvas) = canvas {
        for link in &series.links {
            if let (Some(source), Some(target)) =
                (positions.get(link.source), positions.get(link.target))
            {
                stroke_line(
                    canvas,
                    source.0,
                    source.1,
                    target.0,
                    target.1,
                    edge_color,
                    series.options.line_style.width.max(1.0) * link.value.max(0.25) as f32,
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
                .unwrap_or_else(|| color(palette, node.category.unwrap_or(index))),
            series.options.item_style.opacity,
        );
        if let Some(canvas) = canvas {
            fill_circle(canvas, x, y, radius, fill);
            if series.options.label.show {
                draw_text(
                    canvas,
                    &node.name,
                    x + radius + 3.0,
                    y + 4.0,
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
                component_type: String::from("graph"),
            },
        });
    }
}
