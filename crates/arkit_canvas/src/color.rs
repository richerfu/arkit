use std::cell::RefCell;
use std::rc::Rc;

use ohos_drawing_binding::{Image, ShaderEffect, TileMode};

use crate::color_space::ColorSpaceTransform;
use crate::{CanvasError, CanvasImage, CanvasResult, DomMatrix2D};

/// An sRGB color stored in OpenHarmony's native ARGB layout.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CanvasColor(u32);

impl CanvasColor {
    pub const TRANSPARENT: Self = Self(0x0000_0000);
    pub const BLACK: Self = Self(0xFF00_0000);
    pub const WHITE: Self = Self(0xFFFF_FFFF);

    pub const fn from_argb(argb: u32) -> Self {
        Self(argb)
    }

    pub const fn rgb(red: u8, green: u8, blue: u8) -> Self {
        Self::rgba(red, green, blue, 255)
    }

    pub const fn rgba(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self(((alpha as u32) << 24) | ((red as u32) << 16) | ((green as u32) << 8) | blue as u32)
    }

    pub const fn to_argb(self) -> u32 {
        self.0
    }

    pub const fn alpha(self) -> u8 {
        (self.0 >> 24) as u8
    }

    pub(crate) fn with_global_alpha(self, global_alpha: f32) -> u32 {
        let alpha = (f32::from(self.alpha()) * global_alpha)
            .round()
            .clamp(0.0, 255.0) as u32;
        (self.0 & 0x00FF_FFFF) | (alpha << 24)
    }

    /// Parse an absolute CSS `<color>` into the native sRGB backing format.
    pub fn parse_css(value: &str) -> Option<Self> {
        let value = value.trim();
        if value.is_empty() {
            return None;
        }
        let normalized = value.to_ascii_lowercase();
        if normalized.starts_with("hsv(")
            || normalized.starts_with("hsva(")
            || normalized.starts_with("hwba(")
            || (!normalized.starts_with('#')
                && matches!(normalized.len(), 3 | 4 | 6 | 8)
                && normalized.bytes().all(|byte| byte.is_ascii_hexdigit()))
        {
            return None;
        }
        Self::parse_color_function(&normalized).or_else(|| {
            let color = csscolorparser::parse(&normalized).ok()?;
            let [red, green, blue, alpha] = color.to_rgba8();
            Some(Self::rgba(red, green, blue, alpha))
        })
    }
}

impl From<u32> for CanvasColor {
    fn from(value: u32) -> Self {
        Self::from_argb(value)
    }
}

#[derive(Clone, Debug, PartialEq)]
enum GradientKind {
    Linear {
        start: (f32, f32),
        end: (f32, f32),
    },
    Radial {
        start: (f32, f32),
        start_radius: f32,
        end: (f32, f32),
        end_radius: f32,
    },
    Conic {
        start_angle: f32,
        center: (f32, f32),
    },
}

#[derive(Clone, Debug)]
struct ColorStop {
    offset: f32,
    color: CanvasColor,
}

#[derive(Debug)]
struct GradientData {
    kind: GradientKind,
    transform: DomMatrix2D,
    stops: RefCell<Vec<ColorStop>>,
}

/// Mutable gradient object shared by cloned Canvas styles.
#[derive(Clone, Debug)]
pub struct CanvasGradient(Rc<GradientData>);

impl CanvasGradient {
    pub(crate) fn linear(start: (f32, f32), end: (f32, f32), transform: DomMatrix2D) -> Self {
        Self(Rc::new(GradientData {
            kind: GradientKind::Linear { start, end },
            transform,
            stops: RefCell::new(Vec::new()),
        }))
    }

    pub(crate) fn radial(
        start: (f32, f32),
        start_radius: f32,
        end: (f32, f32),
        end_radius: f32,
        transform: DomMatrix2D,
    ) -> Self {
        Self(Rc::new(GradientData {
            kind: GradientKind::Radial {
                start,
                start_radius,
                end,
                end_radius,
            },
            transform,
            stops: RefCell::new(Vec::new()),
        }))
    }

