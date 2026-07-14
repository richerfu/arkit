//! Minimal native-canvas drawing atoms used to compose series renderers.

use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::sync::Arc;

use ohos_drawing_binding::{
    Brush, Canvas, FontCollection, Path, Pen, Point, Rect, TextStyle, Typography,
    TypographyBuilder, TypographyStyle,
};
use rustc_hash::FxHashMap;

const TEXT_CACHE_CAPACITY: usize = 256;

thread_local! {
    static TEXT_CACHE: RefCell<TextCache> = RefCell::new(TextCache::new());
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct TextStyleKey {
    size_bits: u64,
    color: u32,
    weight: i32,
}

struct TextPaint<'a> {
    text: &'a str,
    x: f64,
    y: f64,
    size: f64,
    color: u32,
    weight: i32,
}

struct TextCache {
    fonts: FontCollection,
    entries: FxHashMap<TextStyleKey, FxHashMap<Arc<str>, Typography>>,
    order: VecDeque<(TextStyleKey, Arc<str>)>,
    len: usize,
}

impl TextCache {
    fn new() -> Self {
        Self {
            fonts: FontCollection::global_instance().unwrap_or_default(),
            entries: FxHashMap::default(),
            order: VecDeque::with_capacity(TEXT_CACHE_CAPACITY),
            len: 0,
        }
    }

    fn paint(&mut self, canvas: &Canvas, paint: TextPaint<'_>) {
        let text = if paint.text.contains('\0') {
            Cow::Owned(paint.text.replace('\0', "\u{fffd}"))
        } else {
            Cow::Borrowed(paint.text)
        };
        let style = TextStyleKey {
            size_bits: paint.size.to_bits(),
            color: paint.color,
            weight: paint.weight,
        };
        if let Some(typography) = self
            .entries
            .get_mut(&style)
            .and_then(|entries| entries.get_mut(text.as_ref()))
        {
            typography.paint(canvas, paint.x, paint.y);
            return;
        }

        if self.len == TEXT_CACHE_CAPACITY {
            if let Some((retired_style, retired_text)) = self.order.pop_front() {
                let mut removed = false;
                let remove_bucket = if let Some(entries) = self.entries.get_mut(&retired_style) {
                    removed = entries.remove(retired_text.as_ref()).is_some();
                    entries.is_empty()
                } else {
                    false
                };
                if remove_bucket {
                    drop(self.entries.remove(&retired_style));
                }
                if removed {
                    self.len -= 1;
                }
            }
        }

        let text: Arc<str> = Arc::from(text.into_owned());
        let mut typography_style = TypographyStyle::new();
        let mut text_style = TextStyle::new();
        text_style.set_color(paint.color);
        text_style.set_font_size(paint.size);
        text_style.set_font_weight(paint.weight);
        let mut builder = TypographyBuilder::new(&mut typography_style, &mut self.fonts);
        builder.push_text_style(&mut text_style);
        builder.add_text(&text);
        builder.pop_text_style();
        let mut typography = builder.build();
        // Chart labels are single-line drawing atoms. A large finite width
        // avoids the old 260px wrap/clipping bug without passing infinity into
        // the native typography engine.
        typography.layout(1_000_000.0);
        typography.paint(canvas, paint.x, paint.y);
        self.entries
            .entry(style)
            .or_default()
            .insert(text.clone(), typography);
        self.order.push_back((style, text));
        self.len += 1;
    }
}

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
    // SAFETY: canvas and rect are live for this synchronous draw call; the
    // attached brush is detached before any owner is dropped.
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
    // SAFETY: canvas and rect are live for this synchronous draw call; the
    // attached pen is detached before any owner is dropped.
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
        // SAFETY: the native constructor consumes the interval values during
        // this call; both entries remain initialized for its duration.
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
        // SAFETY: `pen` and the non-null effect are live. The effect remains
        // owned here until drawing is complete and the pen is detached.
        unsafe { ohos_native_drawing_sys::OH_Drawing_PenSetPathEffect(pen.as_ptr(), effect) };
    }
    canvas.attach_pen(&pen);
    canvas.draw_path(path);
    canvas.detach_pen();
    if let Some(effect) = effect {
        // SAFETY: the pen no longer references the effect and this scope owns
        // the only native effect handle.
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
        // SAFETY: `pen` is a live uniquely configured native pen handle.
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
    TEXT_CACHE.with_borrow_mut(|cache| {
        cache.paint(
            canvas,
            TextPaint {
                text,
                x: x as f64,
                y: (y - size as f32) as f64,
                size,
                color,
                weight,
            },
        );
    });
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
    // SAFETY: `canvas` remains live and the transform is balanced by restore
    // before returning.
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
