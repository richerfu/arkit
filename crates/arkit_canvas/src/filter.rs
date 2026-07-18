use ohos_drawing_binding::{Brush, ColorFilter, Filter, MaskFilter, Pen, ShadowLayer};

use crate::CanvasColor;

#[derive(Debug)]
pub(crate) struct PaintFilter {
    filter: Filter,
    _color: Option<ColorFilter>,
    _blur: Option<MaskFilter>,
    drop_shadow: Option<ShadowLayer>,
}

impl PaintFilter {
    pub(crate) fn is_valid_css(value: &str) -> bool {
        ParsedFilter::parse(value).is_some()
    }

    pub(crate) fn from_css(value: &str) -> Option<Self> {
        let parsed = ParsedFilter::parse(value)?;
        if parsed.identity && parsed.blur == 0.0 && parsed.drop_shadow.is_none() {
            return None;
        }
        let color = (!parsed.identity)
            .then(|| ColorFilter::matrix(parsed.matrix))
            .flatten();
        let blur = (parsed.blur > 0.0)
            .then(|| MaskFilter::blur(parsed.blur * 0.5, true))
            .flatten();
        let mut filter = Filter::new();
        filter.set_color_filter(color.as_ref());
        filter.set_mask_filter(blur.as_ref());
        Some(Self {
            filter,
            _color: color,
            _blur: blur,
            drop_shadow: parsed.drop_shadow.and_then(|shadow| {
                ShadowLayer::new(shadow.blur * 0.5, shadow.x, shadow.y, shadow.color)
            }),
        })
    }

    pub(crate) fn apply_brush(&self, brush: &mut Brush) {
        brush.set_filter(Some(&self.filter));
        if self.drop_shadow.is_some() {
            brush.set_shadow_layer(self.drop_shadow.as_ref());
        }
    }

    pub(crate) fn apply_pen(&self, pen: &mut Pen) {
        pen.set_filter(Some(&self.filter));
        if self.drop_shadow.is_some() {
            pen.set_shadow_layer(self.drop_shadow.as_ref());
        }
    }
}

struct ParsedFilter {
    matrix: [f32; 20],
    blur: f32,
    identity: bool,
    drop_shadow: Option<DropShadow>,
}

struct DropShadow {
    x: f32,
    y: f32,
    blur: f32,
    color: u32,
}

impl ParsedFilter {
    fn parse(value: &str) -> Option<Self> {
        let value = value.trim();
        if value == "none" {
            return Some(ParsedFilter {
                matrix: Self::identity(),
                blur: 0.0,
                identity: true,
                drop_shadow: None,
            });
        }
        let functions = Self::split_functions(value)?;
        if functions.is_empty() {
            return None;
        }
        let mut matrix = Self::identity();
        let mut blur = 0.0_f32;
        let mut changed = false;
        let mut drop_shadow = None;
        for (name, argument) in functions {
            let next = match name {
                "blur" => {
                    let radius = Self::parse_length(argument)?;
                    if radius < 0.0 {
                        return None;
                    }
                    blur = blur.hypot(radius);
                    continue;
                }
                "brightness" => Self::diagonal(Self::parse_amount(argument, 1.0, false)?, 1.0),
                "contrast" => Self::contrast(Self::parse_amount(argument, 1.0, false)?),
                "grayscale" => Self::saturate(1.0 - Self::parse_amount(argument, 1.0, true)?),
                "hue-rotate" => Self::hue_rotate(Self::parse_angle(argument)?),
                "invert" => Self::invert(Self::parse_amount(argument, 1.0, true)?),
                "opacity" => Self::diagonal(1.0, Self::parse_amount(argument, 1.0, true)?),
                "saturate" => Self::saturate(Self::parse_amount(argument, 1.0, false)?),
                "sepia" => Self::sepia(Self::parse_amount(argument, 1.0, true)?),
                "drop-shadow" => {
                    if drop_shadow.is_some() {
                        return None;
                    }
                    drop_shadow = Some(Self::parse_drop_shadow(argument)?);
                    continue;
                }
                _ => return None,
            };
            matrix = Self::multiply(next, matrix);
            changed = true;
        }
        Some(Self {
            matrix,
            blur,
            identity: !changed,
            drop_shadow,
        })
    }

