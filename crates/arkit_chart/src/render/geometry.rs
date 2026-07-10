//! Plot rectangles, palette lookup, and coordinate scaling atoms.

use std::f32::consts::TAU;

use crate::model::{ChartOption, DEFAULT_COLORS};

#[derive(Debug, Clone, Copy)]
pub(super) struct Plot {
    pub(super) x: f32,
    pub(super) y: f32,
    pub(super) width: f32,
    pub(super) height: f32,
}

pub(super) fn effective_palette(option: &ChartOption) -> Vec<u32> {
    if option.visual_style.palette.is_empty() {
        DEFAULT_COLORS.to_vec()
    } else {
        option.visual_style.palette.clone()
    }
}

pub(super) fn color(palette: &[u32], index: usize) -> u32 {
    palette[index % palette.len().max(1)]
}

pub(super) fn normalize_angle(angle: f32) -> f32 {
    let mut angle = angle % TAU;
    if angle < 0.0 {
        angle += TAU;
    }
    angle
}

/// Sample a Catmull-Rom spline through a polyline. The native drawing binding
/// currently exposes line and arc path verbs, so sampling keeps smoothing as a
/// reusable geometry concern instead of embedding it in the line renderer.
pub(super) fn smooth_polyline(points: &[(f32, f32)], smooth: f32) -> Vec<(f32, f32)> {
    if points.len() < 3 || smooth <= 0.0 {
        return points.to_vec();
    }
    let samples = (4.0 + smooth.clamp(0.0, 1.0) * 8.0).round() as usize;
    let mut output = Vec::with_capacity((points.len() - 1) * samples + 1);
    output.push(points[0]);
    for index in 0..points.len() - 1 {
        let p0 = points[index.saturating_sub(1)];
        let p1 = points[index];
        let p2 = points[index + 1];
        let p3 = points[(index + 2).min(points.len() - 1)];
        for sample in 1..=samples {
            let t = sample as f32 / samples as f32;
            let t2 = t * t;
            let t3 = t2 * t;
            let tension = smooth.clamp(0.0, 1.0) * 0.5;
            let h00 = 2.0 * t3 - 3.0 * t2 + 1.0;
            let h10 = t3 - 2.0 * t2 + t;
            let h01 = -2.0 * t3 + 3.0 * t2;
            let h11 = t3 - t2;
            let m1 = ((p2.0 - p0.0) * tension, (p2.1 - p0.1) * tension);
            let m2 = ((p3.0 - p1.0) * tension, (p3.1 - p1.1) * tension);
            let x = h00 * p1.0 + h10 * m1.0 + h01 * p2.0 + h11 * m2.0;
            let y = h00 * p1.1 + h10 * m1.1 + h01 * p2.1 + h11 * m2.1;
            output.push((x, y));
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoothing_preserves_polyline_endpoints() {
        let points = [(0.0, 1.0), (1.0, 3.0), (2.0, 2.0)];
        let smoothed = smooth_polyline(&points, 0.5);
        assert_eq!(smoothed.first(), Some(&points[0]));
        assert_eq!(smoothed.last(), Some(&points[2]));
        assert!(smoothed.len() > points.len());
    }
}
