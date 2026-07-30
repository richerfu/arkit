//! Terminal surface geometry.
//!
//! ArkUI layout attributes and pointer-local coordinates are expressed in vp,
//! while `onarea` and Ghostty cell metrics are physical pixels. Keeping both
//! units in one value prevents scroll/mouse hit testing and VT size reports
//! from drifting away from the painted grid.

const PREFERRED_CELL_WIDTH_VP: f64 = 10.0;
const PREFERRED_CELL_HEIGHT_VP: f64 = 20.0;
const SURFACE_PADDING_VP: f64 = 4.0;
const MIN_SURFACE_EXTENT_VP: f64 = 1.0;

/// Approximate advance of the system Drawing `monospace` face in em.
///
/// Font size is selected from the cell width using this ratio, then any
/// height-constrained remainder is compensated with letter spacing. The
/// resulting text advance and cursor cell width therefore share one metric.
const MONOSPACE_ADVANCE_EM: f64 = 0.60;
const MAX_FONT_HEIGHT_RATIO: f64 = 0.82;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct TerminalSurfaceMetrics {
    pub(crate) cell_width_vp: f64,
    pub(crate) cell_height_vp: f64,
    pub(crate) cell_width_px: f32,
    pub(crate) cell_height_px: f32,
    pub(crate) font_size_fp: f64,
    pub(crate) letter_spacing_fp: f64,
    pub(crate) padding_vp: f64,
    scale: f64,
}

impl TerminalSurfaceMetrics {
    pub(crate) fn fallback(scale: f64) -> Self {
        Self::from_cell_vp(
            PREFERRED_CELL_WIDTH_VP,
            PREFERRED_CELL_HEIGHT_VP,
            normalized_scale(scale),
        )
    }

    /// Fit a fixed terminal grid into a physical-pixel ArkUI surface.
    ///
    /// The aspect ratio stays fixed so glyphs and all cursor styles use the
    /// same cell box. No minimum cell width is imposed: an 80-column terminal
    /// on a phone must shrink rather than silently clip columns.
    pub(crate) fn fit(
        surface_width_px: f64,
        surface_height_px: f64,
        scale: f64,
        cols: u16,
        rows: u16,
    ) -> Self {
        let scale = normalized_scale(scale);
        let width_vp = finite_positive(surface_width_px / scale);
        let height_vp = finite_positive(surface_height_px / scale);
        let inner_width = (width_vp - SURFACE_PADDING_VP * 2.0).max(MIN_SURFACE_EXTENT_VP);
        let inner_height = (height_vp - SURFACE_PADDING_VP * 2.0).max(MIN_SURFACE_EXTENT_VP);
        let cols = f64::from(cols.max(1));
        let rows = f64::from(rows.max(1));
        let aspect = PREFERRED_CELL_HEIGHT_VP / PREFERRED_CELL_WIDTH_VP;

        let cell_width_vp = (inner_width / cols)
            .min(inner_height / rows / aspect)
            .clamp(f64::EPSILON, PREFERRED_CELL_WIDTH_VP);
        let cell_height_vp = (cell_width_vp * aspect).min(PREFERRED_CELL_HEIGHT_VP);

        Self::from_cell_vp(cell_width_vp, cell_height_vp, scale)
    }

    #[cfg(test)]
    pub(crate) fn grid_width_vp(self, cols: u16) -> f64 {
        self.cell_width_vp * f64::from(cols)
    }

    #[cfg(test)]
    pub(crate) fn grid_height_vp(self, rows: u16) -> f64 {
        self.cell_height_vp * f64::from(rows)
    }

    pub(crate) fn scroll_slop_vp(self) -> f32 {
        (self.cell_height_vp as f32 * 0.4).max(6.0)
    }

    pub(crate) fn content_position_px_from_vp(self, x: f32, y: f32) -> (f32, f32) {
        (
            ((x - self.padding_vp as f32).max(0.0) * self.scale as f32),
            ((y - self.padding_vp as f32).max(0.0) * self.scale as f32),
        )
    }

