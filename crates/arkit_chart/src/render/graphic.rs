//! Native subset of the ECharts `graphic` component for chart annotations.

use ohos_drawing_binding::{Canvas, Path};
use serde_json::{Map, Value};

use super::compat;
use super::surface::{
    draw_text, fill_circle, fill_oval, fill_path, fill_rounded_rect, stroke_circle, stroke_line,
    stroke_oval, stroke_path, stroke_rounded_rect,
};
use crate::model::ChartOption;

pub(super) fn draw_graphic(canvas: &Canvas, option: &ChartOption, width: f32, height: f32) {
    let Some(value) = option.extra.get("graphic") else {
        return;
    };
    match value {
        Value::Array(elements) => {
            for element in elements {
                draw_element(canvas, element, (0.0, 0.0), (width, height));
            }
        }
        Value::Object(object) => {
            if let Some(elements) = object.get("elements").and_then(Value::as_array) {
                for element in elements {
                    draw_element(canvas, element, (0.0, 0.0), (width, height));
                }
            } else {
                draw_element(canvas, value, (0.0, 0.0), (width, height));
            }
        }
        _ => {}
    }
}

fn draw_element(canvas: &Canvas, value: &Value, parent: (f32, f32), size: (f32, f32)) {
    let Some(element) = value.as_object() else {
        return;
    };
    if element
        .get("invisible")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        return;
    }
    let position = (
        parent.0 + element_position(element, "x", "left", size.0),
        parent.1 + element_position(element, "y", "top", size.1),
    );
    let kind = element
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or("group");
    if kind == "group" {
        if let Some(children) = element.get("children").and_then(Value::as_array) {
            for child in children {
                draw_element(canvas, child, position, size);
            }
        }
        return;
    }
    let shape = element.get("shape").and_then(Value::as_object);
    let style = element.get("style").and_then(Value::as_object);
    let fill = style
        .and_then(|style| style.get("fill"))
        .or_else(|| style.and_then(|style| style.get("color")))
        .and_then(crate::parser::parse_color);
    let stroke = style
        .and_then(|style| style.get("stroke"))
        .and_then(crate::parser::parse_color);
    let line_width = style
        .and_then(|style| style.get("lineWidth"))
        .and_then(Value::as_f64)
        .unwrap_or(1.0) as f32;
    match kind {
        "rect" => {
            let x = position.0 + number(shape, "x", 0.0);
            let y = position.1 + number(shape, "y", 0.0);
            let width = number(shape, "width", 0.0).max(0.0);
            let height = number(shape, "height", 0.0).max(0.0);
            let radius = number(shape, "r", 0.0).max(0.0);
            if let Some(fill) = fill {
                fill_rounded_rect(canvas, x, y, width, height, [radius; 4], fill);
            }
            if let Some(stroke) = stroke {
                stroke_rounded_rect(
                    canvas,
                    (x, y, width, height),
                    [radius; 4],
                    stroke,
                    line_width,
                );
            }
        }
        "circle" => {
            let cx = position.0 + number(shape, "cx", 0.0);
            let cy = position.1 + number(shape, "cy", 0.0);
            let radius = number(shape, "r", 0.0).max(0.0);
            if let Some(fill) = fill {
                fill_circle(canvas, cx, cy, radius, fill);
            }
            if let Some(stroke) = stroke {
                stroke_circle(canvas, cx, cy, radius, stroke, line_width);
            }
        }
        "ellipse" => {
            let cx = position.0 + number(shape, "cx", 0.0);
            let cy = position.1 + number(shape, "cy", 0.0);
            let rx = number(shape, "rx", 0.0).max(0.0);
            let ry = number(shape, "ry", 0.0).max(0.0);
            if let Some(fill) = fill {
                fill_oval(canvas, cx - rx, cy - ry, rx * 2.0, ry * 2.0, fill);
            }
            if let Some(stroke) = stroke {
                stroke_oval(
                    canvas,
                    cx - rx,
                    cy - ry,
                    rx * 2.0,
                    ry * 2.0,
                    stroke,
                    line_width,
                );
            }
        }
        "line" => {
            stroke_line(
                canvas,
                position.0 + number(shape, "x1", 0.0),
                position.1 + number(shape, "y1", 0.0),
                position.0 + number(shape, "x2", 0.0),
                position.1 + number(shape, "y2", 0.0),
                stroke.or(fill).unwrap_or(0xFF000000),
                line_width,
            );
        }
        "polygon" | "polyline" => {
            let points = shape
                .and_then(|shape| shape.get("points"))
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(|point| {
                    let point = point.as_array()?;
                    Some((
                        position.0 + point.first()?.as_f64()? as f32,
                        position.1 + point.get(1)?.as_f64()? as f32,
                    ))
                })
                .collect::<Vec<_>>();
            let mut path = Path::new();
            for (index, point) in points.iter().enumerate() {
                if index == 0 {
                    path.move_to(point.0, point.1);
                } else {
                    path.line_to(point.0, point.1);
                }
            }
            if kind == "polygon" {
                path.close();
                if let Some(fill) = fill {
                    fill_path(canvas, &path, fill);
                }
            }
            if let Some(stroke) = stroke {
                stroke_path(canvas, &path, stroke, line_width);
            }
        }
        "text" => {
            let text = style
                .and_then(|style| style.get("text"))
                .and_then(Value::as_str)
                .unwrap_or_default();
            let font_size = style
                .and_then(|style| style.get("fontSize"))
                .and_then(Value::as_f64)
                .unwrap_or(12.0);
            draw_text(
                canvas,
                text,
                position.0,
                position.1 + font_size as f32,
                font_size,
                fill.unwrap_or(0xFF1F2937),
                400,
            );
        }
        _ => {}
    }
}

fn element_position(element: &Map<String, Value>, direct: &str, layout: &str, total: f32) -> f32 {
    element
        .get(direct)
        .or_else(|| element.get(layout))
        .map(|value| compat::length(Some(value), total, 0.0))
        .unwrap_or(0.0)
}

fn number(value: Option<&Map<String, Value>>, key: &str, default: f32) -> f32 {
    value
        .and_then(|value| value.get(key))
        .and_then(Value::as_f64)
        .map(|value| value as f32)
        .unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn graphic_positions_accept_percentages() {
        let element = serde_json::json!({"left":"25%"});
        assert_eq!(
            element_position(element.as_object().unwrap(), "x", "left", 200.0),
            50.0
        );
    }
}
