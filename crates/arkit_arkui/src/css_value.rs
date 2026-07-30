//! CSS-oriented parsers for RSX attribute values.
//!
//! Enum-like attributes accept CSS keywords only (no raw ArkUI magic integers).
//! Lengths accept vp numbers and `"N%"` percentage strings.

use dioxus_core::AttributeValue;

/// Resolved length for layout geometry.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CssLength {
    /// Absolute size in vp (ArkUI default unit).
    Vp(f32),
    /// Fraction of the parent, `0.0..=1.0` (`"50%"` → `0.5`).
    Percent(f32),
}

/// Expand a CSS box shorthand into `[top, right, bottom, left]`.
///
/// Accepts 1–4 lengths separated by spaces and/or commas, optionally with
/// `px` / `vp` suffixes (treated as vp).
///
/// | input | result |
/// | ----- | ------ |
/// | `8` / `"8"` / `"8px"` | `[8,8,8,8]` |
/// | `"8 16"` | `[8,16,8,16]` |
/// | `"8 16 12"` | `[8,16,12,16]` |
/// | `"8 16 12 4"` | `[8,16,12,4]` |
pub fn expand_box_shorthand(value: &AttributeValue) -> Option<[f32; 4]> {
    match value {
        AttributeValue::Float(f) => Some([*f as f32; 4]),
        AttributeValue::Int(i) => Some([*i as f32; 4]),
        AttributeValue::Text(s) => {
            let parts = split_css_list(s);
            let nums: Option<Vec<f32>> = parts.into_iter().map(parse_vp_number).collect();
            let nums = nums?;
            match nums.as_slice() {
                [a] => Some([*a, *a, *a, *a]),
                [v, h] => Some([*v, *h, *v, *h]),
                [t, h, b] => Some([*t, *h, *b, *h]),
                [t, r, b, l] => Some([*t, *r, *b, *l]),
                _ => None,
            }
        }
        _ => None,
    }
}

/// Parse a length that may be absolute vp or a percentage.
pub fn parse_length(value: &AttributeValue) -> Option<CssLength> {
    match value {
        AttributeValue::Float(f) => Some(CssLength::Vp(*f as f32)),
        AttributeValue::Int(i) => Some(CssLength::Vp(*i as f32)),
        AttributeValue::Text(s) => {
            let s = s.trim();
            if let Some(pct) = s.strip_suffix('%') {
                let n: f32 = pct.trim().parse().ok()?;
                // CSS 100% → ArkUI 1.0; also accept already-normalized 0..=1.
                let frac = if n > 1.0 { n / 100.0 } else { n };
                Some(CssLength::Percent(frac.clamp(0.0, 1.0)))
            } else {
                parse_vp_number(s).map(CssLength::Vp)
            }
        }
        _ => None,
    }
}

/// Absolute length in vp (rejects percentages).
pub fn parse_vp(value: &AttributeValue) -> Option<f32> {
    match parse_length(value)? {
        CssLength::Vp(v) => Some(v),
        CssLength::Percent(_) => None,
    }
}

/// Color: hex (`#rgb` / `#rrggbb` / `#aarrggbb`) or opaque `0xAARRGGBB` int.
pub fn parse_css_color(value: &AttributeValue) -> Option<u32> {
    match value {
        AttributeValue::Int(i) => Some(*i as u32),
        AttributeValue::Text(s) => parse_hex_color(s),
        _ => None,
    }
}

fn parse_hex_color(s: &str) -> Option<u32> {
    let s = s.trim().trim_start_matches('#');
    match s.len() {
        3 => {
            // #rgb → #rrggbb
            let mut expanded = String::with_capacity(6);
            for ch in s.chars() {
                expanded.push(ch);
                expanded.push(ch);
            }
            u32::from_str_radix(&expanded, 16)
                .ok()
                .map(|v| 0xFF00_0000 | v)
        }
        4 => {
            // #argb nibble form → expand
            let chars: Vec<char> = s.chars().collect();
            let mut expanded = String::with_capacity(8);
            for ch in chars {
                expanded.push(ch);
                expanded.push(ch);
            }
            u32::from_str_radix(&expanded, 16).ok()
        }
        6 => u32::from_str_radix(s, 16).ok().map(|v| 0xFF00_0000 | v),
        8 => u32::from_str_radix(s, 16).ok(),
        _ => None,
    }
}

