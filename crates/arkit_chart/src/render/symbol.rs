//! Shared ECharts symbol resolution and native-canvas drawing.

use ohos_drawing_binding::Canvas;

use super::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct SymbolSpec<'a> {
    pub(super) name: &'a str,
    pub(super) size: [f32; 2],
    pub(super) rotate: f32,
    pub(super) offset: [f32; 2],
}

impl SymbolSpec<'_> {
    pub(super) fn center(&self, x: f32, y: f32) -> (f32, f32) {
        (x + self.offset[0], y + self.offset[1])
    }

    pub(super) fn hit_radius(&self) -> f32 {
        self.size[0].max(self.size[1]) / 2.0
    }
}

pub(super) fn resolve_symbol<'a>(
    series: &'a BasicSeries,
    point: &'a DataPoint,
    visual_size: Option<[f32; 2]>,
) -> SymbolSpec<'a> {
    let name = point
        .extra
        .get("symbol")
        .and_then(serde_json::Value::as_str)
        .unwrap_or(&series.options.symbol);
    let size = point
        .extra
        .get("symbolSize")
        .and_then(symbol_dimensions)
        .or(visual_size)
        .or(series.options.symbol_size_dimensions)
        .unwrap_or([series.options.symbol_size; 2])
        .map(|value| value.max(0.0));
    let rotate = point
        .extra
        .get("symbolRotate")
        .and_then(serde_json::Value::as_f64)
        .map(|value| value as f32)
        .unwrap_or(series.options.symbol_rotate);
    let offsets = point
        .extra
        .get("symbolOffset")
        .and_then(serde_json::Value::as_array)
        .map(|values| {
            [
                values.first().unwrap_or(&series.options.symbol_offset[0]),
                values.get(1).unwrap_or(&series.options.symbol_offset[1]),
            ]
        })
        .unwrap_or([
            &series.options.symbol_offset[0],
            &series.options.symbol_offset[1],
        ]);
    SymbolSpec {
        name,
        size,
        rotate,
        offset: [
            resolve_offset(offsets[0], size[0]),
            resolve_offset(offsets[1], size[1]),
        ],
    }
}

pub(super) fn draw_symbol(
    canvas: &Canvas,
    spec: &SymbolSpec<'_>,
    x: f32,
    y: f32,
    color: u32,
    border: Option<(u32, f32)>,
) {
    let [width, height] = spec.size;
    if spec.name == "none" || width <= 0.0 || height <= 0.0 {
        return;
    }
    let (x, y) = spec.center(x, y);
    canvas.save();
    if spec.rotate.abs() > f32::EPSILON {
        canvas.rotate_degrees_around(spec.rotate, x, y);
    }
    let left = x - width / 2.0;
    let top = y - height / 2.0;
    match spec.name {
        "emptyCircle" => {
            fill_oval(canvas, left, top, width, height, 0xFFFFFFFF);
            stroke_oval(
                canvas,
                left,
                top,
                width,
                height,
                color,
                border.map_or(2.0, |value| value.1),
            );
        }
        "rect" | "roundRect" => {
            let radii = if spec.name == "roundRect" {
                [width.min(height) * 0.25; 4]
            } else {
                [0.0; 4]
            };
            fill_rounded_rect(canvas, left, top, width, height, radii, color);
            if let Some((border_color, border_width)) = border {
                stroke_rounded_rect(
                    canvas,
                    (left, top, width, height),
                    radii,
                    border_color,
                    border_width,
                );
            }
        }
        "triangle" | "diamond" | "arrow" | "pin" => {
            let mut path = Path::new();
            match spec.name {
                "diamond" => {
                    path.move_to(x, top);
                    path.line_to(left + width, y);
                    path.line_to(x, top + height);
                    path.line_to(left, y);
                }
                "arrow" => {
                    path.move_to(x, top);
                    path.line_to(left + width, top + height);
                    path.line_to(x, top + height * 0.72);
                    path.line_to(left, top + height);
                }
                "pin" => {
                    path.move_to(x, top + height);
                    path.line_to(left, top + height * 0.38);
                    path.line_to(x, top);
                    path.line_to(left + width, top + height * 0.38);
                }
                _ => {
                    path.move_to(x, top);
                    path.line_to(left + width, top + height);
                    path.line_to(left, top + height);
                }
            }
            path.close();
            fill_path(canvas, &path, color);
            if let Some((border_color, border_width)) = border {
                stroke_path(canvas, &path, border_color, border_width);
            }
        }
        _ => {
            fill_oval(canvas, left, top, width, height, color);
            if let Some((border_color, border_width)) = border {
                stroke_oval(canvas, left, top, width, height, border_color, border_width);
            }
        }
    }
    canvas.restore();
}

fn symbol_dimensions(value: &serde_json::Value) -> Option<[f32; 2]> {
    if let Some(size) = value.as_f64() {
        let size = (size as f32).max(0.0);
        return Some([size, size]);
    }
    let values = value.as_array()?;
    let width = values.first()?.as_f64()? as f32;
    let height = values
        .get(1)
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(width as f64) as f32;
    Some([width.max(0.0), height.max(0.0)])
}

fn resolve_offset(value: &serde_json::Value, size: f32) -> f32 {
    value.as_f64().map(|value| value as f32).unwrap_or_else(|| {
        value
            .as_str()
            .and_then(|value| value.strip_suffix('%'))
            .and_then(|value| value.parse::<f32>().ok())
            .map(|value| size * value / 100.0)
            .unwrap_or(0.0)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_symbol_dimensions_and_offsets_override_series() {
        let mut series = BasicSeries::new("scatter", [1.0]);
        series.options.symbol = String::from("circle");
        series.options.symbol_size_dimensions = Some([12.0, 8.0]);
        let mut point = DataPoint::scalar(1.0);
        point
            .extra
            .insert(String::from("symbolSize"), serde_json::json!([20, 10]));
        point.extra.insert(
            String::from("symbolOffset"),
            serde_json::json!(["50%", "-50%"]),
        );
        point
            .extra
            .insert(String::from("symbol"), serde_json::json!("diamond"));

        let spec = resolve_symbol(&series, &point, Some([30.0, 30.0]));
        assert_eq!(spec.name, "diamond");
        assert_eq!(spec.size, [20.0, 10.0]);
        assert_eq!(spec.offset, [10.0, -5.0]);
    }
}
