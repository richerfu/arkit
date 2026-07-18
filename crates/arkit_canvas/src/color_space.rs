use crate::CanvasColorSpace;

/// Owns color-space conversion used by CSS colors and `ImageData`.
pub(crate) struct ColorSpaceTransform;

impl ColorSpaceTransform {
    pub(crate) fn convert(
        rgba: &mut [f32; 4],
        source: CanvasColorSpace,
        destination: CanvasColorSpace,
    ) {
        if source == destination {
            return;
        }
        let linear = [rgba[0], rgba[1], rgba[2]].map(Self::decode_srgb_transfer);
        let xyz = match source {
            CanvasColorSpace::Srgb => Self::multiply_rgb(linear, Self::SRGB_TO_XYZ_D65),
            CanvasColorSpace::DisplayP3 => Self::multiply_rgb(linear, Self::DISPLAY_P3_TO_XYZ_D65),
        };
        let converted = match destination {
            CanvasColorSpace::Srgb => Self::xyz_d65_to_linear_srgb(xyz),
            CanvasColorSpace::DisplayP3 => Self::multiply_rgb(xyz, Self::XYZ_D65_TO_DISPLAY_P3),
        };
        rgba[..3].copy_from_slice(&converted.map(Self::encode_srgb_transfer));
    }

    pub(crate) fn xyz_d65_to_srgb(xyz: [f32; 3]) -> [f32; 3] {
        Self::xyz_d65_to_linear_srgb(xyz).map(Self::encode_srgb_transfer)
    }

    pub(crate) fn xyz_d50_to_d65(xyz: [f32; 3]) -> [f32; 3] {
        Self::multiply_rgb(
            xyz,
            [
                [0.955_473_4, -0.023_098_5, 0.063_259_3],
                [-0.028_369_7, 1.009_995_5, 0.021_041_4],
                [0.012_314, -0.020_507_7, 1.330_365_9],
            ],
        )
    }

    pub(crate) fn decode_srgb_transfer(value: f32) -> f32 {
        let sign = value.signum();
        let value = value.abs();
        sign * if value <= 0.040_45 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    }

    pub(crate) fn encode_srgb_transfer(value: f32) -> f32 {
        let sign = value.signum();
        let value = value.abs();
        sign * if value <= 0.003_130_8 {
            value * 12.92
        } else {
            1.055 * value.powf(1.0 / 2.4) - 0.055
        }
    }

    pub(crate) fn multiply_rgb(rgb: [f32; 3], matrix: [[f32; 3]; 3]) -> [f32; 3] {
        matrix.map(|row| row[0] * rgb[0] + row[1] * rgb[1] + row[2] * rgb[2])
    }

    pub(crate) const DISPLAY_P3_TO_XYZ_D65: [[f32; 3]; 3] = [
        [0.486_570_95, 0.265_667_7, 0.198_217_29],
        [0.228_974_57, 0.691_738_55, 0.079_286_92],
        [0.0, 0.045_113_38, 1.043_944_4],
    ];

    const SRGB_TO_XYZ_D65: [[f32; 3]; 3] = [
        [0.412_390_8, 0.357_584_33, 0.180_480_8],
        [0.212_639, 0.715_168_65, 0.072_192_32],
        [0.019_330_818, 0.119_194_78, 0.950_532_14],
    ];

    const XYZ_D65_TO_SRGB: [[f32; 3]; 3] = [
        [3.240_97, -1.537_383_2, -0.498_610_76],
        [-0.969_243_65, 1.875_967_5, 0.041_555_06],
        [0.055_630_08, -0.203_976_96, 1.056_971_5],
    ];

    const XYZ_D65_TO_DISPLAY_P3: [[f32; 3]; 3] = [
        [2.493_497, -0.931_383_6, -0.402_710_8],
        [-0.829_489, 1.762_664, 0.023_624_685],
        [0.035_845_83, -0.076_172_39, 0.956_884_5],
    ];

    fn xyz_d65_to_linear_srgb(xyz: [f32; 3]) -> [f32; 3] {
        Self::multiply_rgb(xyz, Self::XYZ_D65_TO_SRGB)
    }
}
