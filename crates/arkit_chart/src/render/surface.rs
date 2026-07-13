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

pub(super) fn fill_rounded_rect(
    canvas: &Canvas,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    radii: [f32; 4],
    color: u32,
) {
    if radii.iter().all(|radius| *radius <= f32::EPSILON) {
        fill_rect(canvas, x, y, width, height, color);
        return;
    }
    let path = rounded_rect_path(x, y, width, height, radii);
    fill_path(canvas, &path, color);
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

pub(super) fn fill_oval(canvas: &Canvas, x: f32, y: f32, width: f32, height: f32, color: u32) {
    let mut brush = Brush::new();
    brush.set_anti_alias(true);
    brush.set_color(color);
    let rect = Rect::new(x, y, x + width.max(0.0), y + height.max(0.0));
    canvas.attach_brush(&brush);
    unsafe {
        ohos_native_drawing_sys::OH_Drawing_CanvasDrawOval(canvas.as_ptr(), rect.as_ptr());
    }
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

pub(super) fn stroke_rounded_rect(
    canvas: &Canvas,
    rect: (f32, f32, f32, f32),
    radii: [f32; 4],
    color: u32,
    stroke_width: f32,
) {
    let (x, y, width, height) = rect;
    if radii.iter().all(|radius| *radius <= f32::EPSILON) {
        stroke_rect(canvas, x, y, width, height, color, stroke_width);
        return;
    }
    let path = rounded_rect_path(x, y, width, height, radii);
    stroke_path(canvas, &path, color, stroke_width);
}

fn rounded_rect_path(x: f32, y: f32, width: f32, height: f32, radii: [f32; 4]) -> Path {
    let width = width.max(0.0);
    let height = height.max(0.0);
    let mut radii = radii.map(|radius| radius.max(0.0));
    let fit = |limit: f32, sum: f32| {
        if sum > limit && sum > 0.0 {
            limit / sum
        } else {
            1.0
        }
    };
    let ratios = [
        fit(width, radii[0] + radii[1]),
        fit(width, radii[3] + radii[2]),
        fit(height, radii[0] + radii[3]),
        fit(height, radii[1] + radii[2]),
    ];
    let scale = ratios.into_iter().fold(1.0_f32, f32::min).clamp(0.0, 1.0);
    radii.iter_mut().for_each(|radius| *radius *= scale);
    let [top_left, top_right, bottom_right, bottom_left] = radii;
    let right = x + width;
    let bottom = y + height;

    let mut path = Path::new();
    path.move_to(x + top_left, y);
    path.line_to(right - top_right, y);
    if top_right > 0.0 {
        path.arc_to(
            right - top_right * 2.0,
            y,
            right,
            y + top_right * 2.0,
            -90.0,
            90.0,
        );
    }
    path.line_to(right, bottom - bottom_right);
    if bottom_right > 0.0 {
        path.arc_to(
            right - bottom_right * 2.0,
            bottom - bottom_right * 2.0,
            right,
            bottom,
            0.0,
            90.0,
        );
    }
    path.line_to(x + bottom_left, bottom);
    if bottom_left > 0.0 {
        path.arc_to(
            x,
            bottom - bottom_left * 2.0,
            x + bottom_left * 2.0,
            bottom,
            90.0,
            90.0,
        );
    }
    path.line_to(x, y + top_left);
    if top_left > 0.0 {
        path.arc_to(x, y, x + top_left * 2.0, y + top_left * 2.0, 180.0, 90.0);
    }
    path.close();
    path
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

pub(super) fn stroke_oval(
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
    unsafe {
        ohos_native_drawing_sys::OH_Drawing_CanvasDrawOval(canvas.as_ptr(), rect.as_ptr());
    }
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
    stroke_path_style(canvas, path, color, width, "solid");
}

pub(super) fn stroke_path_style(canvas: &Canvas, path: &Path, color: u32, width: f32, kind: &str) {
    let mut pen = Pen::new();
    pen.set_anti_alias(true);
    pen.set_color(color);
    pen.set_width(width);
    let effect = match kind {
        "dashed" => Some([width.max(1.0) * 4.0, width.max(1.0) * 2.0]),
        "dotted" => Some([width.max(1.0), width.max(1.0) * 2.0]),
        _ => None,
    }
    .and_then(|mut intervals| {
        let effect = unsafe {
            ohos_native_drawing_sys::OH_Drawing_CreateDashPathEffect(
                intervals.as_mut_ptr(),
                intervals.len() as i32,
                0.0,
            )
        };
        (!effect.is_null()).then_some(effect)
    });
    if let Some(effect) = effect {
        unsafe { ohos_native_drawing_sys::OH_Drawing_PenSetPathEffect(pen.as_ptr(), effect) };
    }
    canvas.attach_pen(&pen);
    canvas.draw_path(path);
    canvas.detach_pen();
    if let Some(effect) = effect {
        unsafe { ohos_native_drawing_sys::OH_Drawing_PathEffectDestroy(effect) };
    }
}

pub(super) fn fill_path(canvas: &Canvas, path: &Path, color: u32) {
    let mut brush = Brush::new();
    brush.set_anti_alias(true);
    brush.set_color(color);
    canvas.attach_brush(&brush);
    canvas.draw_path(path);
    canvas.detach_brush();
}

pub(super) fn fill_ring_sector(
    canvas: &Canvas,
    center: (f32, f32),
    radii: (f32, f32),
    start: f32,
    sweep: f32,
    color: u32,
) {
    let path = ring_sector_path(center, radii, start, sweep);
    fill_path(canvas, &path, color);
}

pub(super) fn stroke_ring_sector(
    canvas: &Canvas,
    center: (f32, f32),
    radii: (f32, f32),
    start: f32,
    sweep: f32,
    color: u32,
    width: f32,
) {
    let path = ring_sector_path(center, radii, start, sweep);
    stroke_path(canvas, &path, color, width);
}

fn ring_sector_path(center: (f32, f32), radii: (f32, f32), start: f32, sweep: f32) -> Path {
    let (cx, cy) = center;
    let (inner, outer) = radii;
    let end = start + sweep;
    let mut path = Path::new();
    if inner <= 0.0 {
        path.move_to(cx, cy);
        path.line_to(cx + start.cos() * outer, cy + start.sin() * outer);
        path.arc_to(
            cx - outer,
            cy - outer,
            cx + outer,
            cy + outer,
            start.to_degrees(),
            sweep.to_degrees(),
        );
        path.close();
        return path;
    }
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
    path
}

#[allow(clippy::too_many_arguments)]
pub(super) fn stroke_arc_with_cap(
    canvas: &Canvas,
    center: (f32, f32),
    radius: f32,
    start: f32,
    sweep: f32,
    color: u32,
    width: f32,
    round_cap: bool,
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
    let mut pen = Pen::new();
    pen.set_anti_alias(true);
    pen.set_color(color);
    pen.set_width(width);
    if round_cap {
        unsafe {
            ohos_native_drawing_sys::OH_Drawing_PenSetCap(
                pen.as_ptr(),
                ohos_native_drawing_sys::OH_Drawing_PenLineCapStyle_LINE_ROUND_CAP,
            );
        }
    }
    canvas.attach_pen(&pen);
    canvas.draw_path(&path);
    canvas.detach_pen();
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

#[allow(clippy::too_many_arguments)]
pub(super) fn draw_rotated_text(
    canvas: &Canvas,
    text: &str,
    x: f32,
    y: f32,
    pivot_x: f32,
    pivot_y: f32,
    degrees: f32,
    size: f64,
    color: u32,
    weight: i32,
) {
    if degrees.abs() <= f32::EPSILON {
        draw_text(canvas, text, x, y, size, color, weight);
        return;
    }
    canvas.save();
    unsafe {
        ohos_native_drawing_sys::OH_Drawing_CanvasRotate(
            canvas.as_ptr(),
            degrees,
            pivot_x,
            pivot_y,
        );
    }
    draw_text(canvas, text, x, y, size, color, weight);
    canvas.restore();
}