    pub(crate) fn native_cell_width_px(self) -> u32 {
        self.cell_width_px.round().max(1.0) as u32
    }

    pub(crate) fn native_cell_height_px(self) -> u32 {
        self.cell_height_px.round().max(1.0) as u32
    }

    pub(crate) fn scale(self) -> f64 {
        self.scale
    }

    pub(crate) fn differs_from(self, other: Self) -> bool {
        (self.cell_width_vp - other.cell_width_vp).abs() >= 0.01
            || (self.cell_height_vp - other.cell_height_vp).abs() >= 0.01
            || (self.scale - other.scale).abs() >= 0.001
    }

    fn from_cell_vp(cell_width_vp: f64, cell_height_vp: f64, scale: f64) -> Self {
        let font_size_fp = (cell_width_vp / MONOSPACE_ADVANCE_EM)
            .min(cell_height_vp * MAX_FONT_HEIGHT_RATIO)
            .max(f64::EPSILON);
        let letter_spacing_fp = (cell_width_vp - font_size_fp * MONOSPACE_ADVANCE_EM).max(0.0);

        Self {
            cell_width_vp,
            cell_height_vp,
            cell_width_px: (cell_width_vp * scale) as f32,
            cell_height_px: (cell_height_vp * scale) as f32,
            font_size_fp,
            letter_spacing_fp,
            padding_vp: SURFACE_PADDING_VP,
            scale,
        }
    }
}

fn normalized_scale(scale: f64) -> f64 {
    if scale.is_finite() && scale > 0.0 {
        scale
    } else {
        1.0
    }
}

fn finite_positive(value: f64) -> f64 {
    if value.is_finite() && value > 0.0 {
        value
    } else {
        MIN_SURFACE_EXTENT_VP
    }
}

#[cfg(test)]
mod tests {
    use super::{TerminalSurfaceMetrics, MONOSPACE_ADVANCE_EM};

    #[test]
    fn fixed_grid_never_overflows_measured_surface() {
        let metrics = TerminalSurfaceMetrics::fit(1_260.0, 1_470.0, 3.5, 80, 24);
        let inner_width_vp = 1_260.0 / 3.5 - metrics.padding_vp * 2.0;
        let inner_height_vp = 1_470.0 / 3.5 - metrics.padding_vp * 2.0;

        assert!(metrics.grid_width_vp(80) <= inner_width_vp + 0.001);
        assert!(metrics.grid_height_vp(24) <= inner_height_vp + 0.001);
    }

    #[test]
    fn native_metrics_and_pointer_coordinates_use_their_documented_units() {
        let metrics = TerminalSurfaceMetrics::fit(1_260.0, 1_470.0, 3.5, 40, 16);
        assert!((metrics.cell_width_px - metrics.cell_width_vp as f32 * 3.5).abs() < 0.001);

        let (x, y) = metrics.content_position_px_from_vp(
            metrics.padding_vp as f32 + 10.0,
            metrics.padding_vp as f32 + 20.0,
        );
        assert_eq!((x, y), (35.0, 70.0));
    }

    #[test]
    fn font_advance_matches_cursor_cell_width() {
        let metrics = TerminalSurfaceMetrics::fallback(3.5);
        let advance = metrics.font_size_fp * MONOSPACE_ADVANCE_EM + metrics.letter_spacing_fp;
        assert!((advance - metrics.cell_width_vp).abs() < 0.001);
    }

    #[test]
    fn standard_demo_grid_fills_the_phone_surface_height() {
        let metrics = TerminalSurfaceMetrics::fit(1_156.0, 1_365.0, 3.5, 40, 24);
        let used_height_px =
            (metrics.grid_height_vp(24) + metrics.padding_vp * 2.0) * metrics.scale();

        assert!((used_height_px - 1_365.0).abs() <= 1.0);
    }
}
