//! Hit-region atoms shared by every interactive series renderer.

use std::f32::consts::TAU;

use crate::model::{ChartEvent, DataPoint, DataValue};

#[derive(Debug, Clone)]
pub(super) enum HitShape {
    Point {
        x: f32,
        y: f32,
        radius: f32,
    },
    Rect {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    },
    Sector {
        cx: f32,
        cy: f32,
        inner: f32,
        outer: f32,
        start: f32,
        sweep: f32,
    },
}

#[derive(Debug, Clone)]
pub(super) struct HitRegion {
    pub(super) shape: HitShape,
    pub(super) event: ChartEvent,
}

impl HitRegion {
    pub(super) fn hit(&self, x: f32, y: f32) -> Option<f32> {
        match self.shape {
            HitShape::Point {
                x: px,
                y: py,
                radius,
            } => {
                let distance = ((x - px).powi(2) + (y - py).powi(2)).sqrt();
                (distance <= radius).then_some(distance)
            }
            HitShape::Rect {
                x: rx,
                y: ry,
                width,
                height,
            } => (x >= rx && x <= rx + width && y >= ry && y <= ry + height).then_some(0.0),
            HitShape::Sector {
                cx,
                cy,
                inner,
                outer,
                start,
                sweep,
            } => {
                let dx = x - cx;
                let dy = y - cy;
                let distance = (dx * dx + dy * dy).sqrt();
                if distance < inner || distance > outer {
                    return None;
                }
                let mut angle = dy.atan2(dx);
                if angle < 0.0 {
                    angle += TAU;
                }
                let mut local = if sweep >= 0.0 {
                    angle - start
                } else {
                    start - angle
                };
                if local < 0.0 {
                    local += TAU;
                }
                (local <= sweep.abs()).then_some((outer - distance).abs())
            }
        }
    }
}

pub(super) fn point_hit(
    component: &str,
    series_index: usize,
    data_index: usize,
    series_name: Option<String>,
    point: &DataPoint,
    center: (f32, f32),
    radius: f32,
) -> HitRegion {
    let (x, y) = center;
    HitRegion {
        shape: HitShape::Point { x, y, radius },
        event: chart_event(
            component,
            series_index,
            data_index,
            series_name,
            point,
            x,
            y,
        ),
    }
}

pub(super) fn rect_hit(
    component: &str,
    series_index: usize,
    data_index: usize,
    series_name: Option<String>,
    point: &DataPoint,
    bounds: (f32, f32, f32, f32),
) -> HitRegion {
    let (x, y, width, height) = bounds;
    HitRegion {
        shape: HitShape::Rect {
            x,
            y,
            width,
            height,
        },
        event: chart_event(
            component,
            series_index,
            data_index,
            series_name,
            point,
            x + width / 2.0,
            y + height / 2.0,
        ),
    }
}

pub(super) fn chart_event(
    component: &str,
    series_index: usize,
    data_index: usize,
    series_name: Option<String>,
    point: &DataPoint,
    x: f32,
    y: f32,
) -> ChartEvent {
    ChartEvent {
        series_index,
        data_index,
        series_name,
        name: point.name.clone(),
        value: point.values.iter().filter_map(DataValue::as_f64).collect(),
        x,
        y,
        component_type: component.to_string(),
    }
}
