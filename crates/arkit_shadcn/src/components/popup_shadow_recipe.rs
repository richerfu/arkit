//! Tailwind md/lg shadow layer geometry for the popup surface.

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum PopupShadowKind {
    Md,
    Lg,
}

#[derive(Clone, Copy)]
pub(super) struct ShadowLayer {
    pub(super) offset_y: f32,
    pub(super) blur: f32,
    pub(super) inset: f32,
}

pub(super) struct ShadowBounds {
    pub(super) width: f32,
    pub(super) height: f32,
    pub(super) radius: f32,
}

impl PopupShadowKind {
    pub(super) const fn layers(self) -> [ShadowLayer; 2] {
        match self {
            Self::Md => [
                ShadowLayer {
                    offset_y: 4.0,
                    blur: 6.0,
                    inset: 1.0,
                },
                ShadowLayer {
                    offset_y: 2.0,
                    blur: 4.0,
                    inset: 2.0,
                },
            ],
            Self::Lg => [
                ShadowLayer {
                    offset_y: 10.0,
                    blur: 15.0,
                    inset: 3.0,
                },
                ShadowLayer {
                    offset_y: 4.0,
                    blur: 6.0,
                    inset: 4.0,
                },
            ],
        }
    }
}

impl ShadowLayer {
    pub(super) fn bounds(self, width: f32, height: f32, radius: f32) -> Option<ShadowBounds> {
        let width = width - self.inset * 2.0;
        let height = height - self.inset * 2.0;
        (width.is_finite() && height.is_finite() && width > 0.0 && height > 0.0).then_some(
            ShadowBounds {
                width,
                height,
                radius: (radius - self.inset).max(0.0),
            },
        )
    }

    pub(super) fn native_value(self, scale: f32) -> String {
        // Unlike the underlay's layout dimensions, NODE_CUSTOM_SHADOW offsets
        // and the radius passed through to Rosen are physical paint values.
        // Keep the CSS recipe in logical vp and convert at the paint boundary.
        format!(
            "0 {} {} #1A000000",
            self.offset_y * scale,
            self.blur * scale
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn negative_spread_shrinks_the_shadow_shape_and_corner_radius() {
        let layer = PopupShadowKind::Md.layers()[1];
        let bounds = layer.bounds(224.0, 100.0, 6.0).unwrap();
        assert_eq!(
            (bounds.width, bounds.height, bounds.radius),
            (220.0, 96.0, 4.0)
        );
        assert_eq!(layer.native_value(1.0), "0 2 4 #1A000000");
    }

    #[test]
    fn unmeasured_or_tiny_panels_do_not_paint_oversized_underlays() {
        let layer = PopupShadowKind::Lg.layers()[1];
        assert!(layer.bounds(224.0, 0.0, 6.0).is_none());
        assert!(layer.bounds(6.0, 6.0, 6.0).is_none());
        assert!(layer.bounds(f32::NAN, 100.0, 6.0).is_none());
        assert_eq!(layer.bounds(224.0, 100.0, 2.0).unwrap().radius, 0.0);
    }

    #[test]
    fn native_paint_keeps_the_same_logical_recipe_at_phone_density() {
        let layer = PopupShadowKind::Md.layers()[0];
        assert_eq!(layer.native_value(1.0), "0 4 6 #1A000000");
        assert_eq!(layer.native_value(1.5), "0 6 9 #1A000000");
        assert_eq!(layer.native_value(3.25), "0 13 19.5 #1A000000");
        assert_eq!(
            PopupShadowKind::Md.layers()[1].native_value(3.25),
            "0 6.5 13 #1A000000"
        );
    }
}
