use arkit::prelude::{CanvasColor, CanvasLineCap, CanvasLineJoin};
use arkit::{CanvasRenderingContext2D, Path2D};
use serde_json::Value;

const TIGER_DATA: &str = include_str!("../assets/tiger.json");
const SOURCE_WIDTH: f32 = 1024.0;
const SOURCE_HEIGHT: f32 = 768.0;
const CONTENT_PADDING: f32 = 12.0;

struct TigerLayer {
    path: Path2D,
    fill: CanvasColor,
    stroke: CanvasColor,
    line_width: f32,
}

/// The classic Canvas tiger, parsed into native paths once per component
/// lifetime. Redraws only update the native transform and repaint the cached
/// paths; the 95 KiB SVG path source is never reparsed in a draw callback.
pub(crate) struct TigerScene {
    layers: Vec<TigerLayer>,
}

impl TigerScene {
    pub(crate) fn load() -> Self {
        let source: Value =
            serde_json::from_str(TIGER_DATA).expect("embedded Canvas tiger JSON must be valid");
        let layers = source
            .as_array()
            .expect("embedded Canvas tiger root must be an array")
            .iter()
            .map(|layer| {
                let path = required_string(layer, "d");
                let fill = required_string(layer, "fillStyle");
                let stroke = required_string(layer, "strokeStyle");
                let line_width =
                    layer
                        .get("lineWidth")
                        .and_then(Value::as_str)
                        .map_or(1.0, |width| {
                            width
                                .parse::<f32>()
                                .expect("Canvas tiger lineWidth must be numeric")
                        });
                TigerLayer {
                    path: Path2D::from_svg(path).expect("Canvas tiger path must be valid SVG data"),
                    fill: CanvasColor::parse_css(fill)
                        .expect("Canvas tiger fillStyle must be a CSS color"),
                    stroke: CanvasColor::parse_css(stroke)
                        .expect("Canvas tiger strokeStyle must be a CSS color"),
                    line_width,
                }
            })
            .collect();
        Self { layers }
    }

    pub(crate) fn draw(&self, context: &mut CanvasRenderingContext2D<'_>, zoom: f32) {
        draw_background(context);

        let available_width = (context.width() - CONTENT_PADDING * 2.0).max(1.0);
        let available_height = (context.height() - CONTENT_PADDING * 2.0).max(1.0);
        let scale = (available_width / SOURCE_WIDTH).min(available_height / SOURCE_HEIGHT) * zoom;

        context.save();
        context.translate(context.width() * 0.5, context.height() * 0.5);
        context.scale(scale, scale);
        context.translate(-SOURCE_WIDTH * 0.5, -SOURCE_HEIGHT * 0.5);
        context.set_line_cap(CanvasLineCap::Butt);
        context.set_line_join(CanvasLineJoin::Miter);

        for layer in &self.layers {
            context.set_line_width(layer.line_width);
            context.set_stroke_style(layer.stroke);
            context.stroke_path(&layer.path);
            context.set_fill_style(layer.fill);
            context.fill_path(&layer.path);
        }
        context.restore();
    }
}

fn required_string<'a>(value: &'a Value, key: &str) -> &'a str {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("Canvas tiger layer must contain string field `{key}`"))
}

fn draw_background(context: &mut CanvasRenderingContext2D<'_>) {
    if let Ok(background) =
        context.create_linear_gradient(0.0, 0.0, context.width(), context.height())
    {
        let _ = background.add_color_stop(0.0, "#fff7ed");
        let _ = background.add_color_stop(0.55, "#ffffff");
        let _ = background.add_color_stop(1.0, "#fef3c7");
        context.set_fill_style(background);
    } else {
        context.set_fill_style("#ffffff");
    }
    context.fill_rect(0.0, 0.0, context.width(), context.height());
}
