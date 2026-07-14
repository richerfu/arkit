//! Canvas-wide label overlap handling shared by every series renderer.

use std::cell::RefCell;

use ohos_drawing_binding::Canvas;
use serde_json::Value;

use crate::model::{LabelLayoutCallbackParams, LabelLayoutOptions};

#[derive(Clone, Copy)]
pub(crate) struct LabelRect {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) width: f32,
    pub(crate) height: f32,
}

pub(crate) struct LabelHit {
    pub(crate) series_index: usize,
    pub(crate) label_index: usize,
    pub(crate) text: String,
    pub(crate) bounds: LabelRect,
}

#[derive(Default)]
struct LabelLayoutFrame {
    occupied: Vec<LabelRect>,
    policy: Option<LabelLayoutOptions>,
    series_index: usize,
    next_label_index: usize,
    next_data_index: Option<usize>,
    next_label_line_points: Option<Vec<[f32; 2]>>,
    last_label_line_points: Option<Vec<[f32; 2]>>,
    draggable_hits: Vec<LabelHit>,
    width: f32,
    height: f32,
}

thread_local! {
    static FRAME: RefCell<LabelLayoutFrame> = RefCell::new(LabelLayoutFrame::default());
}

pub(super) fn begin_frame(width: f32, height: f32) {
    FRAME.with(|frame| {
        *frame.borrow_mut() = LabelLayoutFrame {
            width,
            height,
            ..LabelLayoutFrame::default()
        }
    });
}

pub(super) fn set_policy(policy: &LabelLayoutOptions, series_index: usize) {
    FRAME.with(|frame| {
        let mut frame = frame.borrow_mut();
        frame.policy = Some(policy.clone());
        frame.series_index = series_index;
        frame.next_label_index = 0;
        frame.next_data_index = None;
        frame.next_label_line_points = None;
        frame.last_label_line_points = None;
    });
}

pub(super) fn clear_policy() {
    FRAME.with(|frame| frame.borrow_mut().policy = None);
}

pub(super) fn set_next_data_index(data_index: usize) {
    FRAME.with(|frame| frame.borrow_mut().next_data_index = Some(data_index));
}

pub(super) fn set_next_label_line_points(points: Vec<[f32; 2]>) {
    FRAME.with(|frame| {
        let mut frame = frame.borrow_mut();
        frame.next_label_line_points = Some(points);
        frame.last_label_line_points = None;
    });
}

pub(super) fn take_last_label_line_points() -> Option<Vec<[f32; 2]>> {
    FRAME.with(|frame| frame.borrow_mut().last_label_line_points.take())
}

pub(crate) fn take_draggable_hits() -> Vec<LabelHit> {
    FRAME.with(|frame| std::mem::take(&mut frame.borrow_mut().draggable_hits))
}

