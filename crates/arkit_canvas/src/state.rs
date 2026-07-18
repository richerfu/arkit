use smallvec::SmallVec;

use crate::{CanvasStyle, IntoCanvasStyle};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DomMatrix2D {
    pub a: f32,
    pub b: f32,
    pub c: f32,
    pub d: f32,
    pub e: f32,
    pub f: f32,
}

impl Default for DomMatrix2D {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl DomMatrix2D {
    pub const IDENTITY: Self = Self {
        a: 1.0,
        b: 0.0,
        c: 0.0,
        d: 1.0,
        e: 0.0,
        f: 0.0,
    };

    pub const fn new(a: f32, b: f32, c: f32, d: f32, e: f32, f: f32) -> Self {
        Self { a, b, c, d, e, f }
    }

    pub fn is_finite(self) -> bool {
        [self.a, self.b, self.c, self.d, self.e, self.f]
            .into_iter()
            .all(f32::is_finite)
    }

    pub fn multiply(self, other: Self) -> Self {
        Self {
            a: self.a * other.a + self.c * other.b,
            b: self.b * other.a + self.d * other.b,
            c: self.a * other.c + self.c * other.d,
            d: self.b * other.c + self.d * other.d,
            e: self.a * other.e + self.c * other.f + self.e,
            f: self.b * other.e + self.d * other.f + self.f,
        }
    }

    pub fn transform_point(self, x: f32, y: f32) -> (f32, f32) {
        (
            self.a * x + self.c * y + self.e,
            self.b * x + self.d * y + self.f,
        )
    }

    pub fn inverse(self) -> Option<Self> {
        let determinant = self.a * self.d - self.b * self.c;
        if !determinant.is_finite() || determinant == 0.0 {
            return None;
        }
        let inverse = determinant.recip();
        Some(Self::new(
            self.d * inverse,
            -self.b * inverse,
            -self.c * inverse,
            self.a * inverse,
            (self.c * self.f - self.d * self.e) * inverse,
            (self.b * self.e - self.a * self.f) * inverse,
        ))
    }

    pub const fn translation(x: f32, y: f32) -> Self {
        Self::new(1.0, 0.0, 0.0, 1.0, x, y)
    }

    pub const fn scaling(x: f32, y: f32) -> Self {
        Self::new(x, 0.0, 0.0, y, 0.0, 0.0)
    }

