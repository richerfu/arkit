use std::{borrow::Cow, cell::RefCell};

use ohos_drawing_binding::{
    Font, FontEdging, FontFeatures, FontHinting, FontManager, FontSlant, FontStyle, FontWeight,
    FontWidth, TextBlob, TextEncoding,
};

use crate::state::CanvasStyleState;
use crate::{
    CanvasFontKerning, CanvasFontRegistry, CanvasFontStretch, CanvasFontStyle,
    CanvasFontVariantCaps, CanvasTextAlign, CanvasTextBaseline, CanvasTextDirection,
    CanvasTextMetrics, CanvasTextRendering,
};

thread_local! {
    static FONT_MANAGER: RefCell<FontManager> = RefCell::new(FontManager::new());
}

pub(crate) struct TextPlacement {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) align: CanvasTextAlign,
    pub(crate) baseline: CanvasTextBaseline,
    pub(crate) direction: CanvasTextDirection,
    pub(crate) max_width: Option<f32>,
}

#[derive(Clone, Copy, Debug, Default)]
struct RunMeasurement {
    width: f32,
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
}

pub(crate) struct TextLayout {
    text: Box<str>,
    font: Font,
    features: Option<FontFeatures>,
    character_paint: bool,
    letter_spacing: f32,
    word_spacing: f32,
    measurement: RunMeasurement,
    height: f32,
    ascent: f32,
    font_top: f32,
    font_bottom: f32,
}

impl TextLayout {
    pub(crate) fn new(text: &str, state: &CanvasStyleState) -> Self {
        let text = normalize_canvas_text(text);
        let native_style = FontStyle::new(
            FontWeight::from_css(state.font.weight),
            native_font_width(state.font_stretch),
            match state.font.style {
                CanvasFontStyle::Normal => FontSlant::Normal,
                CanvasFontStyle::Italic => FontSlant::Italic,
                CanvasFontStyle::Oblique => FontSlant::Oblique,
            },
        );
        let families: Vec<_> = state
            .font
            .family
            .split(',')
            .map(|family| family.trim().trim_matches(['\'', '"']))
            .filter(|family| !family.is_empty())
            .collect();
        let registered_typeface = CanvasFontRegistry::resolve_typeface(&families);
        let system_typeface = registered_typeface
            .is_none()
            .then(|| {
                FONT_MANAGER.with_borrow(|manager| {
                    families
                        .iter()
                        .find_map(|family| manager.match_family_style(family, native_style))
                })
            })
            .flatten();

        let mut font = Font::new();
        font.set_text_size(state.font.size_px);
        if let Some(typeface) = registered_typeface {
            font.set_shared_typeface(typeface);
        } else if let Some(typeface) = system_typeface {
            font.set_typeface(typeface);
        } else {
            // Only synthesize weight/slant when the system has no matching
            // face. Matching an actual family/style remains the preferred path.
            font.set_fake_bold(state.font.weight >= 600);
            font.set_skew_x(match state.font.style {
                CanvasFontStyle::Normal => 0.0,
                CanvasFontStyle::Italic | CanvasFontStyle::Oblique => -0.25,
            });
        }
        configure_text_rendering(&mut font, state.text_rendering);

        let features = font_features(state);
        let character_paint = state.letter_spacing != 0.0
            || state.word_spacing != 0.0
            || state.font_kerning == CanvasFontKerning::None
            || features.is_some();
        let measurement = measure_run(
            &font,
            &text,
            state.letter_spacing,
            state.word_spacing,
            character_paint,
            features.as_ref(),
        );
        let metrics = font.metrics();
        let ascent = (-metrics.ascent).max(0.0);
        let descent = metrics.descent.max(0.0);
        let font_top = metrics.top.min(metrics.ascent);
        let font_bottom = metrics.bottom.max(metrics.descent);
        let height = (-font_top + font_bottom)
            .max(ascent + descent)
            .max(state.font.size_px);

        Self {
            text: text.into_owned().into_boxed_str(),
            font,
            features,
            character_paint,
            letter_spacing: state.letter_spacing,
            word_spacing: state.word_spacing,
            measurement,
            height,
            ascent,
            font_top,
            font_bottom,
        }
    }

