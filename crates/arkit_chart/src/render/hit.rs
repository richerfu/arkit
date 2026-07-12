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
    MultiPolygon {
        polygons: Vec<HitPolygon>,
    },
}

#[derive(Debug, Clone)]
pub(super) struct HitPolygon {
    pub(super) exterior: Vec<(f32, f32)>,
    pub(super) holes: Vec<Vec<(f32, f32)>>,
}

#[derive(Debug, Clone)]
pub(crate) struct HitRegion {
    pub(super) shape: HitShape,
    pub(crate) event: ChartEvent,
}

impl HitRegion {
    pub(crate) fn hit(&self, x: f32, y: f32) -> Option<f32> {
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
            HitShape::MultiPolygon { ref polygons } => polygons
                .iter()
                .any(|polygon| {
                    point_in_ring((x, y), &polygon.exterior)
                        && !polygon.holes.iter().any(|hole| point_in_ring((x, y), hole))
                })
                .then_some(0.0),
        }
    }
}

pub(super) fn polygon_hit(
    component: &str,
    series_index: usize,
    data_index: usize,
    series_name: Option<String>,
    point: &DataPoint,
    center: (f32, f32),
    polygons: Vec<HitPolygon>,
) -> HitRegion {
    HitRegion {
        shape: HitShape::MultiPolygon { polygons },
        event: chart_event(
            component,
            series_index,
            data_index,
            series_name,
            point,
            center.0,
            center.1,
        ),
    }
}

fn point_in_ring(point: (f32, f32), ring: &[(f32, f32)]) -> bool {
    if ring.len() < 3 {
        return false;
    }
    let mut inside = false;
    let mut previous = ring.len() - 1;
    for current in 0..ring.len() {
        let (xi, yi) = ring[current];
        let (xj, yj) = ring[previous];
        if point_on_segment(point, (xi, yi), (xj, yj)) {
            return true;
        }
        let crosses = (yi > point.1) != (yj > point.1)
            && point.0 < (xj - xi) * (point.1 - yi) / (yj - yi) + xi;
        if crosses {
            inside = !inside;
        }
        previous = current;
    }
    inside
}

fn point_on_segment(point: (f32, f32), start: (f32, f32), end: (f32, f32)) -> bool {
    let cross = (point.1 - start.1) * (end.0 - start.0) - (point.0 - start.0) * (end.1 - start.1);
    if cross.abs() > 1e-4 {
        return false;
    }
    point.0 >= start.0.min(end.0) - 1e-4
        && point.0 <= start.0.max(end.0) + 1e-4
        && point.1 >= start.1.min(end.1) - 1e-4
        && point.1 <= start.1.max(end.1) + 1e-4
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn polygon_hit_excludes_holes_and_bounding_box_gaps() {
        let polygon = HitPolygon {
            exterior: vec![(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)],
            holes: vec![vec![(3.0, 3.0), (7.0, 3.0), (7.0, 7.0), (3.0, 7.0)]],
        };
        let point = DataPoint::named("region", 1.0);
        let hit = polygon_hit("map", 0, 0, None, &point, (5.0, 5.0), vec![polygon]);
        assert!(hit.hit(1.0, 1.0).is_some());
        assert!(hit.hit(5.0, 5.0).is_none());
        assert!(hit.hit(11.0, 5.0).is_none());
    }
}
