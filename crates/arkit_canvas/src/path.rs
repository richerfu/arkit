use std::f32::consts::{FRAC_PI_2, TAU};

use ohos_drawing_binding::Path;

use crate::{CanvasError, CanvasResult, DomMatrix2D};

const ARC_EPSILON: f32 = 1.0e-6;

/// One x/y radius pair accepted by [`Path2D::round_rect`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CanvasRadius {
    pub x: f32,
    pub y: f32,
}

impl CanvasRadius {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

impl From<f32> for CanvasRadius {
    fn from(value: f32) -> Self {
        Self::new(value, value)
    }
}

/// Conversion for the one-to-four radii syntax used by `roundRect()`.
pub trait IntoCanvasRadii {
    fn into_canvas_radii(self) -> Vec<CanvasRadius>;
}

impl IntoCanvasRadii for f32 {
    fn into_canvas_radii(self) -> Vec<CanvasRadius> {
        vec![self.into()]
    }
}

impl IntoCanvasRadii for CanvasRadius {
    fn into_canvas_radii(self) -> Vec<CanvasRadius> {
        vec![self]
    }
}

impl<const N: usize> IntoCanvasRadii for [f32; N] {
    fn into_canvas_radii(self) -> Vec<CanvasRadius> {
        self.into_iter().map(CanvasRadius::from).collect()
    }
}

impl<const N: usize> IntoCanvasRadii for [CanvasRadius; N] {
    fn into_canvas_radii(self) -> Vec<CanvasRadius> {
        self.into_iter().collect()
    }
}

impl IntoCanvasRadii for &[f32] {
    fn into_canvas_radii(self) -> Vec<CanvasRadius> {
        self.iter().copied().map(CanvasRadius::from).collect()
    }
}

impl IntoCanvasRadii for &[CanvasRadius] {
    fn into_canvas_radii(self) -> Vec<CanvasRadius> {
        self.to_vec()
    }
}

/// A reusable Canvas 2D path, corresponding to the web platform's `Path2D`.
#[derive(Debug)]
pub struct Path2D {
    pub(crate) inner: Path,
    current_point: Option<(f32, f32)>,
    subpath_start: Option<(f32, f32)>,
}

impl Path2D {
    pub fn new() -> Self {
        Self {
            inner: Path::new(),
            current_point: None,
            subpath_start: None,
        }
    }

    pub fn from_path(path: &Self) -> Self {
        path.clone()
    }

    pub fn from_svg(path: &str) -> CanvasResult<Self> {
        if path.contains('\0') {
            return Err(CanvasError::InvalidSvgPath);
        }
        let mut result = Self::new();
        if result.inner.build_from_svg(path) {
            Ok(result)
        } else {
            Err(CanvasError::InvalidSvgPath)
        }
    }

    pub fn add_path(&mut self, path: &Self, transform: Option<DomMatrix2D>) {
        if transform.is_some_and(|matrix| !matrix.is_finite()) {
            return;
        }
        let matrix = transform.map(DomMatrix2D::to_native_matrix);
        self.inner.add_path(&path.inner, matrix.as_ref());
        self.current_point = None;
        self.subpath_start = None;
    }

    pub fn move_to(&mut self, x: f32, y: f32) {
        self.move_to_transformed(x, y, DomMatrix2D::IDENTITY);
    }

    pub fn line_to(&mut self, x: f32, y: f32) {
        self.line_to_transformed(x, y, DomMatrix2D::IDENTITY);
    }

    pub fn close_path(&mut self) {
        if self.current_point.is_some() {
            self.inner.close();
            self.current_point = self.subpath_start;
        }
    }

    pub fn quadratic_curve_to(&mut self, control_x: f32, control_y: f32, x: f32, y: f32) {
        self.quadratic_curve_to_transformed(control_x, control_y, x, y, DomMatrix2D::IDENTITY);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn bezier_curve_to(
        &mut self,
        control_x1: f32,
        control_y1: f32,
        control_x2: f32,
        control_y2: f32,
        x: f32,
        y: f32,
    ) {
        self.bezier_curve_to_transformed(
            control_x1,
            control_y1,
            control_x2,
            control_y2,
            x,
            y,
            DomMatrix2D::IDENTITY,
        );
    }

    pub fn arc_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, radius: f32) -> CanvasResult<()> {
        self.arc_to_transformed(x1, y1, x2, y2, radius, DomMatrix2D::IDENTITY)
    }

