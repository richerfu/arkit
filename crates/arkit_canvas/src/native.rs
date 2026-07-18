//! Canvas-specific value conversion on top of `ohos-drawing-binding`.
//!
//! Raw drawing handles stay inside the binding crate. Conversion behavior is
//! owned by the corresponding Canvas value or by a narrow extension trait on
//! the native drawing type.

use ohos_drawing_binding::{BlendMode, Canvas, LineCap, LineJoin, Matrix, PathFillType, Pen};

use crate::{CanvasLineCap, CanvasLineJoin, DomMatrix2D, FillRule, GlobalCompositeOperation};

impl DomMatrix2D {
    pub(crate) fn to_native_matrix(self) -> Matrix {
        Matrix::from_affine(self.a, self.b, self.c, self.d, self.e, self.f)
    }
}

impl FillRule {
    pub(crate) const fn to_native_fill_type(self) -> PathFillType {
        match self {
            Self::NonZero => PathFillType::Winding,
            Self::EvenOdd => PathFillType::EvenOdd,
        }
    }
}

impl GlobalCompositeOperation {
    pub(crate) const fn to_native_blend_mode(self) -> BlendMode {
        match self {
            Self::Copy => BlendMode::Src,
            Self::SourceOver => BlendMode::SrcOver,
            Self::DestinationOver => BlendMode::DstOver,
            Self::SourceIn => BlendMode::SrcIn,
            Self::DestinationIn => BlendMode::DstIn,
            Self::SourceOut => BlendMode::SrcOut,
            Self::DestinationOut => BlendMode::DstOut,
            Self::SourceAtop => BlendMode::SrcAtop,
            Self::DestinationAtop => BlendMode::DstAtop,
            Self::Xor => BlendMode::Xor,
            Self::Lighter => BlendMode::Plus,
            Self::Multiply => BlendMode::Multiply,
            Self::Screen => BlendMode::Screen,
            Self::Overlay => BlendMode::Overlay,
            Self::Darken => BlendMode::Darken,
            Self::Lighten => BlendMode::Lighten,
            Self::ColorDodge => BlendMode::ColorDodge,
            Self::ColorBurn => BlendMode::ColorBurn,
            Self::HardLight => BlendMode::HardLight,
            Self::SoftLight => BlendMode::SoftLight,
            Self::Difference => BlendMode::Difference,
            Self::Exclusion => BlendMode::Exclusion,
            Self::Hue => BlendMode::Hue,
            Self::Saturation => BlendMode::Saturation,
            Self::Color => BlendMode::Color,
            Self::Luminosity => BlendMode::Luminosity,
        }
    }
}

pub(crate) trait NativeCanvasExt {
    fn concat_dom_matrix(&self, value: DomMatrix2D);
    fn set_dom_transform(&self, value: DomMatrix2D, device_pixel_ratio: f32);
    fn reset_dom_transform(&self, device_pixel_ratio: f32);
}

impl NativeCanvasExt for Canvas {
    fn concat_dom_matrix(&self, value: DomMatrix2D) {
        self.concat(&value.to_native_matrix());
    }

    fn set_dom_transform(&self, value: DomMatrix2D, device_pixel_ratio: f32) {
        let device = DomMatrix2D::scaling(device_pixel_ratio, device_pixel_ratio).multiply(value);
        self.set_matrix(&device.to_native_matrix());
    }

    fn reset_dom_transform(&self, device_pixel_ratio: f32) {
        self.set_dom_transform(DomMatrix2D::IDENTITY, device_pixel_ratio);
    }
}

pub(crate) trait NativePenExt {
    fn set_canvas_geometry(
        &mut self,
        line_cap: CanvasLineCap,
        line_join: CanvasLineJoin,
        miter_limit: f32,
    );
}

impl NativePenExt for Pen {
    fn set_canvas_geometry(
        &mut self,
        line_cap: CanvasLineCap,
        line_join: CanvasLineJoin,
        miter_limit: f32,
    ) {
        self.set_cap(match line_cap {
            CanvasLineCap::Butt => LineCap::FlatCap,
            CanvasLineCap::Round => LineCap::RoundCap,
            CanvasLineCap::Square => LineCap::SquareCap,
        });
        self.set_join(match line_join {
            CanvasLineJoin::Miter => LineJoin::MiterJoin,
            CanvasLineJoin::Round => LineJoin::RoundJoin,
            CanvasLineJoin::Bevel => LineJoin::BevelJoin,
        });
        self.set_miter_limit(miter_limit);
    }
}