    pub fn rotation(radians: f32) -> Self {
        let (sin, cos) = radians.sin_cos();
        Self::new(cos, sin, -sin, cos, 0.0, 0.0)
    }
}

impl Default for CanvasStyleState {
    fn default() -> Self {
        Self {
            fill_style: CanvasStyle::BLACK,
            stroke_style: CanvasStyle::BLACK,
            global_alpha: 1.0,
            global_composite_operation: GlobalCompositeOperation::SourceOver,
            line_width: 1.0,
            line_cap: CanvasLineCap::Butt,
            line_join: CanvasLineJoin::Miter,
            miter_limit: 10.0,
            line_dash: SmallVec::new(),
            line_dash_offset: 0.0,
            font: CanvasFont::default(),
            text_align: CanvasTextAlign::Start,
            text_baseline: CanvasTextBaseline::Alphabetic,
            direction: CanvasTextDirection::Inherit,
            font_kerning: CanvasFontKerning::Auto,
            font_stretch: CanvasFontStretch::Normal,
            font_variant_caps: CanvasFontVariantCaps::Normal,
            text_rendering: CanvasTextRendering::Auto,
            letter_spacing: 0.0,
            word_spacing: 0.0,
            lang: Box::from("inherit"),
            image_smoothing_enabled: true,
            image_smoothing_quality: CanvasImageSmoothingQuality::Low,
            shadow_offset_x: 0.0,
            shadow_offset_y: 0.0,
            shadow_blur: 0.0,
            shadow_color: crate::CanvasColor::TRANSPARENT,
            filter: Box::from("none"),
            transform: DomMatrix2D::IDENTITY,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CanvasStyleState {
    pub fill_style: CanvasStyle,
    pub stroke_style: CanvasStyle,
    pub global_alpha: f32,
    pub global_composite_operation: GlobalCompositeOperation,
    pub line_width: f32,
    pub line_cap: CanvasLineCap,
    pub line_join: CanvasLineJoin,
    pub miter_limit: f32,
    pub line_dash: SmallVec<[f32; 4]>,
    pub line_dash_offset: f32,
    pub font: CanvasFont,
    pub text_align: CanvasTextAlign,
    pub text_baseline: CanvasTextBaseline,
    pub direction: CanvasTextDirection,
    pub font_kerning: CanvasFontKerning,
    pub font_stretch: CanvasFontStretch,
    pub font_variant_caps: CanvasFontVariantCaps,
    pub text_rendering: CanvasTextRendering,
    pub letter_spacing: f32,
    pub word_spacing: f32,
    pub lang: Box<str>,
    pub image_smoothing_enabled: bool,
    pub image_smoothing_quality: CanvasImageSmoothingQuality,
    pub shadow_offset_x: f32,
    pub shadow_offset_y: f32,
    pub shadow_blur: f32,
    pub shadow_color: crate::CanvasColor,
    pub filter: Box<str>,
    pub transform: DomMatrix2D,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FillRule {
    #[default]
    NonZero,
    EvenOdd,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CanvasLineCap {
    #[default]
    Butt,
    Round,
    Square,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CanvasLineJoin {
    #[default]
    Miter,
    Round,
    Bevel,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GlobalCompositeOperation {
    Copy,
    #[default]
    SourceOver,
    DestinationOver,
    SourceIn,
    DestinationIn,
    SourceOut,
    DestinationOut,
    SourceAtop,
    DestinationAtop,
    Xor,
    Lighter,
    Multiply,
    Screen,
    Overlay,
    Darken,
    Lighten,
    ColorDodge,
    ColorBurn,
    HardLight,
    SoftLight,
    Difference,
    Exclusion,
    Hue,
    Saturation,
    Color,
    Luminosity,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum CanvasFontStyle {
    #[default]
    Normal,
    Italic,
    Oblique,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CanvasFont {
    pub style: CanvasFontStyle,
    pub weight: u16,
    pub stretch: CanvasFontStretch,
    pub variant_caps: CanvasFontVariantCaps,
    pub size_px: f32,
    pub family: Box<str>,
}

impl CanvasFont {
    pub fn new(size_px: f32, family: impl Into<Box<str>>) -> Self {
        Self {
            size_px: if size_px.is_finite() && size_px > 0.0 {
                size_px
            } else {
                Self::default().size_px
            },
            family: family.into(),
            ..Self::default()
        }
    }

    pub fn with_weight(mut self, weight: u16) -> Self {
        self.weight = weight.clamp(1, 1000);
        self
    }

    pub const fn with_style(mut self, style: CanvasFontStyle) -> Self {
        self.style = style;
        self
    }

    pub const fn with_stretch(mut self, stretch: CanvasFontStretch) -> Self {
        self.stretch = stretch;
        self
    }

    pub const fn with_variant_caps(mut self, variant_caps: CanvasFontVariantCaps) -> Self {
        self.variant_caps = variant_caps;
        self
    }

    pub fn parse_css(value: &str) -> Option<Self> {
        let tokens: Vec<_> = value.split_whitespace().collect();
        let (size_index, size) = tokens.iter().enumerate().find_map(|(index, token)| {
            let size = token.split_once('/').map_or(*token, |(size, _)| size);
            parse_absolute_css_length(size).map(|size| (index, size))
        })?;
        if !size.is_finite() || size <= 0.0 {
            return None;
        }
        let family = tokens.get(size_index + 1..)?.join(" ");
        let family = family.trim_matches(['\'', '"']).trim();
        if family.is_empty() {
            return None;
        }
        let mut font = Self::new(size, family.to_owned().into_boxed_str());
        for token in &tokens[..size_index] {
            match *token {
                "normal" => {}
                "italic" => font.style = CanvasFontStyle::Italic,
                "oblique" => font.style = CanvasFontStyle::Oblique,
                "small-caps" => font.variant_caps = CanvasFontVariantCaps::SmallCaps,
                "bold" => font.weight = 700,
                "ultra-condensed" => font.stretch = CanvasFontStretch::UltraCondensed,
                "extra-condensed" => font.stretch = CanvasFontStretch::ExtraCondensed,
                "condensed" => font.stretch = CanvasFontStretch::Condensed,
                "semi-condensed" => font.stretch = CanvasFontStretch::SemiCondensed,
                "semi-expanded" => font.stretch = CanvasFontStretch::SemiExpanded,
                "expanded" => font.stretch = CanvasFontStretch::Expanded,
                "extra-expanded" => font.stretch = CanvasFontStretch::ExtraExpanded,
                "ultra-expanded" => font.stretch = CanvasFontStretch::UltraExpanded,
                value => {
                    if let Ok(weight) = value.parse::<u16>() {
                        font.weight = weight.clamp(1, 1000);
                    } else {
                        return None;
                    }
                }
            }
        }
        Some(font)
    }
}

impl Default for CanvasFont {
    fn default() -> Self {
        Self {
            style: CanvasFontStyle::Normal,
            weight: 400,
            stretch: CanvasFontStretch::Normal,
            variant_caps: CanvasFontVariantCaps::Normal,
            size_px: 10.0,
            family: Box::from("sans-serif"),
        }
    }
}

pub trait IntoCanvasFont {
    fn into_canvas_font(self) -> Option<CanvasFont>;
}

impl IntoCanvasFont for CanvasFont {
    fn into_canvas_font(self) -> Option<CanvasFont> {
        Some(self)
    }
}

impl IntoCanvasFont for &str {
    fn into_canvas_font(self) -> Option<CanvasFont> {
        CanvasFont::parse_css(self)
    }
}

impl IntoCanvasFont for String {
    fn into_canvas_font(self) -> Option<CanvasFont> {
        self.as_str().into_canvas_font()
    }
}

/// A resolved CSS `<length>` used by `letterSpacing` and `wordSpacing`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CanvasTextSpacing(f32);

impl CanvasTextSpacing {
    pub fn from_pixels(pixels: f32) -> Option<Self> {
        pixels.is_finite().then_some(Self(pixels))
    }

    pub fn parse_css(value: &str, font_size: f32) -> Option<Self> {
        let value = value.trim().to_ascii_lowercase();
        if value == "normal" {
            return Some(Self(0.0));
        }
        let pixels = value
            .strip_suffix("em")
            .and_then(|number| number.parse::<f32>().ok())
            .map(|number| number * font_size)
            .or_else(|| parse_absolute_css_length(&value))?;
        Self::from_pixels(pixels)
    }

    pub const fn pixels(self) -> f32 {
        self.0
    }
}

pub trait IntoCanvasTextSpacing {
    fn into_canvas_text_spacing(self, font_size: f32) -> Option<CanvasTextSpacing>;
}

impl IntoCanvasTextSpacing for CanvasTextSpacing {
    fn into_canvas_text_spacing(self, _font_size: f32) -> Option<CanvasTextSpacing> {
        Some(self)
    }
}

impl IntoCanvasTextSpacing for f32 {
    fn into_canvas_text_spacing(self, _font_size: f32) -> Option<CanvasTextSpacing> {
        CanvasTextSpacing::from_pixels(self)
    }
}

impl IntoCanvasTextSpacing for f64 {
    fn into_canvas_text_spacing(self, _font_size: f32) -> Option<CanvasTextSpacing> {
        CanvasTextSpacing::from_pixels(self as f32)
    }
}

impl IntoCanvasTextSpacing for &str {
    fn into_canvas_text_spacing(self, font_size: f32) -> Option<CanvasTextSpacing> {
        CanvasTextSpacing::parse_css(self, font_size)
    }
}

impl IntoCanvasTextSpacing for String {
    fn into_canvas_text_spacing(self, font_size: f32) -> Option<CanvasTextSpacing> {
        self.as_str().into_canvas_text_spacing(font_size)
    }
}

fn parse_absolute_css_length(value: &str) -> Option<f32> {
    let value = value.trim().to_ascii_lowercase();
    let units = [
        ("px", 1.0),
        ("pt", 96.0 / 72.0),
        ("pc", 16.0),
        ("in", 96.0),
        ("cm", 96.0 / 2.54),
        ("mm", 96.0 / 25.4),
        ("q", 96.0 / 101.6),
    ];
    for (unit, factor) in units {
        if let Some(number) = value.strip_suffix(unit) {
            let number = number.parse::<f32>().ok()?;
            let pixels = number * factor;
            return pixels.is_finite().then_some(pixels);
        }
    }
    let number = value.parse::<f32>().ok()?;
    (number == 0.0).then_some(0.0)
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CanvasTextAlign {
    Left,
    Right,
    Center,
    #[default]
    Start,
    End,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CanvasTextBaseline {
    Top,
    Hanging,
    Middle,
    #[default]
    Alphabetic,
    Ideographic,
    Bottom,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CanvasTextDirection {
    Ltr,
    Rtl,
    #[default]
    Inherit,
}

impl CanvasTextDirection {
    pub(crate) const fn resolve_align(self, align: CanvasTextAlign) -> CanvasTextAlign {
        match (self, align) {
            (Self::Rtl, CanvasTextAlign::Start) => CanvasTextAlign::Right,
            (Self::Rtl, CanvasTextAlign::End) => CanvasTextAlign::Left,
            (_, CanvasTextAlign::Start) => CanvasTextAlign::Left,
            (_, CanvasTextAlign::End) => CanvasTextAlign::Right,
            (_, align) => align,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CanvasFontKerning {
    #[default]
    Auto,
    Normal,
    None,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CanvasFontStretch {
    UltraCondensed,
    ExtraCondensed,
    Condensed,
    SemiCondensed,
    #[default]
    Normal,
    SemiExpanded,
    Expanded,
    ExtraExpanded,
    UltraExpanded,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CanvasFontVariantCaps {
    #[default]
    Normal,
    SmallCaps,
    AllSmallCaps,
    PetiteCaps,
    AllPetiteCaps,
    Unicase,
    TitlingCaps,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CanvasTextRendering {
    #[default]
    Auto,
    OptimizeSpeed,
    OptimizeLegibility,
    GeometricPrecision,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CanvasImageSmoothingQuality {
    #[default]
    Low,
    Medium,
    High,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CanvasRenderingContext2DSettings {
    pub alpha: bool,
    pub desynchronized: bool,
    pub color_space: CanvasColorSpace,
    pub color_type: CanvasColorType,
    pub will_read_frequently: bool,
}

impl Default for CanvasRenderingContext2DSettings {
    fn default() -> Self {
        Self {
            alpha: true,
            desynchronized: false,
            color_space: CanvasColorSpace::Srgb,
            color_type: CanvasColorType::Unorm8,
            will_read_frequently: false,
        }
    }
}

impl CanvasRenderingContext2DSettings {
    pub(crate) fn resolved_for_native_bitmap(self) -> Self {
        // OH_Drawing_Bitmap currently exposes only 8-bit sRGB-compatible
        // formats. Report the actual backing-store attributes instead of
        // claiming Display-P3/float16 support that the native surface cannot
        // preserve.
        Self {
            color_space: CanvasColorSpace::Srgb,
            color_type: CanvasColorType::Unorm8,
            ..self
        }
    }

    pub(crate) const fn blank_color(self) -> u32 {
        if self.alpha {
            0x0000_0000
        } else {
            0xFF00_0000
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CanvasColorSpace {
    #[default]
    Srgb,
    DisplayP3,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CanvasColorType {
    #[default]
    Unorm8,
    Float16,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CanvasTextMetrics {
    pub width: f32,
    pub actual_bounding_box_left: f32,
    pub actual_bounding_box_right: f32,
    pub font_bounding_box_ascent: f32,
    pub font_bounding_box_descent: f32,
    pub actual_bounding_box_ascent: f32,
    pub actual_bounding_box_descent: f32,
    pub em_height_ascent: f32,
    pub em_height_descent: f32,
    pub hanging_baseline: f32,
    pub alphabetic_baseline: f32,
    pub ideographic_baseline: f32,
}

impl CanvasStyleState {
    pub(crate) fn set_fill_style(&mut self, value: impl IntoCanvasStyle) {
        if let Some(style) = value.into_canvas_style() {
            self.fill_style = style;
        }
    }

    pub(crate) fn set_stroke_style(&mut self, value: impl IntoCanvasStyle) {
        if let Some(style) = value.into_canvas_style() {
            self.stroke_style = style;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matrix_multiplication_matches_dom_affine_order() {
        let matrix = DomMatrix2D::translation(10.0, 20.0).multiply(DomMatrix2D::scaling(2.0, 3.0));
        assert_eq!(matrix, DomMatrix2D::new(2.0, 0.0, 0.0, 3.0, 10.0, 20.0));
    }

    #[test]
    fn invertible_small_scale_is_not_treated_as_singular() {
        let matrix = DomMatrix2D::scaling(0.001, 0.001);
        let inverse = matrix.inverse().expect("non-zero CTM remains invertible");
        let identity = matrix.multiply(inverse);
        assert!((identity.a - 1.0).abs() < 1.0e-5);
        assert!((identity.d - 1.0).abs() < 1.0e-5);
    }

    #[test]
    fn logical_text_alignment_follows_direction() {
        assert_eq!(
            CanvasTextDirection::Ltr.resolve_align(CanvasTextAlign::Start),
            CanvasTextAlign::Left
        );
        assert_eq!(
            CanvasTextDirection::Rtl.resolve_align(CanvasTextAlign::Start),
            CanvasTextAlign::Right
        );
        assert_eq!(
            CanvasTextDirection::Rtl.resolve_align(CanvasTextAlign::End),
            CanvasTextAlign::Left
        );
        assert_eq!(
            CanvasTextDirection::Rtl.resolve_align(CanvasTextAlign::Center),
            CanvasTextAlign::Center
        );
    }

    #[test]
    fn parses_canvas_css_font_shorthand() {
        assert_eq!(
            CanvasFont::parse_css("italic 700 24.5px sans-serif"),
            Some(CanvasFont {
                style: CanvasFontStyle::Italic,
                weight: 700,
                stretch: CanvasFontStretch::Normal,
                variant_caps: CanvasFontVariantCaps::Normal,
                size_px: 24.5,
                family: Box::from("sans-serif"),
            })
        );
    }

    #[test]
    fn parses_absolute_font_units_and_spacing_lengths() {
        let font = CanvasFont::parse_css("small-caps condensed 700 12pt/1.4 'Noto Sans'")
            .expect("valid CSS font shorthand");
        assert_eq!(font.size_px, 16.0);
        assert_eq!(font.stretch, CanvasFontStretch::Condensed);
        assert_eq!(font.variant_caps, CanvasFontVariantCaps::SmallCaps);
        assert_eq!(font.family.as_ref(), "Noto Sans");

        assert_eq!(
            CanvasTextSpacing::parse_css("0.25em", 20.0),
            Some(CanvasTextSpacing(5.0))
        );
        assert_eq!(
            CanvasTextSpacing::parse_css("6pt", 20.0),
            Some(CanvasTextSpacing(8.0))
        );
        assert_eq!(CanvasTextSpacing::parse_css("12%", 20.0), None);
    }

    #[test]
    fn context_attributes_report_native_backing_store_fallbacks() {
        let requested = CanvasRenderingContext2DSettings {
            alpha: false,
            desynchronized: true,
            color_space: CanvasColorSpace::DisplayP3,
            color_type: CanvasColorType::Float16,
            will_read_frequently: true,
        };
        assert_eq!(
            requested.resolved_for_native_bitmap(),
            CanvasRenderingContext2DSettings {
                alpha: false,
                desynchronized: true,
                color_space: CanvasColorSpace::Srgb,
                color_type: CanvasColorType::Unorm8,
                will_read_frequently: true,
            }
        );
    }
}
