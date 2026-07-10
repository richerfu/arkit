//! Shared ECharts style resolution.

use crate::model::{BasicSeries, DataPoint, LabelStyle};

use super::geometry::color;

pub(super) fn item_color(
    series: &BasicSeries,
    point: Option<&DataPoint>,
    palette: &[u32],
    palette_index: usize,
) -> u32 {
    let style = point
        .and_then(|point| point.item_style.color)
        .or(series.options.item_style.color)
        .unwrap_or_else(|| color(palette, palette_index));
    let opacity = point
        .map(|point| point.item_style.opacity)
        .unwrap_or(series.options.item_style.opacity);
    with_opacity(style, opacity)
}

pub(super) fn line_color(series: &BasicSeries, palette: &[u32], palette_index: usize) -> u32 {
    let color = series
        .options
        .line_style
        .color
        .or(series.options.item_style.color)
        .unwrap_or_else(|| color(palette, palette_index));
    with_opacity(color, series.options.line_style.opacity)
}

pub(super) fn area_color(
    series: &BasicSeries,
    palette: &[u32],
    palette_index: usize,
) -> Option<u32> {
    let style = series.options.area_style.as_ref()?;
    let color = style
        .color
        .or(series.options.item_style.color)
        .unwrap_or_else(|| color(palette, palette_index));
    Some(with_opacity(color, style.opacity))
}

pub(super) fn border(series: &BasicSeries, point: Option<&DataPoint>) -> Option<(u32, f32)> {
    let point_style = point.map(|point| &point.item_style);
    let width = point_style
        .filter(|style| style.border_width > 0.0)
        .map(|style| style.border_width)
        .unwrap_or(series.options.item_style.border_width);
    let color = point_style
        .and_then(|style| style.border_color)
        .or(series.options.item_style.border_color)?;
    (width > 0.0).then_some((color, width))
}

pub(super) fn effective_label<'a>(series: &'a BasicSeries, point: &'a DataPoint) -> &'a LabelStyle {
    if point.label != LabelStyle::default() {
        &point.label
    } else {
        &series.options.label
    }
}

pub(super) fn format_label(
    style: &LabelStyle,
    series: &BasicSeries,
    point: &DataPoint,
    data_index: usize,
) -> String {
    let name = point.name.clone().unwrap_or_else(|| data_index.to_string());
    let value = point
        .values
        .iter()
        .map(|value| match value {
            crate::model::DataValue::Number(value) => format_number(*value),
            crate::model::DataValue::String(value) => value.clone(),
        })
        .collect::<Vec<_>>()
        .join(", ");
    style
        .formatter
        .as_deref()
        .unwrap_or("{c}")
        .replace("{a}", series.name.as_deref().unwrap_or_default())
        .replace("{b}", &name)
        .replace("{c}", &value)
}

pub(super) fn with_opacity(color: u32, opacity: f32) -> u32 {
    let alpha = ((color >> 24) & 0xFF) as f32;
    let alpha = (alpha * opacity.clamp(0.0, 1.0)).round() as u32;
    (color & 0x00FF_FFFF) | alpha << 24
}

pub(super) fn gradient_color(colors: &[u32], normalized: f64) -> u32 {
    match colors {
        [] => 0xFF5470C6,
        [color] => *color,
        colors => {
            let position = normalized.clamp(0.0, 1.0) * (colors.len() - 1) as f64;
            let left = position.floor() as usize;
            let right = (left + 1).min(colors.len() - 1);
            let t = (position - left as f64) as f32;
            interpolate_color(colors[left], colors[right], t)
        }
    }
}

fn interpolate_color(left: u32, right: u32, t: f32) -> u32 {
    let channel = |shift: u32| {
        let left = ((left >> shift) & 0xFF) as f32;
        let right = ((right >> shift) & 0xFF) as f32;
        (left + (right - left) * t).round() as u32
    };
    channel(24) << 24 | channel(16) << 16 | channel(8) << 8 | channel(0)
}

fn format_number(value: f64) -> String {
    if (value - value.round()).abs() < 1e-8 {
        format!("{value:.0}")
    } else {
        let mut value = format!("{value:.3}");
        while value.ends_with('0') {
            value.pop();
        }
        value.trim_end_matches('.').to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn css_alpha_is_multiplied_by_style_opacity() {
        assert_eq!(with_opacity(0x80FF0000, 0.5), 0x40FF0000);
    }

    #[test]
    fn gradient_interpolates_argb_channels() {
        assert_eq!(gradient_color(&[0xFF000000, 0xFFFFFFFF], 0.5), 0xFF808080);
    }
}