    pub fn rect(&mut self, x: f32, y: f32, width: f32, height: f32) {
        self.rect_transformed(x, y, width, height, DomMatrix2D::IDENTITY);
    }

    pub fn round_rect(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        radii: impl IntoCanvasRadii,
    ) -> CanvasResult<()> {
        self.round_rect_transformed(
            x,
            y,
            width,
            height,
            radii.into_canvas_radii(),
            DomMatrix2D::IDENTITY,
        )
    }

    pub fn arc(
        &mut self,
        x: f32,
        y: f32,
        radius: f32,
        start_angle: f32,
        end_angle: f32,
        counterclockwise: bool,
    ) -> CanvasResult<()> {
        self.ellipse(
            x,
            y,
            radius,
            radius,
            0.0,
            start_angle,
            end_angle,
            counterclockwise,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn ellipse(
        &mut self,
        x: f32,
        y: f32,
        radius_x: f32,
        radius_y: f32,
        rotation: f32,
        start_angle: f32,
        end_angle: f32,
        counterclockwise: bool,
    ) -> CanvasResult<()> {
        self.ellipse_transformed(
            x,
            y,
            radius_x,
            radius_y,
            rotation,
            start_angle,
            end_angle,
            counterclockwise,
            DomMatrix2D::IDENTITY,
        )
    }

    pub(crate) fn move_to_transformed(&mut self, x: f32, y: f32, transform: DomMatrix2D) {
        if !Self::finite([x, y]) || !transform.is_finite() {
            return;
        }
        let point = transform.transform_point(x, y);
        self.inner.move_to(point.0, point.1);
        self.current_point = Some(point);
        self.subpath_start = Some(point);
    }

    pub(crate) fn line_to_transformed(&mut self, x: f32, y: f32, transform: DomMatrix2D) {
        if !Self::finite([x, y]) || !transform.is_finite() {
            return;
        }
        let point = transform.transform_point(x, y);
        if self.current_point.is_none() {
            self.inner.move_to(point.0, point.1);
            self.subpath_start = Some(point);
        } else {
            self.inner.line_to(point.0, point.1);
        }
        self.current_point = Some(point);
    }

    pub(crate) fn quadratic_curve_to_transformed(
        &mut self,
        control_x: f32,
        control_y: f32,
        x: f32,
        y: f32,
        transform: DomMatrix2D,
    ) {
        if !Self::finite([control_x, control_y, x, y]) || !transform.is_finite() {
            return;
        }
        let control = transform.transform_point(control_x, control_y);
        let end = transform.transform_point(x, y);
        if self.current_point.is_none() {
            self.inner.move_to(control.0, control.1);
            self.current_point = Some(control);
            self.subpath_start = Some(control);
        }
        self.inner
            .quadratic_curve_to(control.0, control.1, end.0, end.1);
        self.current_point = Some(end);
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn bezier_curve_to_transformed(
        &mut self,
        control_x1: f32,
        control_y1: f32,
        control_x2: f32,
        control_y2: f32,
        x: f32,
        y: f32,
        transform: DomMatrix2D,
    ) {
        if !Self::finite([control_x1, control_y1, control_x2, control_y2, x, y])
            || !transform.is_finite()
        {
            return;
        }
        let control1 = transform.transform_point(control_x1, control_y1);
        let control2 = transform.transform_point(control_x2, control_y2);
        let end = transform.transform_point(x, y);
        if self.current_point.is_none() {
            self.inner.move_to(control1.0, control1.1);
            self.current_point = Some(control1);
            self.subpath_start = Some(control1);
        }
        self.inner
            .bezier_curve_to(control1.0, control1.1, control2.0, control2.1, end.0, end.1);
        self.current_point = Some(end);
    }

    pub(crate) fn rect_transformed(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        transform: DomMatrix2D,
    ) {
        if !Self::finite([x, y, width, height]) || !transform.is_finite() {
            return;
        }
        self.move_to_transformed(x, y, transform);
        self.line_to_transformed(x + width, y, transform);
        self.line_to_transformed(x + width, y + height, transform);
        self.line_to_transformed(x, y + height, transform);
        self.close_path();
    }

    pub(crate) fn arc_to_transformed(
        &mut self,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        radius: f32,
        transform: DomMatrix2D,
    ) -> CanvasResult<()> {
        if radius < 0.0 {
            return Err(CanvasError::NegativeRadius);
        }
        if !Self::finite([x1, y1, x2, y2, radius]) || !transform.is_finite() {
            return Ok(());
        }
        let Some(current_world) = self.current_point else {
            self.move_to_transformed(x1, y1, transform);
            return Ok(());
        };
        let Some(inverse) = transform.inverse() else {
            return Ok(());
        };
        let p0 = inverse.transform_point(current_world.0, current_world.1);
        let p1 = (x1, y1);
        let p2 = (x2, y2);
        let v1 = (p0.0 - p1.0, p0.1 - p1.1);
        let v2 = (p2.0 - p1.0, p2.1 - p1.1);
        let length1 = v1.0.hypot(v1.1);
        let length2 = v2.0.hypot(v2.1);
        if radius == 0.0 || length1 <= ARC_EPSILON || length2 <= ARC_EPSILON {
            self.line_to_transformed(x1, y1, transform);
            return Ok(());
        }
        let v1 = (v1.0 / length1, v1.1 / length1);
        let v2 = (v2.0 / length2, v2.1 / length2);
        let dot = (v1.0 * v2.0 + v1.1 * v2.1).clamp(-1.0, 1.0);
        let cross = v1.0 * v2.1 - v1.1 * v2.0;
        if cross.abs() <= ARC_EPSILON || (1.0 - dot.abs()) <= ARC_EPSILON {
            self.line_to_transformed(x1, y1, transform);
            return Ok(());
        }
        let distance = radius / (dot.acos() * 0.5).tan();
        let tangent1 = (p1.0 + v1.0 * distance, p1.1 + v1.1 * distance);
        let tangent2 = (p1.0 + v2.0 * distance, p1.1 + v2.1 * distance);
        let normal = if cross < 0.0 {
            (v1.1, -v1.0)
        } else {
            (-v1.1, v1.0)
        };
        let center = (
            tangent1.0 + normal.0 * radius,
            tangent1.1 + normal.1 * radius,
        );
        self.line_to_transformed(tangent1.0, tangent1.1, transform);
        self.ellipse_transformed(
            center.0,
            center.1,
            radius,
            radius,
            0.0,
            (tangent1.1 - center.1).atan2(tangent1.0 - center.0),
            (tangent2.1 - center.1).atan2(tangent2.0 - center.0),
            cross > 0.0,
            transform,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn ellipse_transformed(
        &mut self,
        x: f32,
        y: f32,
        radius_x: f32,
        radius_y: f32,
        rotation: f32,
        start_angle: f32,
        end_angle: f32,
        counterclockwise: bool,
        transform: DomMatrix2D,
    ) -> CanvasResult<()> {
        if radius_x < 0.0 || radius_y < 0.0 {
            return Err(CanvasError::NegativeRadius);
        }
        if !Self::finite([x, y, radius_x, radius_y, rotation, start_angle, end_angle])
            || !transform.is_finite()
        {
            return Ok(());
        }
        let sweep = Self::canvas_arc_sweep(start_angle, end_angle, counterclockwise);
        let start = Self::ellipse_point(x, y, radius_x, radius_y, rotation, start_angle);
        if self.current_point.is_none() {
            self.move_to_transformed(start.0, start.1, transform);
        } else {
            self.line_to_transformed(start.0, start.1, transform);
        }
        if radius_x == 0.0 || radius_y == 0.0 || sweep.abs() <= ARC_EPSILON {
            return Ok(());
        }
        let segment_count = (sweep.abs() / FRAC_PI_2).ceil().max(1.0) as usize;
        let delta = sweep / segment_count as f32;
        let mut angle = start_angle;
        for _ in 0..segment_count {
            let next = angle + delta;
            let start = Self::ellipse_point(x, y, radius_x, radius_y, rotation, angle);
            let end = Self::ellipse_point(x, y, radius_x, radius_y, rotation, next);
            let start_tangent = Self::ellipse_tangent(radius_x, radius_y, rotation, angle);
            let end_tangent = Self::ellipse_tangent(radius_x, radius_y, rotation, next);
            let factor = (4.0 / 3.0) * (delta * 0.25).tan();
            self.bezier_curve_to_transformed(
                start.0 + start_tangent.0 * factor,
                start.1 + start_tangent.1 * factor,
                end.0 - end_tangent.0 * factor,
                end.1 - end_tangent.1 * factor,
                end.0,
                end.1,
                transform,
            );
            angle = next;
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn round_rect_transformed(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        radii: Vec<CanvasRadius>,
        transform: DomMatrix2D,
    ) -> CanvasResult<()> {
        if !Self::finite([x, y, width, height]) || !transform.is_finite() {
            return Ok(());
        }
        let mut corners = Self::expand_radii(&radii)?;
        if corners
            .iter()
            .any(|radius| !Self::finite([radius.x, radius.y]) || radius.x < 0.0 || radius.y < 0.0)
        {
            return Err(CanvasError::NegativeRadius);
        }
        let mut left = x;
        let mut right = x + width;
        let mut top = y;
        let mut bottom = y + height;
        if width < 0.0 {
            std::mem::swap(&mut left, &mut right);
            corners.swap(0, 1);
            corners.swap(3, 2);
        }
        if height < 0.0 {
            std::mem::swap(&mut top, &mut bottom);
            corners.swap(0, 3);
            corners.swap(1, 2);
        }
        Self::normalize_radii(&mut corners, right - left, bottom - top);
        let [top_left, top_right, bottom_right, bottom_left] = corners;
        self.move_to_transformed(left + top_left.x, top, transform);
        self.line_to_transformed(right - top_right.x, top, transform);
        self.corner_curve(
            right - top_right.x,
            top,
            right,
            top + top_right.y,
            right,
            top,
            top_right,
            transform,
        );
        self.line_to_transformed(right, bottom - bottom_right.y, transform);
        self.corner_curve(
            right,
            bottom - bottom_right.y,
            right - bottom_right.x,
            bottom,
            right,
            bottom,
            bottom_right,
            transform,
        );
        self.line_to_transformed(left + bottom_left.x, bottom, transform);
        self.corner_curve(
            left + bottom_left.x,
            bottom,
            left,
            bottom - bottom_left.y,
            left,
            bottom,
            bottom_left,
            transform,
        );
        self.line_to_transformed(left, top + top_left.y, transform);
        self.corner_curve(
            left,
            top + top_left.y,
            left + top_left.x,
            top,
            left,
            top,
            top_left,
            transform,
        );
        self.close_path();
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn corner_curve(
        &mut self,
        start_x: f32,
        start_y: f32,
        end_x: f32,
        end_y: f32,
        corner_x: f32,
        corner_y: f32,
        radius: CanvasRadius,
        transform: DomMatrix2D,
    ) {
        if radius.x == 0.0 || radius.y == 0.0 {
            self.line_to_transformed(end_x, end_y, transform);
            return;
        }
        const KAPPA: f32 = 0.552_284_8;
        let control1 = (
            start_x + (corner_x - start_x) * KAPPA,
            start_y + (corner_y - start_y) * KAPPA,
        );
        let control2 = (
            end_x + (corner_x - end_x) * KAPPA,
            end_y + (corner_y - end_y) * KAPPA,
        );
        self.bezier_curve_to_transformed(
            control1.0, control1.1, control2.0, control2.1, end_x, end_y, transform,
        );
    }

    pub(crate) fn transformed(&self, transform: DomMatrix2D) -> Path {
        let mut path = self.inner.clone_path();
        path.transform(&transform.to_native_matrix());
        path
    }

    pub(crate) fn clone_with_fill_rule(&self, rule: crate::FillRule) -> Path {
        let mut path = self.inner.clone_path();
        path.set_fill_type(rule.to_native_fill_type());
        path
    }

    pub(crate) fn transformed_with_fill_rule(
        &self,
        transform: DomMatrix2D,
        rule: crate::FillRule,
    ) -> Path {
        let mut path = self.transformed(transform);
        path.set_fill_type(rule.to_native_fill_type());
        path
    }

    pub(crate) fn contains(&self, x: f32, y: f32, rule: crate::FillRule) -> bool {
        self.clone_with_fill_rule(rule).contains(x, y)
    }

    pub(crate) fn reset(&mut self) {
        self.inner.reset();
        self.current_point = None;
        self.subpath_start = None;
    }

    fn canvas_arc_sweep(start: f32, end: f32, counterclockwise: bool) -> f32 {
        let raw = end - start;
        if counterclockwise {
            if -raw >= TAU {
                return -TAU;
            }
            let sweep = -(-raw).rem_euclid(TAU);
            if sweep.abs() <= ARC_EPSILON && raw.abs() > ARC_EPSILON {
                -TAU
            } else {
                sweep
            }
        } else {
            if raw >= TAU {
                return TAU;
            }
            let sweep = raw.rem_euclid(TAU);
            if sweep.abs() <= ARC_EPSILON && raw.abs() > ARC_EPSILON {
                TAU
            } else {
                sweep
            }
        }
    }

    fn ellipse_point(
        x: f32,
        y: f32,
        radius_x: f32,
        radius_y: f32,
        rotation: f32,
        angle: f32,
    ) -> (f32, f32) {
        let (sin_rotation, cos_rotation) = rotation.sin_cos();
        let (sin_angle, cos_angle) = angle.sin_cos();
        (
            x + cos_rotation * radius_x * cos_angle - sin_rotation * radius_y * sin_angle,
            y + sin_rotation * radius_x * cos_angle + cos_rotation * radius_y * sin_angle,
        )
    }

    fn ellipse_tangent(radius_x: f32, radius_y: f32, rotation: f32, angle: f32) -> (f32, f32) {
        let (sin_rotation, cos_rotation) = rotation.sin_cos();
        let (sin_angle, cos_angle) = angle.sin_cos();
        (
            -cos_rotation * radius_x * sin_angle - sin_rotation * radius_y * cos_angle,
            -sin_rotation * radius_x * sin_angle + cos_rotation * radius_y * cos_angle,
        )
    }

    fn expand_radii(radii: &[CanvasRadius]) -> CanvasResult<[CanvasRadius; 4]> {
        match radii {
            [] => Err(CanvasError::InvalidRadiiCount),
            [all] => Ok([*all; 4]),
            [first, second] => Ok([*first, *second, *first, *second]),
            [first, second, third] => Ok([*first, *second, *third, *second]),
            [top_left, top_right, bottom_right, bottom_left] => {
                Ok([*top_left, *top_right, *bottom_right, *bottom_left])
            }
            _ => Err(CanvasError::InvalidRadiiCount),
        }
    }

    fn normalize_radii(corners: &mut [CanvasRadius; 4], width: f32, height: f32) {
        let ratios = [
            Self::safe_ratio(width, corners[0].x + corners[1].x),
            Self::safe_ratio(height, corners[1].y + corners[2].y),
            Self::safe_ratio(width, corners[2].x + corners[3].x),
            Self::safe_ratio(height, corners[3].y + corners[0].y),
        ];
        let scale = ratios.into_iter().fold(1.0_f32, f32::min).min(1.0);
        if scale < 1.0 {
            for corner in corners {
                corner.x *= scale;
                corner.y *= scale;
            }
        }
    }

    fn safe_ratio(numerator: f32, denominator: f32) -> f32 {
        if denominator <= 0.0 {
            1.0
        } else {
            numerator / denominator
        }
    }

    fn finite<const N: usize>(values: [f32; N]) -> bool {
        values.into_iter().all(f32::is_finite)
    }
}

impl Clone for Path2D {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone_path(),
            current_point: self.current_point,
            subpath_start: self.subpath_start,
        }
    }
}

impl Default for Path2D {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn clockwise_arc_wraps_forward() {
        let sweep = Path2D::canvas_arc_sweep(PI * 1.5, PI * 0.5, false);
        assert!((sweep - PI).abs() < 0.0001);
    }

    #[test]
    fn counterclockwise_arc_wraps_backward() {
        let sweep = Path2D::canvas_arc_sweep(PI * 0.5, PI * 1.5, true);
        assert!((sweep + PI).abs() < 0.0001);
    }

    #[test]
    fn css_round_rect_radii_expand() {
        let expanded = Path2D::expand_radii(&[1.0.into(), 2.0.into(), 3.0.into()]).unwrap();
        assert_eq!(expanded.map(|radius| radius.x), [1.0, 2.0, 3.0, 2.0]);
    }
}
