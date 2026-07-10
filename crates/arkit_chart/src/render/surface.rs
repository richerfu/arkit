//! Minimal native-canvas drawing atoms used to compose series renderers.

use ohos_drawing_binding::{
    Brush, Canvas, FontCollection, Path, Pen, Point, Rect, TextStyle, TypographyBuilder,
    TypographyStyle,
};

pub(super) fn fill_rect(canvas: &Canvas, x: f32, y: f32, width: f32, height: f32, color: u32) {
    let mut brush = Brush::new();
    brush.set_anti_alias(true);
    brush.set_color(color);
    let rect = Rect::new(x, y, x + width.max(0.0), y + height.max(0.0));
    canvas.attach_brush(&brush);
    canvas.draw_rect(&rect);
    canvas.detach_brush();
}

pub(super) fn fill_circle(canvas: &Canvas, x: f32, y: f32, radius: f32, color: u32) {
    let mut brush = Brush::new();
    brush.set_anti_alias(true);
    brush.set_color(color);
    let point = Point::new(x, y);
    canvas.attach_brush(&brush);
    canvas.draw_circle(&point, radius);
    canvas.detach_brush();
}

pub(super) fn stroke_rect(
    canvas: &Canvas,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    color: u32,
    stroke_width: f32,
) {
    let mut pen = Pen::new();
    pen.set_anti_alias(true);
    pen.set_color(color);
    pen.set_width(stroke_width);
    let rect = Rect::new(x, y, x + width.max(0.0), y + height.max(0.0));
    canvas.attach_pen(&pen);
    canvas.draw_rect(&rect);
    canvas.detach_pen();
}

pub(super) fn stroke_circle(canvas: &Canvas, x: f32, y: f32, radius: f32, color: u32, width: f32) {
    let mut pen = Pen::new();
    pen.set_anti_alias(true);
    pen.set_color(color);
    pen.set_width(width);
    let point = Point::new(x, y);
    canvas.attach_pen(&pen);
    canvas.draw_circle(&point, radius);
    canvas.detach_pen();
}

pub(super) fn stroke_line(
    canvas: &Canvas,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    color: u32,
    width: f32,
) {
    let mut pen = Pen::new();
    pen.set_anti_alias(true);
    pen.set_color(color);
    pen.set_width(width);
    canvas.attach_pen(&pen);
    canvas.draw_line(x1, y1, x2, y2);
    canvas.detach_pen();
}

pub(super) fn stroke_path(canvas: &Canvas, path: &Path, color: u32, width: f32) {
    let mut pen = Pen::new();
    pen.set_anti_alias(true);
    pen.set_color(color);
    pen.set_width(width);
    canvas.attach_pen(&pen);
    canvas.draw_path(path);
    canvas.detach_pen();
}

pub(super) fn fill_path(canvas: &Canvas, path: &Path, color: u32) {
    let mut brush = Brush::new();
    brush.set_anti_alias(true);
    brush.set_color(color);
    canvas.attach_brush(&brush);
    canvas.draw_path(path);
    canvas.detach_brush();
}

pub(super) fn fill_sector(
    canvas: &Canvas,
    cx: f32,
    cy: f32,
    radius: f32,
    start: f32,
    sweep: f32,
    color: u32,
) {
    let mut path = Path::new();
    path.move_to(cx, cy);
    path.line_to(cx + start.cos() * radius, cy + start.sin() * radius);
    path.arc_to(
        cx - radius,
        cy - radius,
        cx + radius,
        cy + radius,
        start.to_degrees(),
        sweep.to_degrees(),
    );
    path.close();
    fill_path(canvas, &path, color);
}

pub(super) fn fill_ring_sector(
    canvas: &Canvas,
    center: (f32, f32),
    radii: (f32, f32),
    start: f32,
    sweep: f32,
    color: u32,
) {
    let (cx, cy) = center;
    let (inner, outer) = radii;
    if inner <= 0.0 {
        fill_sector(canvas, cx, cy, outer, start, sweep, color);
        return;
    }
    let end = start + sweep;
    let mut path = Path::new();
    path.move_to(cx + start.cos() * outer, cy + start.sin() * outer);
    path.arc_to(
        cx - outer,
        cy - outer,
        cx + outer,
        cy + outer,
        start.to_degrees(),
        sweep.to_degrees(),
    );
    path.line_to(cx + end.cos() * inner, cy + end.sin() * inner);
    path.arc_to(
        cx - inner,
        cy - inner,
        cx + inner,
        cy + inner,
        end.to_degrees(),
        -sweep.to_degrees(),
    );
    path.close();
    fill_path(canvas, &path, color);
}

pub(super) fn stroke_arc(
    canvas: &Canvas,
    center: (f32, f32),
    radius: f32,
    start: f32,
    sweep: f32,
    color: u32,
    width: f32,
) {
    let (cx, cy) = center;
    let mut path = Path::new();
    path.arc_to(
        cx - radius,
        cy - radius,
        cx + radius,
        cy + radius,
        start.to_degrees(),
        sweep.to_degrees(),
    );
    stroke_path(canvas, &path, color, width);
}

pub(super) fn draw_text(
    canvas: &Canvas,
    text: &str,
    x: f32,
    y: f32,
    size: f64,
    color: u32,
    weight: i32,
) {
    if text.is_empty() {
        return;
    }
    let mut font_collection = FontCollection::global_instance().unwrap_or_default();
    let mut typography_style = TypographyStyle::new();
    let mut text_style = TextStyle::new();
    text_style.set_color(color);
    text_style.set_font_size(size);
    text_style.set_font_weight(weight);
    let mut builder = TypographyBuilder::new(&mut typography_style, &mut font_collection);
    builder.push_text_style(&mut text_style);
    builder.add_text(text);
    builder.pop_text_style();
    let mut typography = builder.build();
    typography.layout(260.0);
    typography.paint(canvas, x as f64, (y - size as f32) as f64);
}