    fn parse_drop_shadow(value: &str) -> Option<DropShadow> {
        let parts = Self::split_arguments(value);
        if !(2..=4).contains(&parts.len()) {
            return None;
        }
        let x = Self::parse_length(parts[0])?;
        let y = Self::parse_length(parts[1])?;
        let mut blur = 0.0;
        let mut color = CanvasColor::BLACK.to_argb();
        for part in &parts[2..] {
            if let Some(length) = Self::parse_length(part) {
                if length < 0.0 || blur != 0.0 {
                    return None;
                }
                blur = length;
            } else {
                color = CanvasColor::parse_css(part)?.to_argb();
            }
        }
        Some(DropShadow { x, y, blur, color })
    }

    fn split_arguments(value: &str) -> Vec<&str> {
        let mut result = Vec::new();
        let mut start = None;
        let mut depth = 0_u32;
        for (offset, character) in value.char_indices() {
            match character {
                '(' => {
                    depth += 1;
                    start.get_or_insert(offset);
                }
                ')' => depth = depth.saturating_sub(1),
                character if character.is_whitespace() && depth == 0 => {
                    if let Some(start) = start.take() {
                        result.push(&value[start..offset]);
                    }
                }
                _ => {
                    start.get_or_insert(offset);
                }
            }
        }
        if let Some(start) = start {
            result.push(&value[start..]);
        }
        result
    }

    fn split_functions(mut value: &str) -> Option<Vec<(&str, &str)>> {
        let mut result = Vec::new();
        while !value.trim_start().is_empty() {
            value = value.trim_start();
            let open = value.find('(')?;
            let name = value[..open].trim();
            if name.is_empty() || name.chars().any(char::is_whitespace) {
                return None;
            }
            let mut depth = 0_u32;
            let mut close = None;
            for (offset, character) in value[open..].char_indices() {
                match character {
                    '(' => depth += 1,
                    ')' => {
                        depth = depth.checked_sub(1)?;
                        if depth == 0 {
                            close = Some(open + offset);
                            break;
                        }
                    }
                    _ => {}
                }
            }
            let close = close?;
            result.push((name, value[open + 1..close].trim()));
            value = &value[close + 1..];
        }
        Some(result)
    }

    fn parse_amount(value: &str, default: f32, clamp_one: bool) -> Option<f32> {
        let mut amount = if value.is_empty() {
            default
        } else if let Some(percent) = value.strip_suffix('%') {
            percent.trim().parse::<f32>().ok()? / 100.0
        } else {
            value.parse::<f32>().ok()?
        };
        if !amount.is_finite() || amount < 0.0 {
            return None;
        }
        if clamp_one {
            amount = amount.min(1.0);
        }
        Some(amount)
    }

    fn parse_length(value: &str) -> Option<f32> {
        let value = value.trim();
        if value == "0" {
            return Some(0.0);
        }
        value.strip_suffix("px")?.trim().parse().ok()
    }

    fn parse_angle(value: &str) -> Option<f32> {
        let value = value.trim();
        if value == "0" {
            return Some(0.0);
        }
        if let Some(degrees) = value.strip_suffix("deg") {
            Some(degrees.trim().parse::<f32>().ok()?.to_radians())
        } else if let Some(radians) = value.strip_suffix("rad") {
            radians.trim().parse().ok()
        } else if let Some(turns) = value.strip_suffix("turn") {
            Some(turns.trim().parse::<f32>().ok()? * std::f32::consts::TAU)
        } else if let Some(gradians) = value.strip_suffix("grad") {
            Some(gradians.trim().parse::<f32>().ok()? * std::f32::consts::PI / 200.0)
        } else {
            None
        }
    }

