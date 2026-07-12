//! Shared ECharts-style enter/update interpolation for every native renderer.

use std::time::{Duration, Instant};

use crate::model::*;

#[derive(Debug, Clone)]
pub(crate) struct ChartTransition {
    from: ChartOption,
    to: ChartOption,
    started: Instant,
    timing: AnimationTiming,
}

impl ChartTransition {
    pub(crate) fn initial(option: &ChartOption) -> Option<Self> {
        animation_allowed(option).then(|| Self {
            from: collapsed_option(option),
            to: option.clone(),
            started: Instant::now(),
            timing: option.animation.initial.clone(),
        })
    }

    pub(crate) fn update(from: &ChartOption, to: &ChartOption) -> Option<Self> {
        (animation_allowed(to) && from != to).then(|| Self {
            from: from.clone(),
            to: to.clone(),
            started: Instant::now(),
            timing: to.animation.update.clone(),
        })
    }

    pub(crate) fn state(from: &ChartOption, to: &ChartOption) -> Option<Self> {
        (to.animation.enabled && from != to && to.animation.state.duration > 0).then(|| Self {
            from: from.clone(),
            to: to.clone(),
            started: Instant::now(),
            timing: to.animation.state.clone(),
        })
    }

    pub(crate) fn snapshot(&self) -> (ChartOption, bool) {
        let elapsed = self.started.elapsed();
        let delay = Duration::from_millis(self.timing.delay);
        if elapsed <= delay {
            return (self.from.clone(), false);
        }
        if self.timing.duration == 0 {
            return (self.to.clone(), true);
        }
        let progress = ((elapsed - delay).as_secs_f64()
            / Duration::from_millis(self.timing.duration).as_secs_f64())
        .clamp(0.0, 1.0) as f32;
        let finished = progress >= 1.0;
        (
            interpolate_option(&self.from, &self.to, easing(&self.timing.easing, progress)),
            finished,
        )
    }
}

fn animation_allowed(option: &ChartOption) -> bool {
    option.animation.enabled
        && option
            .animation
            .initial
            .duration
            .max(option.animation.update.duration)
            > 0
        && data_count(option) <= option.animation.threshold
}

fn data_count(option: &ChartOption) -> usize {
    option
        .series
        .iter()
        .map(|series| match series {
            Series::Line(series)
            | Series::Bar(series)
            | Series::Pie(series)
            | Series::Scatter(series)
            | Series::EffectScatter(series)
            | Series::Radar(series)
            | Series::Gauge(series)
            | Series::Funnel(series)
            | Series::Heatmap(series)
            | Series::Candlestick(series)
            | Series::Boxplot(series)
            | Series::PictorialBar(series)
            | Series::Parallel(series)
            | Series::ThemeRiver(series)
            | Series::Treemap(series) => series.data.len(),
            Series::Tree(series) | Series::Graph(series) => series.nodes.len() + series.links.len(),
            Series::Sankey(series) => series.nodes.len() + series.links.len(),
            Series::Map(series) => series.features.len(),
            Series::Lines(series) => series.data.len(),
            Series::Sunburst(series) => count_sunburst_nodes(&series.data),
            Series::Custom(series) => series.data.len(),
        })
        .sum()
}

fn count_sunburst_nodes(nodes: &[SunburstNode]) -> usize {
    nodes
        .iter()
        .map(|node| 1 + count_sunburst_nodes(&node.children))
        .sum()
}

fn collapsed_option(option: &ChartOption) -> ChartOption {
    let mut collapsed = option.clone();
    for series in &mut collapsed.series {
        collapse_series(series);
    }
    collapsed
}

