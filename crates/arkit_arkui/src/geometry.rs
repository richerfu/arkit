//! Unit-bearing geometry at native/UI boundaries.

use crate::element_ref::LayoutFramePx;

/// A size measured in physical pixels.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LayoutSizePx {
    pub width: f32,
    pub height: f32,
}

impl LayoutSizePx {
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    pub fn is_measured(self) -> bool {
        self.width.is_finite() && self.height.is_finite() && self.width > 0.0 && self.height > 0.0
    }
}

impl From<LayoutFramePx> for LayoutSizePx {
    fn from(frame: LayoutFramePx) -> Self {
        Self::new(frame.width, frame.height)
    }
}

/// A size measured in ArkUI logical viewport units (vp).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LogicalSizeVp {
    pub width: f32,
    pub height: f32,
}

impl LogicalSizeVp {
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    /// Convert physical dimensions to vp.
    ///
    /// Returns `None` for a non-finite/non-positive scale, negative or
    /// non-finite dimensions, or a non-finite result.
    pub fn from_physical(size: LayoutSizePx, scale: f32) -> Option<Self> {
        if !valid_scale(scale)
            || !size.width.is_finite()
            || !size.height.is_finite()
            || size.width < 0.0
            || size.height < 0.0
        {
            return None;
        }
        let logical = Self::new(size.width / scale, size.height / scale);
        (logical.width.is_finite() && logical.height.is_finite()).then_some(logical)
    }

    pub fn is_measured(self) -> bool {
        self.width.is_finite() && self.height.is_finite() && self.width > 0.0 && self.height > 0.0
    }
}

/// A point in physical window coordinates.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WindowPxPoint {
    pub x: f32,
    pub y: f32,
}

impl WindowPxPoint {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// A point in logical vp relative to a local canvas or element.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LocalVpPoint {
    pub x: f32,
    pub y: f32,
}

impl LocalVpPoint {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Convert a physical window point to vp relative to `frame`.
    ///
    /// The window origin is subtracted before scaling. Returns `None` rather
    /// than guessing when `scale` is non-finite or not positive.
    pub fn from_window_px(point: WindowPxPoint, frame: LayoutFramePx, scale: f32) -> Option<Self> {
        if !valid_scale(scale)
            || !point.x.is_finite()
            || !point.y.is_finite()
            || !frame.x.is_finite()
            || !frame.y.is_finite()
        {
            return None;
        }
        let local = Self::new((point.x - frame.x) / scale, (point.y - frame.y) / scale);
        (local.x.is_finite() && local.y.is_finite()).then_some(local)
    }
}

fn valid_scale(scale: f32) -> bool {
    scale.is_finite() && scale > 0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dimensions_divide_without_applying_the_frame_origin() {
        assert_eq!(
            LogicalSizeVp::from_physical(LayoutSizePx::new(350.0, 175.0), 3.5),
            Some(LogicalSizeVp::new(100.0, 50.0))
        );
    }

    #[test]
    fn window_point_subtracts_nonzero_origin_before_scaling() {
        let frame = LayoutFramePx {
            x: 70.0,
            y: 35.0,
            width: 350.0,
            height: 175.0,
        };
        assert_eq!(
            LocalVpPoint::from_window_px(WindowPxPoint::new(105.0, 105.0), frame, 3.5),
            Some(LocalVpPoint::new(10.0, 20.0))
        );
    }

    #[test]
    fn invalid_scales_are_rejected_explicitly() {
        let size = LayoutSizePx::new(10.0, 20.0);
        assert_eq!(LogicalSizeVp::from_physical(size, 0.0), None);
        assert_eq!(LogicalSizeVp::from_physical(size, f32::NAN), None);
        assert_eq!(
            LocalVpPoint::from_window_px(
                WindowPxPoint::new(1.0, 2.0),
                LayoutFramePx::default(),
                -1.0,
            ),
            None
        );
    }

    #[test]
    fn non_finite_inputs_and_conversion_overflow_are_rejected() {
        assert!(!LayoutSizePx::new(f32::INFINITY, 1.0).is_measured());
        assert!(!LogicalSizeVp::new(1.0, f32::NAN).is_measured());
        assert_eq!(
            LogicalSizeVp::from_physical(LayoutSizePx::new(f32::MAX, 1.0), f32::MIN_POSITIVE),
            None
        );
        assert_eq!(
            LocalVpPoint::from_window_px(
                WindowPxPoint::new(f32::INFINITY, 2.0),
                LayoutFramePx::default(),
                1.0,
            ),
            None
        );
    }
}