    const fn identity() -> [f32; 20] {
        [
            1.0, 0.0, 0.0, 0.0, 0.0, // red
            0.0, 1.0, 0.0, 0.0, 0.0, // green
            0.0, 0.0, 1.0, 0.0, 0.0, // blue
            0.0, 0.0, 0.0, 1.0, 0.0, // alpha
        ]
    }

    fn diagonal(color: f32, alpha: f32) -> [f32; 20] {
        [
            color, 0.0, 0.0, 0.0, 0.0, 0.0, color, 0.0, 0.0, 0.0, 0.0, 0.0, color, 0.0, 0.0, 0.0,
            0.0, 0.0, alpha, 0.0,
        ]
    }

    fn contrast(amount: f32) -> [f32; 20] {
        let offset = 128.0 * (1.0 - amount);
        [
            amount, 0.0, 0.0, 0.0, offset, 0.0, amount, 0.0, 0.0, offset, 0.0, 0.0, amount, 0.0,
            offset, 0.0, 0.0, 0.0, 1.0, 0.0,
        ]
    }

    fn invert(amount: f32) -> [f32; 20] {
        let scale = 1.0 - 2.0 * amount;
        let offset = 255.0 * amount;
        [
            scale, 0.0, 0.0, 0.0, offset, 0.0, scale, 0.0, 0.0, offset, 0.0, 0.0, scale, 0.0,
            offset, 0.0, 0.0, 0.0, 1.0, 0.0,
        ]
    }

    fn saturate(amount: f32) -> [f32; 20] {
        let inverse = 1.0 - amount;
        let red = 0.213 * inverse;
        let green = 0.715 * inverse;
        let blue = 0.072 * inverse;
        [
            red + amount,
            green,
            blue,
            0.0,
            0.0,
            red,
            green + amount,
            blue,
            0.0,
            0.0,
            red,
            green,
            blue + amount,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
            0.0,
        ]
    }

    fn sepia(amount: f32) -> [f32; 20] {
        let inverse = 1.0 - amount;
        [
            inverse + 0.393 * amount,
            0.769 * amount,
            0.189 * amount,
            0.0,
            0.0,
            0.349 * amount,
            inverse + 0.686 * amount,
            0.168 * amount,
            0.0,
            0.0,
            0.272 * amount,
            0.534 * amount,
            inverse + 0.131 * amount,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
            0.0,
        ]
    }

    fn hue_rotate(angle: f32) -> [f32; 20] {
        let cosine = angle.cos();
        let sine = angle.sin();
        [
            0.213 + cosine * 0.787 - sine * 0.213,
            0.715 - cosine * 0.715 - sine * 0.715,
            0.072 - cosine * 0.072 + sine * 0.928,
            0.0,
            0.0,
            0.213 - cosine * 0.213 + sine * 0.143,
            0.715 + cosine * 0.285 + sine * 0.140,
            0.072 - cosine * 0.072 - sine * 0.283,
            0.0,
            0.0,
            0.213 - cosine * 0.213 - sine * 0.787,
            0.715 - cosine * 0.715 + sine * 0.715,
            0.072 + cosine * 0.928 + sine * 0.072,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
            0.0,
        ]
    }

    fn multiply(left: [f32; 20], right: [f32; 20]) -> [f32; 20] {
        let mut result = [0.0; 20];
        for row in 0..4 {
            for column in 0..4 {
                result[row * 5 + column] = (0..4)
                    .map(|index| left[row * 5 + index] * right[index * 5 + column])
                    .sum();
            }
            result[row * 5 + 4] = left[row * 5 + 4]
                + (0..4)
                    .map(|index| left[row * 5 + index] * right[index * 5 + 4])
                    .sum::<f32>();
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_filter_chain() {
        assert!(ParsedFilter::parse("blur(4px) saturate(120%) hue-rotate(90deg)").is_some());
        assert!(ParsedFilter::parse("blur(-1px)").is_none());
        assert!(ParsedFilter::parse("url(filter.svg)").is_none());
    }
}