/// Strip units and parse a bare number (`"12px"`, `"12vp"`, `"12.5"`).
pub fn parse_vp_number(s: &str) -> Option<f32> {
    let s = s.trim().trim_matches('"').trim_matches('\'');
    let s = s
        .strip_suffix("px")
        .or_else(|| s.strip_suffix("PX"))
        .or_else(|| s.strip_suffix("vp"))
        .or_else(|| s.strip_suffix("VP"))
        .unwrap_or(s)
        .trim();
    s.parse().ok()
}

/// Split CSS lists on spaces and/or commas.
pub fn split_css_list(s: &str) -> Vec<&str> {
    s.split(|c: char| c.is_whitespace() || c == ',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect()
}

pub fn enum_token(s: &str) -> String {
    s.trim()
        .trim_matches('"')
        .trim_matches('\'')
        .to_ascii_lowercase()
        .replace('-', "_")
}

// --- Keyword maps (ArkUI enums) ---------------------------------------------

pub fn text_align_keyword(s: &str) -> Option<i32> {
    match enum_token(s).as_str() {
        "start" | "left" => Some(0),
        "center" => Some(1),
        "end" | "right" => Some(2),
        "justify" => Some(3),
        _ => None,
    }
}

pub fn visibility_keyword(s: &str) -> Option<i32> {
    match enum_token(s).as_str() {
        "visible" | "show" => Some(0),
        "hidden" | "invisible" => Some(1),
        "none" | "gone" => Some(2),
        _ => None,
    }
}

pub fn font_style_keyword(s: &str) -> Option<i32> {
    match enum_token(s).as_str() {
        "normal" => Some(0),
        "italic" | "oblique" => Some(1),
        _ => None,
    }
}

pub fn font_weight_keyword(s: &str) -> Option<i32> {
    // Return *CSS-like* weights (100–900); caller maps to ArkUI 0–8 index.
    match enum_token(s).as_str() {
        "thin" | "hairline" => Some(100),
        "extralight" | "ultralight" => Some(200),
        "light" => Some(300),
        "normal" | "regular" => Some(400),
        "medium" => Some(500),
        "semibold" | "demibold" => Some(600),
        "bold" => Some(700),
        "extrabold" | "ultrabold" => Some(800),
        "black" | "heavy" => Some(900),
        _ => None,
    }
}

pub fn border_style_keyword(s: &str) -> Option<i32> {
    match enum_token(s).as_str() {
        "solid" => Some(0),
        "dashed" => Some(1),
        "dotted" => Some(2),
        "none" => Some(0), // ArkUI has no none; solid + 0 width preferred by callers
        _ => None,
    }
}

pub fn text_overflow_keyword(s: &str) -> Option<i32> {
    match enum_token(s).as_str() {
        "none" => Some(0),
        "clip" => Some(1),
        "ellipsis" => Some(2),
        "marquee" => Some(3),
        _ => None,
    }
}

pub fn object_fit_value(s: &str) -> Option<i32> {
    match enum_token(s).as_str() {
        "contain" => Some(0),
        "cover" => Some(1),
        "auto" => Some(2),
        "fill" | "stretch" => Some(3),
        "scale_down" | "scaledown" => Some(4),
        "none" => Some(5),
        _ => None,
    }
}

pub fn scroll_bar_keyword(s: &str) -> Option<i32> {
    match enum_token(s).as_str() {
        "off" | "hidden" | "none" | "false" => Some(0),
        "auto" => Some(1),
        "on" | "visible" | "always" | "true" | "show" => Some(2),
        _ => None,
    }
}

pub fn text_decoration_keyword(s: &str) -> Option<i32> {
    match enum_token(s).as_str() {
        "none" => Some(0),
        "underline" => Some(1),
        "overline" => Some(2),
        "line_through" | "linethrough" | "strikethrough" | "strike" => Some(3),
        _ => None,
    }
}

/// Stack / absolute `alignment` (ArkUI Alignment 0–8).
pub fn alignment_keyword(s: &str) -> Option<i32> {
    match enum_token(s).as_str() {
        "top_start" | "top_left" | "start_top" | "left_top" => Some(0),
        "top" | "top_center" => Some(1),
        "top_end" | "top_right" | "end_top" | "right_top" => Some(2),
        "start" | "left" | "center_start" | "center_left" => Some(3),
        "center" | "middle" => Some(4),
        "end" | "right" | "center_end" | "center_right" => Some(5),
        "bottom_start" | "bottom_left" | "start_bottom" | "left_bottom" => Some(6),
        "bottom" | "bottom_center" => Some(7),
        "bottom_end" | "bottom_right" | "end_bottom" | "right_bottom" => Some(8),
        _ => None,
    }
}

pub fn hit_test_keyword(s: &str) -> Option<i32> {
    match enum_token(s).as_str() {
        "default" | "auto" => Some(0),
        "block" => Some(1),
        "transparent" => Some(2),
        "none" => Some(3),
        _ => None,
    }
}

pub fn scroll_edge_effect_keyword(s: &str) -> Option<i32> {
    match enum_token(s).as_str() {
        "spring" | "bounce" => Some(0),
        "fade" => Some(1),
        "none" | "hard" | "clamp" => Some(2),
        _ => None,
    }
}

/// Shadow style presets (ArkUI ShadowStyle).
pub fn shadow_keyword(s: &str) -> Option<i32> {
    match enum_token(s).as_str() {
        "none" | "off" | "false" => Some(-1), // special: caller may skip apply
        "xs" | "outer_default_xs" => Some(0),
        "sm" | "small" | "outer_default_sm" => Some(1),
        "md" | "medium" | "outer_default_md" => Some(2),
        "lg" | "large" | "outer_default_lg" => Some(3),
        "floating_sm" => Some(4),
        "floating_md" => Some(5),
        _ => None,
    }
}

pub fn input_type_keyword(s: &str) -> Option<i32> {
    match enum_token(s).as_str() {
        "text" | "normal" | "default" => Some(0),
        "number" | "numeric" | "tel_number" => Some(2),
        "phone" | "phone_number" | "tel" => Some(3),
        "email" => Some(5),
        "password" => Some(7),
        "number_password" | "numeric_password" => Some(8),
        "screen_lock_password" => Some(9),
        "user_name" | "username" => Some(10),
        "new_password" => Some(11),
        "number_decimal" | "decimal" => Some(12),
        "one_time_code" | "otp" | "verification_code" => Some(14), // if platform supports
        _ => None,
    }
}

pub fn progress_type_keyword(s: &str) -> Option<i32> {
    match enum_token(s).as_str() {
        "linear" | "bar" | "line" => Some(0),
        "ring" | "circle" => Some(1),
        "eclipse" | "arc" => Some(2),
        "scale_ring" | "scalering" => Some(3),
        "capsule" => Some(4),
        _ => None,
    }
}

pub fn list_sticky_keyword(s: &str) -> Option<i32> {
    match enum_token(s).as_str() {
        "none" | "off" | "false" => Some(0),
        "header" | "start" => Some(1),
        "footer" | "end" => Some(2),
        "both" | "all" | "true" => Some(3),
        _ => None,
    }
}

pub fn button_type_keyword(s: &str) -> Option<i32> {
    match enum_token(s).as_str() {
        "normal" | "default" | "rectangle" | "rect" => Some(0),
        "capsule" | "pill" | "rounded" => Some(1),
        "circle" | "round" => Some(2),
        _ => None,
    }
}

pub fn animation_curve_keyword(s: &str) -> Option<i32> {
    match enum_token(s).as_str() {
        "linear" => Some(0),
        "ease" => Some(1),
        "ease_in" | "easein" => Some(2),
        "ease_out" | "easeout" => Some(3),
        "ease_in_out" | "easeinout" => Some(4),
        "fast_out_slow_in" => Some(5),
        "linear_out_slow_in" => Some(6),
        "fast_out_linear_in" => Some(7),
        "extreme_deceleration" => Some(8),
        "sharp" => Some(9),
        "rhythm" => Some(10),
        "smooth" => Some(11),
        "friction" => Some(12),
        _ => None,
    }
}

/// Flex / row / column `align-items` style keywords (FlexOption path uses AlignSelf enum).
pub fn flex_align_items_keyword(s: &str) -> Option<i32> {
    match enum_token(s).as_str() {
        "auto" => Some(0),
        "start" | "flex_start" | "top" | "left" => Some(1),
        "center" => Some(2),
        "end" | "flex_end" | "bottom" | "right" => Some(3),
        "stretch" => Some(4),
        "baseline" => Some(5),
        _ => None,
    }
}

/// Opacity: 0..=1, or `"50%"` → 0.5.
pub fn parse_opacity(value: &AttributeValue) -> Option<f32> {
    match value {
        AttributeValue::Float(f) => Some((*f as f32).clamp(0.0, 1.0)),
        AttributeValue::Int(i) => {
            let v = *i as f32;
            Some(if v > 1.0 {
                (v / 100.0).clamp(0.0, 1.0)
            } else {
                v.clamp(0.0, 1.0)
            })
        }
        AttributeValue::Text(s) => {
            let s = s.trim();
            if let Some(pct) = s.strip_suffix('%') {
                let n: f32 = pct.trim().parse().ok()?;
                Some((n / 100.0).clamp(0.0, 1.0))
            } else {
                s.parse::<f32>().ok().map(|v| v.clamp(0.0, 1.0))
            }
        }
        _ => None,
    }
}

/// Map CSS font-weight (100–900 or 0–8) to ArkUI FontWeight index (0–8).
pub fn map_font_weight_to_arkui(raw: i32) -> i32 {
    if raw >= 100 {
        ((raw / 100).saturating_sub(1)).min(8)
    } else {
        raw.clamp(0, 8)
    }
}

/// Resolve enum attributes from CSS keywords only (no raw magic integers).
pub fn i32_or_keyword(
    value: &AttributeValue,
    keyword: impl Fn(&str) -> Option<i32>,
) -> Option<i32> {
    match value {
        AttributeValue::Text(s) => keyword(s),
        // Bool only for binary-ish maps that accept true/false in keyword()
        AttributeValue::Bool(b) => keyword(if *b { "true" } else { "false" }),
        _ => None,
    }
}

/// Font-weight: CSS keywords or CSS numeric weights (100–900). Rejects ArkUI indices.
pub fn font_weight_value(value: &AttributeValue) -> Option<i32> {
    match value {
        AttributeValue::Text(s) => font_weight_keyword(s).or_else(|| {
            s.trim()
                .parse::<i32>()
                .ok()
                .filter(|n| (100..=900).contains(n))
        }),
        AttributeValue::Int(i) => {
            let n = *i as i32;
            if (100..=900).contains(&n) {
                Some(n)
            } else {
                None
            }
        }
        AttributeValue::Float(f) => {
            let n = *f as i32;
            if (100..=900).contains(&n) {
                Some(n)
            } else {
                None
            }
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dioxus_core::AttributeValue;

    #[test]
    fn box_shorthand_css_rules() {
        assert_eq!(
            expand_box_shorthand(&AttributeValue::Float(8.0)),
            Some([8.0; 4])
        );
        assert_eq!(
            expand_box_shorthand(&AttributeValue::Text("8 16".into())),
            Some([8.0, 16.0, 8.0, 16.0])
        );
        assert_eq!(
            expand_box_shorthand(&AttributeValue::Text("8px, 16vp, 12, 4".into())),
            Some([8.0, 16.0, 12.0, 4.0])
        );
        assert_eq!(
            expand_box_shorthand(&AttributeValue::Text("8 16 12".into())),
            Some([8.0, 16.0, 12.0, 16.0])
        );
    }

    #[test]
    fn length_percent_and_units() {
        assert_eq!(
            parse_length(&AttributeValue::Text("50%".into())),
            Some(CssLength::Percent(0.5))
        );
        assert_eq!(
            parse_length(&AttributeValue::Text("100%".into())),
            Some(CssLength::Percent(1.0))
        );
        assert_eq!(
            parse_length(&AttributeValue::Text("12px".into())),
            Some(CssLength::Vp(12.0))
        );
        assert_eq!(
            parse_length(&AttributeValue::Float(24.0)),
            Some(CssLength::Vp(24.0))
        );
    }

    #[test]
    fn keywords_map_to_arkui() {
        assert_eq!(text_align_keyword("center"), Some(1));
        assert_eq!(visibility_keyword("hidden"), Some(1));
        assert_eq!(font_weight_keyword("bold"), Some(700));
        assert_eq!(map_font_weight_to_arkui(700), 6);
        assert_eq!(object_fit_value("cover"), Some(1));
        assert_eq!(scroll_bar_keyword("off"), Some(0));
        assert_eq!(border_style_keyword("dashed"), Some(1));
    }

    #[test]
    fn hex_colors() {
        assert_eq!(parse_hex_color("#fff"), Some(0xFFFF_FFFF));
        assert_eq!(parse_hex_color("#112233"), Some(0xFF11_2233));
        assert_eq!(parse_hex_color("#80112233"), Some(0x8011_2233));
    }
}
