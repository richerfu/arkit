//! Logical shadow lengths converted at the native paint boundary.

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PaintScale(f32);

impl Default for PaintScale {
    fn default() -> Self {
        Self(1.0)
    }
}

impl PaintScale {
    pub(crate) fn new(scale: f32) -> Self {
        if scale.is_finite() && scale > 0.0 {
            Self(scale)
        } else {
            Self::default()
        }
    }

    pub(crate) fn physical(self, logical: f32) -> f32 {
        logical * self.0
    }
}

#[cfg(test)]
mod tests {
    use super::PaintScale;

    #[test]
    fn logical_shadow_lengths_have_equal_size_across_densities() {
        for scale in [1.0, 1.5, 3.25] {
            let paint = PaintScale::new(scale);
            assert_eq!(paint.physical(4.0) / scale, 4.0);
            assert_eq!(paint.physical(6.0) / scale, 6.0);
        }
    }

    #[test]
    fn invalid_density_does_not_poison_native_values() {
        for invalid in [0.0, -1.0, f32::NAN, f32::INFINITY] {
            assert_eq!(PaintScale::new(invalid).physical(6.0), 6.0);
        }
    }
}