    pub(crate) fn metrics(
        &self,
        align: CanvasTextAlign,
        baseline: CanvasTextBaseline,
        direction: CanvasTextDirection,
    ) -> CanvasTextMetrics {
        let origin_x = match direction.resolve_align(align) {
            CanvasTextAlign::Left => 0.0,
            CanvasTextAlign::Right => -self.measurement.width,
            CanvasTextAlign::Center => -self.measurement.width * 0.5,
            CanvasTextAlign::Start | CanvasTextAlign::End => unreachable!("alignment resolved"),
        };
        let baseline_from_top = self.baseline_from_top(baseline);
        let alphabetic_from_anchor = self.ascent - baseline_from_top;
        CanvasTextMetrics {
            width: self.measurement.width,
            actual_bounding_box_left: -(origin_x + self.measurement.left),
            actual_bounding_box_right: origin_x + self.measurement.right,
            font_bounding_box_ascent: -(alphabetic_from_anchor + self.font_top),
            font_bounding_box_descent: alphabetic_from_anchor + self.font_bottom,
            actual_bounding_box_ascent: -(alphabetic_from_anchor + self.measurement.top),
            actual_bounding_box_descent: alphabetic_from_anchor + self.measurement.bottom,
            em_height_ascent: baseline_from_top,
            em_height_descent: self.height - baseline_from_top,
            hanging_baseline: baseline_from_top - self.ascent * 0.2,
            alphabetic_baseline: baseline_from_top - self.ascent,
            ideographic_baseline: baseline_from_top - self.height,
        }
    }

    pub(crate) fn paint(&self, canvas: &ohos_drawing_binding::Canvas, placement: TextPlacement) {
        if self.text.is_empty() || self.measurement.width <= 0.0 {
            return;
        }
        let scale = placement
            .max_width
            .filter(|max_width| max_width.is_finite())
            .map_or(1.0, |max_width| {
                (max_width / self.measurement.width).min(1.0)
            });
        let painted_width = self.measurement.width * scale;
        let x = match placement.direction.resolve_align(placement.align) {
            CanvasTextAlign::Left => placement.x,
            CanvasTextAlign::Right => placement.x - painted_width,
            CanvasTextAlign::Center => placement.x - painted_width * 0.5,
            CanvasTextAlign::Start | CanvasTextAlign::End => unreachable!("alignment resolved"),
        };
        let y = placement.y - self.baseline_from_top(placement.baseline);
        canvas.save();
        canvas.translate(x, y);
        canvas.scale(scale, 1.0);
        if self.character_paint {
            self.paint_characters(canvas);
        } else if let Some(blob) = TextBlob::from_utf8(&self.text, &self.font) {
            canvas.draw_text_blob(&blob, 0.0, self.ascent);
        }
        canvas.restore();
    }

    fn paint_characters(&self, canvas: &ohos_drawing_binding::Canvas) {
        let characters: Vec<_> = self.text.chars().collect();
        let mut cursor = 0.0;
        for (index, character) in characters.iter().copied().enumerate() {
            let mut encoded = [0_u8; 4];
            let character = character.encode_utf8(&mut encoded);
            let feature_drawn = self.features.as_ref().is_some_and(|features| {
                canvas
                    .draw_single_character_with_features(
                        character,
                        &self.font,
                        cursor,
                        self.ascent,
                        features,
                    )
                    .is_ok()
            });
            if !feature_drawn {
                if let Some(blob) = TextBlob::from_utf8(character, &self.font) {
                    canvas.draw_text_blob(&blob, cursor, self.ascent);
                }
            }
            cursor += measure_character(&self.font, character, self.features.as_ref())
                .map_or(0.0, |measurement| measurement.width);
            if character == " " {
                cursor += self.word_spacing;
            }
            if index + 1 < characters.len() {
                cursor += self.letter_spacing;
            }
        }
    }

    fn baseline_from_top(&self, baseline: CanvasTextBaseline) -> f32 {
        match baseline {
            CanvasTextBaseline::Top => 0.0,
            CanvasTextBaseline::Hanging => self.ascent * 0.2,
            CanvasTextBaseline::Middle => self.height * 0.5,
            CanvasTextBaseline::Alphabetic => self.ascent,
            CanvasTextBaseline::Ideographic | CanvasTextBaseline::Bottom => self.height,
        }
    }
}

fn normalize_canvas_text(text: &str) -> Cow<'_, str> {
    if text
        .chars()
        .any(|character| matches!(character, '\t' | '\n' | '\u{000c}' | '\r'))
    {
        Cow::Owned(
            text.chars()
                .map(|character| {
                    if matches!(character, '\t' | '\n' | '\u{000c}' | '\r') {
                        ' '
                    } else {
                        character
                    }
                })
                .collect(),
        )
    } else {
        Cow::Borrowed(text)
    }
}

fn native_font_width(stretch: CanvasFontStretch) -> FontWidth {
    match stretch {
        CanvasFontStretch::UltraCondensed => FontWidth::UltraCondensed,
        CanvasFontStretch::ExtraCondensed => FontWidth::ExtraCondensed,
        CanvasFontStretch::Condensed => FontWidth::Condensed,
        CanvasFontStretch::SemiCondensed => FontWidth::SemiCondensed,
        CanvasFontStretch::Normal => FontWidth::Normal,
        CanvasFontStretch::SemiExpanded => FontWidth::SemiExpanded,
        CanvasFontStretch::Expanded => FontWidth::Expanded,
        CanvasFontStretch::ExtraExpanded => FontWidth::ExtraExpanded,
        CanvasFontStretch::UltraExpanded => FontWidth::UltraExpanded,
    }
}