    pub(crate) fn conic(start_angle: f32, center: (f32, f32), transform: DomMatrix2D) -> Self {
        Self(Rc::new(GradientData {
            kind: GradientKind::Conic {
                start_angle,
                center,
            },
            transform,
            stops: RefCell::new(Vec::new()),
        }))
    }

    pub fn add_color_stop(&self, offset: f32, color: impl IntoCanvasStyle) -> CanvasResult<()> {
        if !offset.is_finite() || !(0.0..=1.0).contains(&offset) {
            return Err(CanvasError::InvalidColorStop);
        }
        let Some(CanvasStyle::Color(color)) = color.into_canvas_style() else {
            return Err(CanvasError::InvalidColorStop);
        };
        self.0.stops.borrow_mut().push(ColorStop { offset, color });
        Ok(())
    }

    pub(crate) fn shader(
        &self,
        global_alpha: f32,
        native_transform: DomMatrix2D,
    ) -> Option<ShaderEffect> {
        let mut stops = self.0.stops.borrow().clone();
        if stops.is_empty() {
            stops.extend([
                ColorStop {
                    offset: 0.0,
                    color: CanvasColor::TRANSPARENT,
                },
                ColorStop {
                    offset: 1.0,
                    color: CanvasColor::TRANSPARENT,
                },
            ]);
        } else if stops.len() == 1 {
            let color = stops[0].color;
            stops = vec![
                ColorStop { offset: 0.0, color },
                ColorStop { offset: 1.0, color },
            ];
        }
        stops.sort_by(|left, right| left.offset.total_cmp(&right.offset));
        if let GradientKind::Conic { start_angle, .. } = self.0.kind {
            stops = GradientData::rotate_conic_stops(stops, start_angle / std::f32::consts::TAU);
        }
        let colors: Vec<_> = stops
            .iter()
            .map(|stop| stop.color.with_global_alpha(global_alpha))
            .collect();
        let positions: Vec<_> = stops.iter().map(|stop| stop.offset).collect();
        // Gradients capture the CTM at creation time. If the native canvas is
        // currently transformed while painting (text uses this path), remove
        // that transform from the shader's local matrix so the gradient is not
        // transformed a second time.
        let matrix = native_transform
            .inverse()?
            .multiply(self.0.transform)
            .to_native_matrix();
        match self.0.kind {
            GradientKind::Linear { start, end } => {
                if start == end {
                    return None;
                }
                ShaderEffect::linear_gradient_with_local_matrix(
                    start,
                    end,
                    &colors,
                    &positions,
                    TileMode::Clamp,
                    &matrix,
                )
            }
            GradientKind::Radial {
                start,
                start_radius,
                end,
                end_radius,
            } => {
                if start == end && start_radius == end_radius {
                    return None;
                }
                ShaderEffect::two_point_conical_gradient(
                    start,
                    start_radius,
                    end,
                    end_radius,
                    &colors,
                    &positions,
                    TileMode::Clamp,
                    Some(&matrix),
                )
            }
            GradientKind::Conic { center, .. } => ShaderEffect::conic_gradient_with_local_matrix(
                center,
                &colors,
                &positions,
                TileMode::Clamp,
                &matrix,
            ),
        }
    }
}

impl PartialEq for CanvasGradient {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CanvasPatternRepetition {
    #[default]
    Repeat,
    RepeatX,
    RepeatY,
    NoRepeat,
}

#[derive(Debug)]
struct PatternData {
    image: CanvasImage,
    repetition: CanvasPatternRepetition,
    transform: RefCell<DomMatrix2D>,
}

/// Repeating image pattern corresponding to `CanvasPattern`.
#[derive(Clone, Debug)]
pub struct CanvasPattern(Rc<PatternData>);

impl CanvasPattern {
    pub(crate) fn new(image: CanvasImage, repetition: CanvasPatternRepetition) -> Self {
        Self(Rc::new(PatternData {
            image,
            repetition,
            transform: RefCell::new(DomMatrix2D::IDENTITY),
        }))
    }

    pub fn set_transform(&self, transform: DomMatrix2D) {
        if transform.is_finite() {
            self.0.transform.replace(transform);
        }
    }

