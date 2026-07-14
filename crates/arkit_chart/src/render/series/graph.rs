use super::super::compat;
use super::super::layout::{circular_layout, force_layout, positioned_graph_layout};
use super::super::prelude::*;
use super::super::symbol::{draw_symbol, SymbolSpec};

pub(super) fn render(series: &GraphSeries, context: &mut FreeRenderContext<'_>) {
    let series_index = context.series_index;
    let plot = context.plot;
    let palette = context.palette;
    let canvas = context.canvas;
    let hits = &mut *context.hits;
    let layout = compat::string(&series.options.extra, "layout").unwrap_or("none");
    let geo_index = super::geo_index(&context.option.series[series_index]);
    let geo_transform = geo_index
        .and_then(|index| super::map::transform_from_geo_component(context.option, plot, index));
    if let (Some(index), Some(_)) = (geo_index, geo_transform) {
        if super::should_draw_geo_base(context.option, series_index, index) {
            super::map::draw_geo_component(context.option, plot, index, canvas);
        }
    }
    let positions = if let Some(transform) = geo_transform {
        let fallback = circular_layout(series.nodes.len(), plot);
        series
            .nodes
            .iter()
            .enumerate()
            .map(|(index, node)| {
                node.x
                    .zip(node.y)
                    .map(|point| transform.project(point))
                    .unwrap_or(fallback[index])
            })
            .collect()
    } else {
        match layout {
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
        }
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
        let size = node
            .symbol_size
            .unwrap_or(series.options.symbol_size)
            .max(2.0);
        let dimensions = node
            .symbol_size_dimensions
            .or(series.options.symbol_size_dimensions)
            .unwrap_or([size, size]);
        let symbol = SymbolSpec {
            name: node.symbol.as_deref().unwrap_or(&series.options.symbol),
            size: dimensions,
            rotate: if node.symbol_rotate.abs() > f32::EPSILON {
                node.symbol_rotate
            } else {
                series.options.symbol_rotate
            },
            offset: [0.0, 0.0],
        };
        let style = merge_item_style(&series.options.item_style, &node.item_style);
        let fill = with_opacity(
            style
                .color
                .unwrap_or_else(|| color(palette, node.category.unwrap_or(index))),
            style.opacity,
        );
        if let Some(canvas) = canvas {
            let node_border = (style.border_width > 0.0)
                .then(|| style.border_color.map(|color| (color, style.border_width)))
                .flatten();
            draw_symbol(canvas, &symbol, x, y, fill, node_border);
            let label = merge_label_style(&series.options.label, &node.label);
            if label.show {
                let text = node_label(&label, node);
                set_next_data_index(index);
                draw_text(
                    canvas,
                    &text,
                    x + dimensions[0] / 2.0 + label.distance,
                    y + 4.0,
                    label.font_size as f64,
                    label.color.unwrap_or(0xFF333333),
                    label.font_weight,
                );
            }
        }
        hits.push(HitRegion {
            shape: HitShape::Point {
                x,
                y,
                radius: symbol.hit_radius().max(12.0),
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

fn node_label(label: &LabelStyle, node: &NodeData) -> String {
    label
        .formatter
        .as_deref()
        .unwrap_or("{b}")
        .replace("{b}", &node.name)
        .replace("{c}", &format_value(node.value))
}

fn format_value(value: f64) -> String {
    if (value - value.round()).abs() < 1e-6 {
        format!("{value:.0}")
    } else {
        format!("{value:.2}")
    }
}