fn configure_text_rendering(font: &mut Font, rendering: CanvasTextRendering) {
    match rendering {
        CanvasTextRendering::Auto => {
            font.set_hinting(FontHinting::Normal);
            font.set_edging(FontEdging::AntiAlias);
        }
        CanvasTextRendering::OptimizeSpeed => {
            font.set_hinting(FontHinting::Slight);
            font.set_edging(FontEdging::Alias);
            font.set_subpixel(false);
        }
        CanvasTextRendering::OptimizeLegibility => {
            font.set_hinting(FontHinting::Full);
            font.set_edging(FontEdging::SubpixelAntiAlias);
            font.set_subpixel(true);
            font.set_baseline_snap(true);
        }
        CanvasTextRendering::GeometricPrecision => {
            font.set_hinting(FontHinting::None);
            font.set_edging(FontEdging::AntiAlias);
            font.set_linear_text(true);
            font.set_subpixel(true);
            font.set_baseline_snap(false);
        }
    }
}

fn font_features(state: &CanvasStyleState) -> Option<FontFeatures> {
    let mut features = FontFeatures::new();
    let mut used = false;
    if state.font_kerning == CanvasFontKerning::None {
        used |= features.add("kern", 0.0).is_ok();
    }
    let tags: &[&str] = match state.font_variant_caps {
        CanvasFontVariantCaps::Normal => &[],
        CanvasFontVariantCaps::SmallCaps => &["smcp"],
        CanvasFontVariantCaps::AllSmallCaps => &["smcp", "c2sc"],
        CanvasFontVariantCaps::PetiteCaps => &["pcap"],
        CanvasFontVariantCaps::AllPetiteCaps => &["pcap", "c2pc"],
        CanvasFontVariantCaps::Unicase => &["unic"],
        CanvasFontVariantCaps::TitlingCaps => &["titl"],
    };
    for tag in tags {
        used |= features.add(tag, 1.0).is_ok();
    }
    used.then_some(features)
}

fn measure_run(
    font: &Font,
    text: &str,
    letter_spacing: f32,
    word_spacing: f32,
    character_paint: bool,
    features: Option<&FontFeatures>,
) -> RunMeasurement {
    if text.is_empty() {
        return RunMeasurement::default();
    }
    if !character_paint {
        return font
            .measure_text(text, TextEncoding::Utf8)
            .map(|measurement| RunMeasurement {
                width: measurement.width,
                left: measurement.bounds.left(),
                top: measurement.bounds.top(),
                right: measurement.bounds.right(),
                bottom: measurement.bounds.bottom(),
            })
            .unwrap_or_default();
    }

    let characters: Vec<_> = text.chars().collect();
    let mut result = RunMeasurement::default();
    let mut cursor = 0.0;
    let mut has_bounds = false;
    for (index, character) in characters.iter().copied().enumerate() {
        let mut encoded = [0_u8; 4];
        let character = character.encode_utf8(&mut encoded);
        if let Some(measurement) = measure_character(font, character, features) {
            let left = cursor + measurement.bounds.left();
            let right = cursor + measurement.bounds.right();
            if has_bounds {
                result.left = result.left.min(left);
                result.top = result.top.min(measurement.bounds.top());
                result.right = result.right.max(right);
                result.bottom = result.bottom.max(measurement.bounds.bottom());
            } else {
                result.left = left;
                result.top = measurement.bounds.top();
                result.right = right;
                result.bottom = measurement.bounds.bottom();
                has_bounds = true;
            }
            cursor += measurement.width;
        }
        if character == " " {
            cursor += word_spacing;
        }
        if index + 1 < characters.len() {
            cursor += letter_spacing;
        }
    }
    result.width = cursor;
    result
}

fn measure_character(
    font: &Font,
    character: &str,
    features: Option<&FontFeatures>,
) -> Option<ohos_drawing_binding::TextMeasure> {
    let mut measurement = font.measure_text(character, TextEncoding::Utf8).ok()?;
    if let Some(features) = features {
        if let Ok(width) = font.measure_single_character_with_features(character, features) {
            measurement.width = width;
        }
    }
    Some(measurement)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canvas_text_replaces_ascii_control_whitespace() {
        assert_eq!(normalize_canvas_text("a\tb\nc\rd\u{000c}e"), "a b c d e");
    }

    #[test]
    fn canvas_text_preserves_nul_and_non_ascii_whitespace() {
        assert_eq!(normalize_canvas_text("a\0b\u{2003}c"), "a\0b\u{2003}c");
    }
}