    pub(crate) fn shader(
        &self,
        sampling: &ohos_drawing_binding::SamplingOptions,
        native_transform: DomMatrix2D,
        current_transform: DomMatrix2D,
    ) -> Option<(Image<'_>, ShaderEffect)> {
        let image = Image::from_bitmap(self.0.image.bitmap())?;
        let (tile_x, tile_y) = match self.0.repetition {
            CanvasPatternRepetition::Repeat => (TileMode::Repeat, TileMode::Repeat),
            CanvasPatternRepetition::RepeatX => (TileMode::Repeat, TileMode::Decal),
            CanvasPatternRepetition::RepeatY => (TileMode::Decal, TileMode::Repeat),
            CanvasPatternRepetition::NoRepeat => (TileMode::Decal, TileMode::Decal),
        };
        // Patterns are transformed first by their own matrix and then by the
        // CTM active at paint time. Compensate for whatever transform remains
        // on the native canvas used for the actual draw call.
        let matrix = native_transform
            .inverse()?
            .multiply(current_transform)
            .multiply(*self.0.transform.borrow())
            .to_native_matrix();
        let shader = ShaderEffect::image(&image, tile_x, tile_y, sampling, Some(&matrix))?;
        Some((image, shader))
    }
}

impl PartialEq for CanvasPattern {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

/// Color, gradient, or image pattern accepted by fill/stroke style.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
pub enum CanvasStyle {
    Color(CanvasColor),
    Gradient(CanvasGradient),
    Pattern(CanvasPattern),
}

impl CanvasStyle {
    pub const BLACK: Self = Self::Color(CanvasColor::BLACK);
}

impl From<CanvasColor> for CanvasStyle {
    fn from(value: CanvasColor) -> Self {
        Self::Color(value)
    }
}

impl From<CanvasGradient> for CanvasStyle {
    fn from(value: CanvasGradient) -> Self {
        Self::Gradient(value)
    }
}

impl From<CanvasPattern> for CanvasStyle {
    fn from(value: CanvasPattern) -> Self {
        Self::Pattern(value)
    }
}

impl From<u32> for CanvasStyle {
    fn from(value: u32) -> Self {
        Self::Color(CanvasColor::from_argb(value))
    }
}

/// Conversion used by style setters and gradient color stops.
pub trait IntoCanvasStyle {
    fn into_canvas_style(self) -> Option<CanvasStyle>;
}

impl IntoCanvasStyle for CanvasStyle {
    fn into_canvas_style(self) -> Option<CanvasStyle> {
        Some(self)
    }
}

impl IntoCanvasStyle for CanvasColor {
    fn into_canvas_style(self) -> Option<CanvasStyle> {
        Some(self.into())
    }
}

impl IntoCanvasStyle for CanvasGradient {
    fn into_canvas_style(self) -> Option<CanvasStyle> {
        Some(self.into())
    }
}

impl IntoCanvasStyle for CanvasPattern {
    fn into_canvas_style(self) -> Option<CanvasStyle> {
        Some(self.into())
    }
}

impl IntoCanvasStyle for u32 {
    fn into_canvas_style(self) -> Option<CanvasStyle> {
        Some(self.into())
    }
}

impl IntoCanvasStyle for &str {
    fn into_canvas_style(self) -> Option<CanvasStyle> {
        CanvasColor::parse_css(self).map(CanvasStyle::Color)
    }
}

impl IntoCanvasStyle for String {
    fn into_canvas_style(self) -> Option<CanvasStyle> {
        self.as_str().into_canvas_style()
    }
}

impl GradientData {
    fn rotate_conic_stops(stops: Vec<ColorStop>, rotation: f32) -> Vec<ColorStop> {
        let rotation = rotation.rem_euclid(1.0);
        if rotation <= f32::EPSILON {
            return stops;
        }
        let first = stops.first().cloned().unwrap();
        let last = stops.last().cloned().unwrap();
        let mut result: Vec<_> = stops
            .into_iter()
            .map(|stop| ColorStop {
                offset: (stop.offset + rotation).rem_euclid(1.0),
                color: stop.color,
            })
            .collect();
        result.push(ColorStop {
            offset: rotation,
            color: first.color,
        });
        result.push(ColorStop {
            offset: rotation,
            color: last.color,
        });
        result.sort_by(|left, right| left.offset.total_cmp(&right.offset));
        result
    }
}

impl CanvasColor {
    fn parse_color_function(value: &str) -> Option<CanvasColor> {
        let body = value.strip_prefix("color(")?.strip_suffix(')')?;
        let normalized = body.replace('/', " / ");
        let parts: Vec<_> = normalized.split_whitespace().collect();
        let profile = *parts.first()?;
        let slash = parts.iter().position(|part| *part == "/");
        let color_end = slash.unwrap_or(parts.len());
        if color_end != 4 || slash.is_some_and(|slash| slash + 2 != parts.len()) {
            return None;
        }
        let components = [
            Self::parse_color_component(parts[1])?,
            Self::parse_color_component(parts[2])?,
            Self::parse_color_component(parts[3])?,
        ];
        let alpha = slash.map_or(Some(1.0), |slash| {
            Self::parse_alpha_component(parts[slash + 1])
        })?;

        let rgb = match profile {
            "srgb" => components,
            "srgb-linear" => components.map(ColorSpaceTransform::encode_srgb_transfer),
            "display-p3" => {
                ColorSpaceTransform::xyz_d65_to_srgb(ColorSpaceTransform::multiply_rgb(
                    components.map(ColorSpaceTransform::decode_srgb_transfer),
                    ColorSpaceTransform::DISPLAY_P3_TO_XYZ_D65,
                ))
            }
            "a98-rgb" => ColorSpaceTransform::xyz_d65_to_srgb(ColorSpaceTransform::multiply_rgb(
                components.map(Self::decode_a98),
                A98_TO_XYZ_D65,
            )),
            "prophoto-rgb" => {
                let xyz_d50 = ColorSpaceTransform::multiply_rgb(
                    components.map(Self::decode_prophoto),
                    PROPHOTO_TO_XYZ_D50,
                );
                ColorSpaceTransform::xyz_d65_to_srgb(ColorSpaceTransform::xyz_d50_to_d65(xyz_d50))
            }
            "rec2020" => ColorSpaceTransform::xyz_d65_to_srgb(ColorSpaceTransform::multiply_rgb(
                components.map(Self::decode_rec2020),
                REC2020_TO_XYZ_D65,
            )),
            "xyz" | "xyz-d65" => ColorSpaceTransform::xyz_d65_to_srgb(components),
            "xyz-d50" => ColorSpaceTransform::xyz_d65_to_srgb(ColorSpaceTransform::xyz_d50_to_d65(
                components,
            )),
            _ => return None,
        };
        Some(CanvasColor::rgba(
            Self::float_to_unorm8(rgb[0]),
            Self::float_to_unorm8(rgb[1]),
            Self::float_to_unorm8(rgb[2]),
            Self::float_to_unorm8(alpha),
        ))
    }