pub(super) fn draw_text(
    canvas: &Canvas,
    text: &str,
    mut x: f32,
    mut baseline: f32,
    size: f64,
    color: u32,
    weight: i32,
) {
    let (policy, series_index, label_index, frame_size, label_line_points) = FRAME.with(|frame| {
        let mut frame = frame.borrow_mut();
        let data_index = frame.next_data_index.take();
        let policy = data_index.and_then(|_| frame.policy.clone());
        let series_index = frame.series_index;
        let label_index = data_index.unwrap_or(frame.next_label_index);
        if policy.is_some() {
            frame.next_label_index = frame.next_label_index.max(label_index + 1);
        }
        (
            policy,
            series_index,
            label_index,
            (frame.width, frame.height),
            frame.next_label_line_points.take(),
        )
    });
    let Some(mut policy) = policy else {
        super::surface::draw_text(canvas, text, x, baseline, size, color, weight);
        return;
    };
    if let Some(points) = label_line_points {
        policy.label_line_points = points;
    }
    apply_callback(
        &mut policy,
        series_index,
        label_index,
        text,
        x,
        baseline,
        size as f32,
    );
    FRAME.with(|frame| {
        frame.borrow_mut().last_label_line_points = Some(policy.label_line_points.clone())
    });
    let (layout_x, layout_y, size, width, height) =
        apply_layout(&policy, text, x, baseline, size as f32, frame_size);
    x = layout_x;
    baseline = layout_y;
    if let Some(offset) = policy.drag_offsets.get(&label_index) {
        x += offset[0];
        baseline += offset[1];
    }
    let mut bounds = LabelRect {
        x,
        y: baseline - size,
        width,
        height,
    };
    let visible = FRAME.with(|frame| {
        let mut frame = frame.borrow_mut();
        if let Some(direction) = policy.move_overlap.as_deref() {
            let vertical = matches!(direction, "shiftY" | "shuffleY");
            let step = (size * 0.6).max(2.0);
            for offset in std::iter::once(0.0).chain((1..=8).flat_map(|index| {
                let offset = index as f32 * step;
                [offset, -offset]
            })) {
                let mut candidate = bounds;
                if vertical {
                    candidate.y += offset;
                } else {
                    candidate.x += offset;
                }
                if !frame
                    .occupied
                    .iter()
                    .any(|occupied| overlaps(candidate, *occupied))
                {
                    bounds = candidate;
                    if vertical {
                        baseline += offset;
                    } else {
                        x += offset;
                    }
                    break;
                }
            }
        }
        let collided = frame
            .occupied
            .iter()
            .any(|occupied| overlaps(bounds, *occupied));
        if collided && policy.hide_overlap {
            false
        } else {
            frame.occupied.push(bounds);
            if policy.draggable {
                frame.draggable_hits.push(LabelHit {
                    series_index,
                    label_index,
                    text: text.to_string(),
                    bounds: padded_hit_bounds(bounds),
                });
            }
            true
        }
    });
    if visible {
        if let Some(degrees) = policy.rotate.filter(|degrees| degrees.abs() > f32::EPSILON) {
            super::surface::draw_rotated_text(
                canvas,
                text,
                x,
                baseline,
                x,
                baseline,
                degrees,
                size as f64,
                color,
                weight,
            );
        } else {
            super::surface::draw_text(canvas, text, x, baseline, size as f64, color, weight);
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn draw_rotated_text(
    canvas: &Canvas,
    text: &str,
    mut x: f32,
    mut y: f32,
    mut pivot_x: f32,
    mut pivot_y: f32,
    degrees: f32,
    size: f64,
    color: u32,
    weight: i32,
) {
    let (policy, series_index, label_index, frame_size, label_line_points) = FRAME.with(|frame| {
        let mut frame = frame.borrow_mut();
        let data_index = frame.next_data_index.take();
        let policy = data_index.and_then(|_| frame.policy.clone());
        let series_index = frame.series_index;
        let label_index = data_index.unwrap_or(frame.next_label_index);
        if policy.is_some() {
            frame.next_label_index = frame.next_label_index.max(label_index + 1);
        }
        (
            policy,
            series_index,
            label_index,
            (frame.width, frame.height),
            frame.next_label_line_points.take(),
        )
    });
    let Some(mut policy) = policy else {
        super::surface::draw_rotated_text(
            canvas, text, x, y, pivot_x, pivot_y, degrees, size, color, weight,
        );
        return;
    };
    if let Some(points) = label_line_points {
        policy.label_line_points = points;
    }
    apply_callback(
        &mut policy,
        series_index,
        label_index,
        text,
        x,
        y,
        size as f32,
    );
    FRAME.with(|frame| {
        frame.borrow_mut().last_label_line_points = Some(policy.label_line_points.clone())
    });
    let (layout_x, layout_y, layout_size, estimated_width, estimated_height) =
        apply_layout(&policy, text, x, y, size as f32, frame_size);
    let delta = (layout_x - x, layout_y - y);
    x = layout_x;
    y = layout_y;
    pivot_x += delta.0;
    pivot_y += delta.1;
    if let Some(offset) = policy.drag_offsets.get(&label_index) {
        x += offset[0];
        y += offset[1];
        pivot_x += offset[0];
        pivot_y += offset[1];
    }
    let bounds = LabelRect {
        x,
        y: y - layout_size,
        width: estimated_width,
        height: estimated_height,
    };
    let visible = FRAME.with(|frame| {
        let mut frame = frame.borrow_mut();
        let collided = frame
            .occupied
            .iter()
            .any(|occupied| overlaps(bounds, *occupied));
        if collided && policy.hide_overlap {
            false
        } else {
            frame.occupied.push(bounds);
            if policy.draggable {
                frame.draggable_hits.push(LabelHit {
                    series_index,
                    label_index,
                    text: text.to_string(),
                    bounds: padded_hit_bounds(bounds),
                });
            }
            true
        }
    });
    if visible {
        super::surface::draw_rotated_text(
            canvas,
            text,
            x,
            y,
            pivot_x,
            pivot_y,
            policy.rotate.unwrap_or(degrees),
            layout_size as f64,
            color,
            weight,
        );
    }
}

fn padded_hit_bounds(bounds: LabelRect) -> LabelRect {
    // Text glyph metrics are tighter than a usable touch target. Keep layout and
    // overlap geometry exact, but allow a small margin around draggable labels.
    const PADDING: f32 = 6.0;
    LabelRect {
        x: bounds.x - PADDING,
        y: bounds.y - PADDING,
        width: bounds.width + PADDING * 2.0,
        height: bounds.height + PADDING * 2.0,
    }
}

fn apply_layout(
    policy: &LabelLayoutOptions,
    text: &str,
    mut x: f32,
    mut baseline: f32,
    mut size: f32,
    frame_size: (f32, f32),
) -> (f32, f32, f32, f32, f32) {
    size = policy.font_size.unwrap_or(size).max(1.0);
    if let Some(value) = policy.x.as_ref() {
        x = resolve_position(value, frame_size.0).unwrap_or(x);
    }
    if let Some(value) = policy.y.as_ref() {
        baseline = resolve_position(value, frame_size.1).unwrap_or(baseline);
    }
    x += policy.dx.unwrap_or(0.0);
    baseline += policy.dy.unwrap_or(0.0);
    let width = policy
        .width
        .unwrap_or_else(|| text.chars().count() as f32 * size * 0.56)
        .max(0.0);
    let height = policy.height.unwrap_or(size * 1.2).max(0.0);
    match policy.align.as_deref() {
        Some("center") => x -= width / 2.0,
        Some("right") => x -= width,
        _ => {}
    }
    match policy.vertical_align.as_deref() {
        Some("top") => baseline += size,
        Some("middle") => baseline += size / 2.0,
        _ => {}
    }
    (x, baseline, size, width, height)
}

#[allow(clippy::too_many_arguments)]
fn apply_callback(
    policy: &mut LabelLayoutOptions,
    series_index: usize,
    label_index: usize,
    text: &str,
    x: f32,
    baseline: f32,
    size: f32,
) {
    let Some(callback) = policy.callback.clone() else {
        return;
    };
    let width = text.chars().count() as f32 * size * 0.56;
    let rect = [x, baseline - size, width, size * 1.2];
    let result = callback(LabelLayoutCallbackParams {
        series_index,
        data_index: Some(label_index),
        text: text.to_string(),
        align: policy.align.clone().unwrap_or_else(|| String::from("left")),
        vertical_align: policy
            .vertical_align
            .clone()
            .unwrap_or_else(|| String::from("bottom")),
        rect,
        label_rect: rect,
        label_line_points: policy.label_line_points.clone(),
    });
    if let Some(value) = result.hide_overlap {
        policy.hide_overlap = value;
    }
    if let Some(value) = result.move_overlap {
        policy.move_overlap = Some(value);
    }
    if let Some(value) = result.draggable {
        policy.draggable = value;
    }
    macro_rules! replace_some {
        ($field:ident) => {
            if result.$field.is_some() {
                policy.$field = result.$field;
            }
        };
    }
    replace_some!(x);
    replace_some!(y);
    replace_some!(dx);
    replace_some!(dy);
    replace_some!(rotate);
    replace_some!(align);
    replace_some!(vertical_align);
    replace_some!(width);
    replace_some!(height);
    replace_some!(font_size);
    if let Some(points) = result.label_line_points {
        policy.label_line_points = points;
    }
}

fn resolve_position(value: &Value, extent: f32) -> Option<f32> {
    match value {
        Value::Number(value) => value.as_f64().map(|value| value as f32),
        Value::String(value) => value
            .strip_suffix('%')
            .and_then(|value| value.trim().parse::<f32>().ok())
            .map(|percent| extent * percent / 100.0)
            .or_else(|| value.parse::<f32>().ok()),
        _ => None,
    }
}

fn overlaps(left: LabelRect, right: LabelRect) -> bool {
    left.x < right.x + right.width
        && left.x + left.width > right.x
        && left.y < right.y + right.height
        && left.y + left.height > right.y
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::rc::Rc;

    use super::*;
    use crate::model::LabelLayoutCallbackResult;

    #[test]
    fn rectangle_overlap_is_strict() {
        let left = LabelRect {
            x: 0.0,
            y: 0.0,
            width: 10.0,
            height: 10.0,
        };
        assert!(overlaps(
            left,
            LabelRect {
                x: 5.0,
                y: 5.0,
                width: 10.0,
                height: 10.0
            }
        ));
        assert!(!overlaps(
            left,
            LabelRect {
                x: 10.0,
                y: 0.0,
                width: 10.0,
                height: 10.0
            }
        ));
    }

    #[test]
    fn callback_receives_data_index_and_can_replace_guide_points() {
        let received_index = Rc::new(Cell::new(None));
        let callback_index = received_index.clone();
        let mut policy = LabelLayoutOptions::default().with_callback(move |params| {
            callback_index.set(params.data_index);
            LabelLayoutCallbackResult {
                label_line_points: Some(vec![[1.0, 2.0], [3.0, 4.0]]),
                ..LabelLayoutCallbackResult::default()
            }
        });
        apply_callback(&mut policy, 2, 7, "value", 10.0, 20.0, 12.0);
        assert_eq!(received_index.get(), Some(7));
        assert_eq!(policy.label_line_points, vec![[1.0, 2.0], [3.0, 4.0]]);
    }
}
