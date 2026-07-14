//! Shared ECharts style resolution.

use crate::model::{
    BasicSeries, DataPoint, ItemStyle, LabelStyle, Series, SeriesOptions, VisualMap,
};

use super::geometry::color;

pub(super) fn visual_map_color(visual_map: &VisualMap, value: f64) -> u32 {
    visual_map
        .pieces
        .iter()
        .find(|piece| piece.contains(value))
        .and_then(|piece| piece.color)
        .unwrap_or_else(|| {
            let normalized =
                (value - visual_map.min) / (visual_map.max - visual_map.min).max(1e-12);
            gradient_color(&visual_map.colors, normalized)
        })
}

pub(super) fn visual_map_symbol_size(visual_map: &VisualMap, value: f64) -> Option<[f32; 2]> {
    if let Some(size) = visual_map
        .pieces
        .iter()
        .find(|piece| piece.contains(value))
        .and_then(|piece| piece.symbol_size)
    {
        return Some([size.max(0.0); 2]);
    }
    visual_map.symbol_size_range.map(|[min, max]| {
        let normalized = ((value - visual_map.min) / (visual_map.max - visual_map.min).max(1e-12))
            .clamp(0.0, 1.0) as f32;
        let size = min + (max - min) * normalized;
        [size, size]
    })
}

pub(super) fn item_color(
    series: &BasicSeries,
    point: Option<&DataPoint>,
    palette: &[u32],
    palette_index: usize,
) -> u32 {
    let style = effective_item_style(series, point);
    let color = style.color.unwrap_or_else(|| color(palette, palette_index));
    with_opacity(color, style.opacity)
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

pub(super) fn legend_color(series: &Series, palette: &[u32], palette_index: usize) -> u32 {
    if matches!(series, Series::Candlestick(_)) {
        return 0xFFEC0000;
    }
    let Some(options) = series_options(series) else {
        return color(palette, palette_index);
    };
    let uses_line_color = matches!(
        series,
        Series::Line(_) | Series::Lines(_) | Series::Parallel(_) | Series::ThemeRiver(_)
    );
    if uses_line_color {
        with_opacity(
            options
                .line_style
                .color
                .or(options.item_style.color)
                .unwrap_or_else(|| color(palette, palette_index)),
            options.line_style.opacity,
        )
    } else {
        with_opacity(
            options
                .item_style
                .color
                .unwrap_or_else(|| color(palette, palette_index)),
            options.item_style.opacity,
        )
    }
}

fn series_options(series: &Series) -> Option<&SeriesOptions> {
    Some(match series {
        Series::Line(value)
        | Series::Bar(value)
        | Series::Pie(value)
        | Series::Scatter(value)
        | Series::EffectScatter(value)
        | Series::Radar(value)
        | Series::Gauge(value)
        | Series::Funnel(value)
        | Series::Heatmap(value)
        | Series::Candlestick(value)
        | Series::Boxplot(value)
        | Series::PictorialBar(value)
        | Series::Parallel(value)
        | Series::ThemeRiver(value)
        | Series::Treemap(value) => &value.options,
        Series::Tree(value) | Series::Graph(value) => &value.options,
        Series::Sankey(value) => &value.options,
        Series::Map(value) => &value.options,
        Series::Lines(value) => &value.options,
        Series::Sunburst(value) => &value.options,
        Series::Custom(_) => return None,
    })
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
    let style = effective_item_style(series, point);
    (style.border_width > 0.0).then_some((style.border_color?, style.border_width))
}

pub(super) fn effective_item_style(series: &BasicSeries, point: Option<&DataPoint>) -> ItemStyle {
    point
        .map(|point| merge_item_style(&series.options.item_style, &point.item_style))
        .unwrap_or_else(|| series.options.item_style.clone())
}

pub(super) fn effective_label(series: &BasicSeries, point: &DataPoint) -> LabelStyle {
    merge_label_style(&series.options.label, &point.label)
}

pub(super) fn merge_item_style(base: &ItemStyle, override_style: &ItemStyle) -> ItemStyle {
    let default = ItemStyle::default();
    ItemStyle {
        color: if override_style.specified.contains("color") {
            override_style.color
        } else {
            override_style.color.or(base.color)
        },
        color0: if override_style.specified.contains("color0") {
            override_style.color0
        } else {
            override_style.color0.or(base.color0)
        },
        border_color: if override_style.specified.contains("borderColor") {
            override_style.border_color
        } else {
            override_style.border_color.or(base.border_color)
        },
        border_color0: if override_style.specified.contains("borderColor0") {
            override_style.border_color0
        } else {
            override_style.border_color0.or(base.border_color0)
        },
        border_width: if override_style.specified.contains("borderWidth")
            || override_style.border_width != default.border_width
        {
            override_style.border_width
        } else {
            base.border_width
        },
        border_radius: if override_style.specified.contains("borderRadius")
            || override_style.border_radius != default.border_radius
        {
            override_style.border_radius
        } else {
            base.border_radius
        },
        opacity: if override_style.specified.contains("opacity")
            || override_style.opacity != default.opacity
        {
            override_style.opacity
        } else {
            base.opacity
        },
        specified: {
            let mut specified = base.specified.clone();
            specified.extend(override_style.specified.iter().cloned());
            specified
        },
    }
}

pub(super) fn merge_label_style(base: &LabelStyle, override_style: &LabelStyle) -> LabelStyle {
    let default = LabelStyle::default();
    LabelStyle {
        show: if override_style.specified.contains("show") || override_style.show != default.show {
            override_style.show
        } else {
            base.show
        },
        color: if override_style.specified.contains("color") {
            override_style.color
        } else {
            override_style.color.or(base.color)
        },
        font_size: if override_style.specified.contains("fontSize")
            || override_style.font_size != default.font_size
        {
            override_style.font_size
        } else {
            base.font_size
        },
        font_weight: if override_style.specified.contains("fontWeight")
            || override_style.font_weight != default.font_weight
        {
            override_style.font_weight
        } else {
            base.font_weight
        },
        position: if override_style.specified.contains("position")
            || override_style.position != default.position
        {
            override_style.position.clone()
        } else {
            base.position.clone()
        },
        distance: if override_style.specified.contains("distance")
            || override_style.distance != default.distance
        {
            override_style.distance
        } else {
            base.distance
        },
        rotate: if override_style.specified.contains("rotate")
            || override_style.rotate != default.rotate
        {
            override_style.rotate
        } else {
            base.rotate
        },
        offset: if override_style.specified.contains("offset")
            || override_style.offset != default.offset
        {
            override_style.offset
        } else {
            base.offset
        },
        formatter: if override_style.specified.contains("formatter") {
            override_style.formatter.clone()
        } else {
            override_style
                .formatter
                .clone()
                .or_else(|| base.formatter.clone())
        },
        specified: {
            let mut specified = base.specified.clone();
            specified.extend(override_style.specified.iter().cloned());
            specified
        },
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
            crate::model::DataValue::Null => String::from("-"),
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

    #[test]
    fn legend_uses_explicit_line_color() {
        let mut series = Series::line("line", [1.0]);
        let Series::Line(value) = &mut series else {
            unreachable!();
        };
        value.options.line_style.color = Some(0xFFF97316);
        assert_eq!(legend_color(&series, &[0xFF5470C6], 0), 0xFFF97316);
    }
}