    fn parse_color_component(value: &str) -> Option<f32> {
        if value == "none" {
            return Some(0.0);
        }
        if let Some(percent) = value.strip_suffix('%') {
            Some(percent.parse::<f32>().ok()? / 100.0)
        } else {
            value.parse::<f32>().ok().filter(|value| value.is_finite())
        }
    }

    fn parse_alpha_component(value: &str) -> Option<f32> {
        Self::parse_color_component(value).map(|value| value.clamp(0.0, 1.0))
    }

    fn decode_a98(value: f32) -> f32 {
        value.signum() * value.abs().powf(563.0 / 256.0)
    }

    fn decode_prophoto(value: f32) -> f32 {
        let sign = value.signum();
        let value = value.abs();
        sign * if value <= 16.0 / 512.0 {
            value / 16.0
        } else {
            value.powf(1.8)
        }
    }

    fn decode_rec2020(value: f32) -> f32 {
        const ALPHA: f32 = 1.099_296_8;
        const BETA: f32 = 0.018_053_97;
        let sign = value.signum();
        let value = value.abs();
        sign * if value < BETA * 4.5 {
            value / 4.5
        } else {
            ((value + ALPHA - 1.0) / ALPHA).powf(1.0 / 0.45)
        }
    }

    fn float_to_unorm8(value: f32) -> u8 {
        (value.clamp(0.0, 1.0) * 255.0).round() as u8
    }
}

const A98_TO_XYZ_D65: [[f32; 3]; 3] = [
    [0.576_669_04, 0.185_558_24, 0.188_228_65],
    [0.297_344_98, 0.627_363_56, 0.075_291_46],
    [0.027_031_36, 0.070_688_85, 0.991_337_54],
];

const PROPHOTO_TO_XYZ_D50: [[f32; 3]; 3] = [
    [0.797_766_6, 0.135_181_3, 0.031_347_733],
    [0.288_074_82, 0.711_835_2, 0.000_089_936_9],
    [0.0, 0.0, 0.825_104_6],
];

const REC2020_TO_XYZ_D65: [[f32; 3]; 3] = [
    [0.636_958_06, 0.144_616_9, 0.168_880_97],
    [0.262_700_2, 0.677_998_07, 0.059_301_715],
    [0.0, 0.028_072_694, 1.060_985_1],
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_css_hex_in_rgba_order() {
        assert_eq!(
            CanvasColor::parse_css("#369c"),
            Some(CanvasColor::rgba(0x33, 0x66, 0x99, 0xCC))
        );
        assert_eq!(
            CanvasColor::parse_css("#33669980"),
            Some(CanvasColor::rgba(0x33, 0x66, 0x99, 0x80))
        );
    }

    #[test]
    fn parses_modern_rgb_and_hsl() {
        assert_eq!(
            CanvasColor::parse_css("rgb(100% 0% 0% / 50%)"),
            Some(CanvasColor::rgba(255, 0, 0, 128))
        );
        assert_eq!(
            CanvasColor::parse_css("hsl(120 100% 50%)"),
            Some(CanvasColor::rgba(0, 255, 0, 255))
        );
    }

    #[test]
    fn parses_css_color_level_four_surface() {
        assert_eq!(
            CanvasColor::parse_css("aliceblue"),
            Some(CanvasColor::rgb(240, 248, 255))
        );
        assert!(CanvasColor::parse_css("hwb(120 20% 10% / 40%)").is_some());
        assert!(CanvasColor::parse_css("lab(50% 20 -30)").is_some());
        assert!(CanvasColor::parse_css("lch(60% 40 120deg)").is_some());
        assert!(CanvasColor::parse_css("oklab(60% 0.1 -0.1)").is_some());
        assert!(CanvasColor::parse_css("oklch(60% 0.1 0.5turn)").is_some());
    }

    #[test]
    fn parses_css_predefined_color_spaces_into_srgb() {
        assert_eq!(
            CanvasColor::parse_css("color(xyz-d65 0.4123908 0.212639 0.0193308)"),
            Some(CanvasColor::rgb(255, 0, 0))
        );
        assert_eq!(
            CanvasColor::parse_css("color(display-p3 1 0 0 / 50%)").map(CanvasColor::alpha),
            Some(128)
        );
        assert!(CanvasColor::parse_css("color(a98-rgb 0.8 0.2 0.1)").is_some());
        assert!(CanvasColor::parse_css("color(prophoto-rgb 0.8 0.2 0.1)").is_some());
        assert!(CanvasColor::parse_css("color(rec2020 0.8 0.2 0.1)").is_some());
    }

    #[test]
    fn rejects_non_css_color_extensions() {
        assert_eq!(CanvasColor::parse_css("ffffff"), None);
        assert_eq!(CanvasColor::parse_css("hsv(120 100% 50%)"), None);
        assert_eq!(CanvasColor::parse_css("hwba(120 0% 0% / 1)"), None);
    }

    #[test]
    fn gradient_preserves_the_complete_creation_transform() {
        let transform = DomMatrix2D::new(2.0, 0.35, -0.2, 0.75, 18.0, 24.0);
        let gradient = CanvasGradient::linear((0.0, 0.0), (100.0, 0.0), transform);
        assert_eq!(gradient.0.transform, transform);
        assert_eq!(
            gradient.0.kind,
            GradientKind::Linear {
                start: (0.0, 0.0),
                end: (100.0, 0.0),
            }
        );
    }
}