fn collapse_series(series: &mut Series) {
    match series {
        Series::Line(series) | Series::Bar(series) | Series::PictorialBar(series) => {
            for point in &mut series.data {
                collapse_point(point, point.values.len().saturating_sub(1));
            }
        }
        Series::Scatter(series) | Series::EffectScatter(series) => {
            for point in &mut series.data {
                collapse_point(point, usize::from(point.values.len() > 1));
            }
        }
        Series::Heatmap(series) => {
            for point in &mut series.data {
                collapse_point(point, point.values.len().saturating_sub(1));
            }
        }
        Series::Pie(series)
        | Series::Radar(series)
        | Series::Gauge(series)
        | Series::Funnel(series)
        | Series::Candlestick(series)
        | Series::Boxplot(series)
        | Series::Parallel(series)
        | Series::ThemeRiver(series)
        | Series::Treemap(series) => {
            for point in &mut series.data {
                for value in &mut point.values {
                    collapse_value(value);
                }
            }
        }
        Series::Tree(series) | Series::Graph(series) => {
            for node in &mut series.nodes {
                node.value = 0.0;
            }
        }
        Series::Sankey(series) => {
            for node in &mut series.nodes {
                node.value = 0.0;
            }
            for link in &mut series.links {
                link.value = 0.0;
            }
        }
        Series::Map(series) => {
            for feature in &mut series.features {
                if feature.value.is_some() {
                    feature.value = Some(0.0);
                }
            }
        }
        Series::Lines(series) => {
            for line in &mut series.data {
                let origin = line.coords.first().copied().unwrap_or(line.from);
                line.from = origin;
                line.to = origin;
                for point in &mut line.coords {
                    *point = origin;
                }
                line.value = 0.0;
            }
        }
        Series::Sunburst(series) => collapse_sunburst(&mut series.data),
        Series::Custom(_) => {}
    }
}

fn collapse_point(point: &mut DataPoint, dimension: usize) {
    if let Some(value) = point.values.get_mut(dimension) {
        collapse_value(value);
    }
}

fn collapse_value(value: &mut DataValue) {
    if matches!(value, DataValue::Number(_)) {
        *value = DataValue::Number(0.0);
    }
}

fn collapse_sunburst(nodes: &mut [SunburstNode]) {
    for node in nodes {
        node.value = 0.0;
        collapse_sunburst(&mut node.children);
    }
}

pub(crate) fn interpolate_option(
    from: &ChartOption,
    to: &ChartOption,
    progress: f32,
) -> ChartOption {
    let mut output = to.clone();
    for (index, target) in output.series.iter_mut().enumerate() {
        let Some(source) = matching_series(&from.series, target, index) else {
            let mut collapsed = target.clone();
            collapse_series(&mut collapsed);
            interpolate_series(&collapsed, target, progress);
            continue;
        };
        interpolate_series(source, target, progress);
    }
    output
}

fn matching_series<'a>(series: &'a [Series], target: &Series, index: usize) -> Option<&'a Series> {
    let target_id = series_id(target);
    target_id
        .and_then(|id| {
            series
                .iter()
                .find(|candidate| series_id(candidate) == Some(id))
        })
        .or_else(|| {
            let target_name = target.name()?;
            series
                .iter()
                .find(|candidate| candidate.name() == Some(target_name))
        })
        .or_else(|| series.get(index))
        .filter(|source| std::mem::discriminant(*source) == std::mem::discriminant(target))
}

fn series_id(series: &Series) -> Option<&str> {
    series_options(series)?
        .extra
        .get("id")
        .and_then(serde_json::Value::as_str)
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

fn interpolate_series(from: &Series, to: &mut Series, progress: f32) {
    match (from, to) {
        (Series::Line(from), Series::Line(to))
        | (Series::Bar(from), Series::Bar(to))
        | (Series::Pie(from), Series::Pie(to))
        | (Series::Scatter(from), Series::Scatter(to))
        | (Series::EffectScatter(from), Series::EffectScatter(to))
        | (Series::Radar(from), Series::Radar(to))
        | (Series::Gauge(from), Series::Gauge(to))
        | (Series::Funnel(from), Series::Funnel(to))
        | (Series::Heatmap(from), Series::Heatmap(to))
        | (Series::Candlestick(from), Series::Candlestick(to))
        | (Series::Boxplot(from), Series::Boxplot(to))
        | (Series::PictorialBar(from), Series::PictorialBar(to))
        | (Series::Parallel(from), Series::Parallel(to))
        | (Series::ThemeRiver(from), Series::ThemeRiver(to))
        | (Series::Treemap(from), Series::Treemap(to)) => {
            interpolate_basic_series(from, to, progress)
        }
        (Series::Tree(from), Series::Tree(to)) | (Series::Graph(from), Series::Graph(to)) => {
            interpolate_nodes(&from.nodes, &mut to.nodes, progress);
            interpolate_links(&from.links, &mut to.links, progress);
            interpolate_options(&from.options, &mut to.options, progress);
        }
        (Series::Sankey(from), Series::Sankey(to)) => {
            interpolate_nodes(&from.nodes, &mut to.nodes, progress);
            interpolate_links(&from.links, &mut to.links, progress);
            interpolate_options(&from.options, &mut to.options, progress);
        }
        (Series::Map(from), Series::Map(to)) => {
            for (index, target) in to.features.iter_mut().enumerate() {
                let source = from
                    .features
                    .iter()
                    .find(|source| source.name == target.name)
                    .or_else(|| from.features.get(index));
                if let (Some(source), Some(target_value)) = (source, target.value) {
                    target.value = Some(lerp(source.value.unwrap_or(0.0), target_value, progress));
                    target.item_style =
                        interpolate_item_style(&source.item_style, &target.item_style, progress);
                }
            }
            interpolate_options(&from.options, &mut to.options, progress);
        }
        (Series::Lines(from), Series::Lines(to)) => {
            for (index, target) in to.data.iter_mut().enumerate() {
                let Some(source) = from.data.get(index) else {
                    continue;
                };
                target.from = lerp_point(source.from, target.from, progress);
                target.to = lerp_point(source.to, target.to, progress);
                target.value = lerp(source.value, target.value, progress);
                for (point_index, point) in target.coords.iter_mut().enumerate() {
                    if let Some(source) = source.coords.get(point_index) {
                        *point = lerp_point(*source, *point, progress);
                    }
                }
            }
            interpolate_options(&from.options, &mut to.options, progress);
        }
        (Series::Sunburst(from), Series::Sunburst(to)) => {
            interpolate_sunburst(&from.data, &mut to.data, progress);
            interpolate_options(&from.options, &mut to.options, progress);
        }
        _ => {}
    }
}

fn interpolate_basic_series(from: &BasicSeries, to: &mut BasicSeries, progress: f32) {
    for (index, target) in to.data.iter_mut().enumerate() {
        let source = target
            .extra
            .get("id")
            .and_then(serde_json::Value::as_str)
            .and_then(|id| {
                from.data.iter().find(|point| {
                    point.extra.get("id").and_then(serde_json::Value::as_str) == Some(id)
                })
            })
            .or_else(|| {
                target.name.as_ref().and_then(|name| {
                    from.data
                        .iter()
                        .find(|point| point.name.as_ref() == Some(name))
                })
            })
            .or_else(|| from.data.get(index));
        let collapsed;
        let source = if let Some(source) = source {
            source
        } else {
            collapsed = {
                let mut point = target.clone();
                for value in &mut point.values {
                    collapse_value(value);
                }
                point
            };
            &collapsed
        };
        interpolate_point(source, target, progress);
    }
    interpolate_options(&from.options, &mut to.options, progress);
}

fn interpolate_point(from: &DataPoint, to: &mut DataPoint, progress: f32) {
    for (index, target) in to.values.iter_mut().enumerate() {
        let Some(target_number) = target.as_f64() else {
            continue;
        };
        let source = from
            .values
            .get(index)
            .and_then(DataValue::as_f64)
            .unwrap_or(0.0);
        *target = DataValue::Number(lerp(source, target_number, progress));
    }
    to.item_style = interpolate_item_style(&from.item_style, &to.item_style, progress);
    to.label = interpolate_label(&from.label, &to.label, progress);
}

fn interpolate_nodes(from: &[NodeData], to: &mut [NodeData], progress: f32) {
    for (index, target) in to.iter_mut().enumerate() {
        let source = from
            .iter()
            .find(|source| source.name == target.name)
            .or_else(|| from.get(index));
        let Some(source) = source else { continue };
        target.value = lerp(source.value, target.value, progress);
        target.x = interpolate_optional(source.x, target.x, progress);
        target.y = interpolate_optional(source.y, target.y, progress);
        target.symbol_size =
            interpolate_optional_f32(source.symbol_size, target.symbol_size, progress);
        target.item_style =
            interpolate_item_style(&source.item_style, &target.item_style, progress);
        target.label = interpolate_label(&source.label, &target.label, progress);
    }
}

fn interpolate_links(from: &[LinkData], to: &mut [LinkData], progress: f32) {
    for (index, target) in to.iter_mut().enumerate() {
        if let Some(source) = from.get(index) {
            target.value = lerp(source.value, target.value, progress);
        }
    }
}

fn interpolate_sunburst(from: &[SunburstNode], to: &mut [SunburstNode], progress: f32) {
    for (index, target) in to.iter_mut().enumerate() {
        let source = from
            .iter()
            .find(|source| source.name == target.name)
            .or_else(|| from.get(index));
        let Some(source) = source else { continue };
        target.value = lerp(source.value, target.value, progress);
        target.item_style =
            interpolate_item_style(&source.item_style, &target.item_style, progress);
        interpolate_sunburst(&source.children, &mut target.children, progress);
    }
}

fn interpolate_options(from: &SeriesOptions, to: &mut SeriesOptions, progress: f32) {
    to.item_style = interpolate_item_style(&from.item_style, &to.item_style, progress);
    to.line_style = interpolate_line_style(&from.line_style, &to.line_style, progress);
    to.label = interpolate_label(&from.label, &to.label, progress);
    to.symbol_size = lerp_f32(from.symbol_size, to.symbol_size, progress);
    to.symbol_rotate = lerp_f32(from.symbol_rotate, to.symbol_rotate, progress);
    if let (Some(from), Some(target)) = (&from.area_style, &to.area_style) {
        to.area_style = Some(interpolate_item_style(from, target, progress));
    }
}

fn interpolate_item_style(from: &ItemStyle, to: &ItemStyle, progress: f32) -> ItemStyle {
    let mut output = to.clone();
    output.color = interpolate_optional_color(from.color, to.color, progress);
    output.color0 = interpolate_optional_color(from.color0, to.color0, progress);
    output.border_color = interpolate_optional_color(from.border_color, to.border_color, progress);
    output.border_color0 =
        interpolate_optional_color(from.border_color0, to.border_color0, progress);
    output.border_width = lerp_f32(from.border_width, to.border_width, progress);
    output.opacity = lerp_f32(from.opacity, to.opacity, progress);
    for (index, value) in output.border_radius.iter_mut().enumerate() {
        *value = lerp_f32(from.border_radius[index], *value, progress);
    }
    output
}

fn interpolate_line_style(from: &LineStyle, to: &LineStyle, progress: f32) -> LineStyle {
    let mut output = to.clone();
    output.color = interpolate_optional_color(from.color, to.color, progress);
    output.width = lerp_f32(from.width, to.width, progress);
    output.opacity = lerp_f32(from.opacity, to.opacity, progress);
    output
}

fn interpolate_label(from: &LabelStyle, to: &LabelStyle, progress: f32) -> LabelStyle {
    let mut output = to.clone();
    output.color = interpolate_optional_color(from.color, to.color, progress);
    output.font_size = lerp_f32(from.font_size, to.font_size, progress);
    output.distance = lerp_f32(from.distance, to.distance, progress);
    output.rotate = lerp_f32(from.rotate, to.rotate, progress);
    output.offset = [
        lerp_f32(from.offset[0], to.offset[0], progress),
        lerp_f32(from.offset[1], to.offset[1], progress),
    ];
    output
}

fn interpolate_optional(from: Option<f64>, to: Option<f64>, progress: f32) -> Option<f64> {
    to.map(|target| lerp(from.unwrap_or(target), target, progress))
}

fn interpolate_optional_f32(from: Option<f32>, to: Option<f32>, progress: f32) -> Option<f32> {
    to.map(|target| lerp_f32(from.unwrap_or(target), target, progress))
}

fn interpolate_optional_color(from: Option<u32>, to: Option<u32>, progress: f32) -> Option<u32> {
    to.map(|target| interpolate_color(from.unwrap_or(target), target, progress))
}

fn interpolate_color(from: u32, to: u32, progress: f32) -> u32 {
    let channel = |shift: u32| {
        lerp_f32(
            ((from >> shift) & 0xFF_u32) as f32,
            ((to >> shift) & 0xFF_u32) as f32,
            progress,
        )
        .round() as u32
    };
    channel(24) << 24 | channel(16) << 16 | channel(8) << 8 | channel(0)
}

fn lerp(from: f64, to: f64, progress: f32) -> f64 {
    from + (to - from) * progress as f64
}

fn lerp_f32(from: f32, to: f32, progress: f32) -> f32 {
    from + (to - from) * progress
}

fn lerp_point(from: (f64, f64), to: (f64, f64), progress: f32) -> (f64, f64) {
    (lerp(from.0, to.0, progress), lerp(from.1, to.1, progress))
}

pub(crate) fn easing(name: &str, t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    match name {
        "quadraticIn" => t * t,
        "quadraticOut" => t * (2.0 - t),
        "quadraticInOut" => {
            if t < 0.5 {
                2.0 * t * t
            } else {
                -1.0 + (4.0 - 2.0 * t) * t
            }
        }
        "cubicIn" => t * t * t,
        "cubicOut" => 1.0 - (1.0 - t).powi(3),
        "cubicInOut" => {
            if t < 0.5 {
                4.0 * t.powi(3)
            } else {
                1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
            }
        }
        "quarticIn" => t.powi(4),
        "quarticOut" => 1.0 - (1.0 - t).powi(4),
        "quinticIn" => t.powi(5),
        "quinticOut" => 1.0 - (1.0 - t).powi(5),
        "sinusoidalIn" => 1.0 - (t * std::f32::consts::FRAC_PI_2).cos(),
        "sinusoidalOut" => (t * std::f32::consts::FRAC_PI_2).sin(),
        "circularIn" => 1.0 - (1.0 - t * t).sqrt(),
        "circularOut" => (1.0 - (t - 1.0).powi(2)).sqrt(),
        "exponentialIn" => {
            if t == 0.0 {
                0.0
            } else {
                2.0_f32.powf(10.0 * t - 10.0)
            }
        }
        "exponentialOut" => {
            if t == 1.0 {
                1.0
            } else {
                1.0 - 2.0_f32.powf(-10.0 * t)
            }
        }
        "bounceOut" => bounce_out(t),
        "bounceIn" => 1.0 - bounce_out(1.0 - t),
        "backIn" => 2.70158 * t.powi(3) - 1.70158 * t * t,
        "backOut" => 1.0 + 2.70158 * (t - 1.0).powi(3) + 1.70158 * (t - 1.0).powi(2),
        _ => t,
    }
}

fn bounce_out(mut t: f32) -> f32 {
    const N: f32 = 7.5625;
    const D: f32 = 2.75;
    if t < 1.0 / D {
        N * t * t
    } else if t < 2.0 / D {
        t -= 1.5 / D;
        N * t * t + 0.75
    } else if t < 2.5 / D {
        t -= 2.25 / D;
        N * t * t + 0.9375
    } else {
        t -= 2.625 / D;
        N * t * t + 0.984375
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interpolates_basic_series_by_name() {
        let from = ChartOption::new().push_series(Series::bar("Revenue", [10.0, 20.0]));
        let to = ChartOption::new().push_series(Series::bar("Revenue", [30.0, 60.0]));
        let snapshot = interpolate_option(&from, &to, 0.5);
        let Series::Bar(series) = &snapshot.series[0] else {
            panic!("bar")
        };
        assert_eq!(series.data[0].number_opt(0), Some(20.0));
        assert_eq!(series.data[1].number_opt(0), Some(40.0));
    }

    #[test]
    fn initial_snapshot_collapses_bar_values() {
        let option = ChartOption::new().push_series(Series::bar("Revenue", [30.0]));
        let collapsed = collapsed_option(&option);
        let Series::Bar(series) = &collapsed.series[0] else {
            panic!("bar")
        };
        assert_eq!(series.data[0].number_opt(0), Some(0.0));
    }

    #[test]
    fn cubic_easing_matches_endpoints() {
        assert_eq!(easing("cubicInOut", 0.0), 0.0);
        assert_eq!(easing("cubicInOut", 1.0), 1.0);
        assert!((easing("cubicOut", 0.5) - 0.875).abs() < 1e-6);
    }
}
