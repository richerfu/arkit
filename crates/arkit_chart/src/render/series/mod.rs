//! Per-series composition layer. Each renderer owns one chart type and uses
//! only shared geometry, hit-region, and canvas atoms.

mod bar;
mod boxplot;
mod candlestick;
mod custom;
mod funnel;
mod gauge;
mod graph;
mod heatmap;
mod line;
mod lines;
mod map;
mod parallel;
mod pictorial_bar;
mod pie;
mod radar;
mod sankey;
mod scatter;
mod sunburst;
mod theme_river;
mod tree;
mod treemap;

use std::collections::BTreeMap;

use ohos_drawing_binding::Canvas;

use super::geometry::Plot;
use super::hit::HitRegion;
use super::scale::CartesianLayout;
use crate::model::*;

pub(super) struct CartesianRenderContext<'a> {
    pub(super) series_index: usize,
    pub(super) plot: &'a Plot,
    pub(super) layout: &'a CartesianLayout,
    pub(super) bar_layout: Option<BarLayout>,
    pub(super) stack: Option<&'a [(f64, f64)]>,
    pub(super) visual_map: Option<&'a VisualMap>,
    pub(super) palette: &'a [u32],
    pub(super) canvas: Option<&'a Canvas>,
    pub(super) hits: &'a mut Vec<HitRegion>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct BarLayout {
    /// Offset from the category center to the leading edge of the bar.
    pub(super) offset: f32,
    pub(super) width: f32,
}

pub(super) struct FreeRenderContext<'a> {
    pub(super) series_index: usize,
    pub(super) option: &'a ChartOption,
    pub(super) plot: Plot,
    pub(super) palette: &'a [u32],
    pub(super) canvas: Option<&'a Canvas>,
    pub(super) hits: &'a mut Vec<HitRegion>,
    pub(super) selected: Option<&'a ChartEvent>,
    pub(super) selected_items: &'a std::collections::BTreeSet<(usize, usize)>,
}

pub(super) fn is_cartesian(series: &Series) -> bool {
    match series {
        Series::Line(series) | Series::Bar(series) => {
            series
                .options
                .extra
                .get("coordinateSystem")
                .and_then(serde_json::Value::as_str)
                != Some("polar")
        }
        Series::Scatter(series) | Series::EffectScatter(series) => !matches!(
            series
                .options
                .extra
                .get("coordinateSystem")
                .and_then(serde_json::Value::as_str),
            Some("polar" | "singleAxis" | "geo")
        ),
        Series::Heatmap(series) => !matches!(
            series
                .options
                .extra
                .get("coordinateSystem")
                .and_then(serde_json::Value::as_str),
            Some("calendar" | "geo")
        ),
        Series::Candlestick(_) | Series::Boxplot(_) | Series::PictorialBar(_) => true,
        _ => false,
    }
}

pub(super) fn cartesian_axis_indices(series: &Series) -> (usize, usize) {
    let options = match series {
        Series::Line(value)
        | Series::Bar(value)
        | Series::Scatter(value)
        | Series::EffectScatter(value)
        | Series::Heatmap(value)
        | Series::Candlestick(value)
        | Series::Boxplot(value)
        | Series::PictorialBar(value) => &value.options.extra,
        _ => return (0, 0),
    };
    let index = |key: &str| {
        options
            .get(key)
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0) as usize
    };
    (index("xAxisIndex"), index("yAxisIndex"))
}

pub(super) fn geo_index(series: &Series) -> Option<usize> {
    let extra = match series {
        Series::Scatter(value) | Series::EffectScatter(value) | Series::Heatmap(value) => {
            &value.options.extra
        }
        Series::Graph(value) => &value.options.extra,
        Series::Lines(value) => &value.options.extra,
        _ => return None,
    };
    (extra
        .get("coordinateSystem")
        .and_then(serde_json::Value::as_str)
        == Some("geo"))
    .then(|| {
        extra
            .get("geoIndex")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0) as usize
    })
}

pub(super) fn should_draw_geo_base(
    option: &ChartOption,
    series_index: usize,
    geo_index: usize,
) -> bool {
    !option.series[..series_index]
        .iter()
        .any(|series| self::geo_index(series) == Some(geo_index))
}

pub(super) fn data(series: &Series) -> &[DataPoint] {
    match series {
        Series::Line(v)
        | Series::Bar(v)
        | Series::Pie(v)
        | Series::Scatter(v)
        | Series::EffectScatter(v)
        | Series::Radar(v)
        | Series::Gauge(v)
        | Series::Funnel(v)
        | Series::Heatmap(v)
        | Series::Candlestick(v)
        | Series::Boxplot(v)
        | Series::PictorialBar(v)
        | Series::Parallel(v)
        | Series::ThemeRiver(v)
        | Series::Treemap(v) => &v.data,
        Series::Custom(v) => &v.data,
        Series::Tree(_)
        | Series::Graph(_)
        | Series::Sankey(_)
        | Series::Map(_)
        | Series::Lines(_)
        | Series::Sunburst(_) => &[],
    }
}

pub(super) fn render_cartesian_set(
    option: &ChartOption,
    series_indices: &[usize],
    plot: &Plot,
    layout: &CartesianLayout,
    palette: &[u32],
    canvas: Option<&Canvas>,
    hits: &mut Vec<HitRegion>,
) {
    let category_vertical = layout.y.is_category() && !layout.x.is_category();
    let stacks = stack_layouts(option, series_indices, category_vertical);
    let fallback_count = series_indices
        .iter()
        .map(|index| data(&option.series[*index]).len())
        .max()
        .unwrap_or(1);
    let slot = if category_vertical {
        layout.y.band_width(plot, true, fallback_count)
    } else {
        layout.x.band_width(plot, false, fallback_count)
    };
    let bar_layouts = bar_layouts(option, series_indices, slot);
    let mut render_order = series_indices.to_vec();
    render_order.sort_by(|left, right| {
        cartesian_z(&option.series[*left])
            .total_cmp(&cartesian_z(&option.series[*right]))
            .then_with(|| left.cmp(right))
    });
    for series_index in &render_order {
        let bar_layout = bar_layouts.get(series_index).copied();
        let mut context = CartesianRenderContext {
            series_index: *series_index,
            plot,
            layout,
            bar_layout,
            stack: stacks.get(series_index).map(Vec::as_slice),
            visual_map: option.visual_map_for_series(*series_index),
            palette,
            canvas,
            hits,
        };
        super::label_layout::set_policy(
            label_layout_options(&option.series[*series_index]),
            *series_index,
        );
        match &option.series[*series_index] {
            Series::Line(value) => line::render(value, &mut context),
            Series::Bar(value) => {
                bar::render(value, &mut context);
            }
            Series::Scatter(value) => scatter::render(value, &mut context),
            Series::EffectScatter(value) => scatter::render_effect(value, &mut context),
            Series::Heatmap(value) => heatmap::render(value, &mut context),
            Series::Candlestick(value) => candlestick::render(value, &mut context),
            Series::Boxplot(value) => boxplot::render(value, &mut context),
            Series::PictorialBar(value) => pictorial_bar::render(value, &mut context),
            _ => {}
        }
        super::marker::render(&option.series[*series_index], &mut context);
        super::label_layout::clear_policy();
    }
}

fn cartesian_z(series: &Series) -> f64 {
    let default = match series {
        Series::Line(_) | Series::Scatter(_) | Series::EffectScatter(_) => 3.0,
        _ => 2.0,
    };
    let options = match series {
        Series::Line(value)
        | Series::Bar(value)
        | Series::Scatter(value)
        | Series::EffectScatter(value)
        | Series::Heatmap(value)
        | Series::Candlestick(value)
        | Series::Boxplot(value)
        | Series::PictorialBar(value) => &value.options.extra,
        _ => return default,
    };
    options
        .get("z")
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(default)
}

fn stack_layouts(
    option: &ChartOption,
    series_indices: &[usize],
    horizontal_bar: bool,
) -> BTreeMap<usize, Vec<(f64, f64)>> {
    let mut output = BTreeMap::new();
    let mut accumulators: BTreeMap<String, Vec<(f64, f64)>> = BTreeMap::new();
    for series_index in series_indices {
        let (data, stack, horizontal) = match &option.series[*series_index] {
            Series::Line(series) => (&series.data, series.options.stack.as_deref(), false),
            Series::Bar(series) => (
                &series.data,
                series.options.stack.as_deref(),
                horizontal_bar,
            ),
            _ => continue,
        };
        let Some(stack) = stack else { continue };
        let accumulator = accumulators
            .entry(stack.to_string())
            .or_insert_with(|| vec![(0.0, 0.0); data.len()]);
        accumulator.resize(data.len(), (0.0, 0.0));
        let mut values = Vec::with_capacity(data.len());
        for (index, point) in data.iter().enumerate() {
            let value = if point.values.len() > 1 && !horizontal {
                point.number_opt(1)
            } else {
                point.number_opt(0)
            };
            let Some(value) = value else {
                values.push((0.0, 0.0));
                continue;
            };
            let base = if value >= 0.0 {
                let base = accumulator[index].0;
                accumulator[index].0 += value;
                base
            } else {
                let base = accumulator[index].1;
                accumulator[index].1 += value;
                base
            };
            values.push((base, base + value));
        }
        output.insert(*series_index, values);
    }
    output
}

fn bar_layouts(
    option: &ChartOption,
    series_indices: &[usize],
    slot: f32,
) -> BTreeMap<usize, BarLayout> {
    let mut groups: Vec<(String, Vec<usize>)> = Vec::new();
    let mut last_options = None;
    for series_index in series_indices {
        let Some(options) = bar_like_options(&option.series[*series_index]) else {
            continue;
        };
        last_options = Some(options);
        let key = options
            .stack
            .clone()
            .unwrap_or_else(|| format!("__series_{series_index}"));
        if let Some((_, indices)) = groups.iter_mut().find(|(group, _)| group == &key) {
            indices.push(*series_index);
        } else {
            groups.push((key, vec![*series_index]));
        }
    }
    let Some(last_options) = last_options else {
        return BTreeMap::new();
    };

    let slot = slot.max(1.0);
    let category_gap = last_options
        .bar_category_gap
        .resolve(slot)
        .clamp(0.0, slot * 0.95);
    let available = (slot - category_gap).max(1.0);
    let gap_ratio = last_options.bar_gap.max(-1.0);
    let group_count = groups.len().max(1);
    let mut widths = Vec::with_capacity(group_count);
    let mut fixed_width = 0.0;
    let mut auto_count = 0usize;
    for (_, indices) in &groups {
        let options =
            bar_like_options(&option.series[*indices.last().expect("bar group is non-empty")])
                .expect("bar group contains bar-like series");
        let min = options
            .bar_min_width
            .map(|width| width.resolve(slot))
            .unwrap_or(1.0)
            .max(0.0);
        let max = options
            .bar_max_width
            .map(|width| width.resolve(slot))
            .unwrap_or(available)
            .max(min);
        let width = options
            .bar_width
            .map(|width| width.resolve(slot).clamp(min, max));
        if let Some(width) = width {
            fixed_width += width;
        } else {
            auto_count += 1;
        }
        widths.push((width, min, max));
    }

    let gap_slots = group_count.saturating_sub(1) as f32;
    let denominator = (auto_count as f32 + gap_ratio * gap_slots).max(0.05);
    let auto_width = ((available - fixed_width).max(1.0) / denominator).max(1.0);
    let mut resolved_widths = widths
        .into_iter()
        .map(|(width, min, max)| width.unwrap_or(auto_width.clamp(min, max)))
        .collect::<Vec<_>>();
    if resolved_widths.is_empty() {
        resolved_widths.push(available);
    }
    let gap_reference = if auto_count > 0 {
        auto_width
    } else {
        resolved_widths.iter().sum::<f32>() / resolved_widths.len() as f32
    };
    let gap = gap_reference * gap_ratio;
    let total = resolved_widths.iter().sum::<f32>() + gap * gap_slots;
    let mut cursor = -total / 2.0;
    let mut output = BTreeMap::new();
    for ((_, indices), width) in groups.into_iter().zip(resolved_widths) {
        let layout = BarLayout {
            offset: cursor,
            width,
        };
        for index in indices {
            output.insert(index, layout);
        }
        cursor += width + gap;
    }
    output
}

fn bar_like_options(series: &Series) -> Option<&SeriesOptions> {
    match series {
        Series::Bar(series) | Series::PictorialBar(series) => Some(&series.options),
        _ => None,
    }
}

pub(super) fn render_free(
    option: &ChartOption,
    series_index: usize,
    series: &Series,
    plot: Plot,
    palette: &[u32],
    canvas: Option<&Canvas>,
    hits: &mut Vec<HitRegion>,
    selected: Option<&ChartEvent>,
    selected_items: &std::collections::BTreeSet<(usize, usize)>,
) {
    let mut context = FreeRenderContext {
        series_index,
        option,
        plot,
        palette,
        canvas,
        hits,
        selected,
        selected_items,
    };
    if let Some(policy) = label_layout_options_optional(series) {
        super::label_layout::set_policy(policy, series_index);
    } else {
        super::label_layout::clear_policy();
    }
    match series {
        Series::Line(value) => line::render_polar(value, &mut context),
        Series::Bar(value) => bar::render_polar(value, &mut context),
        Series::Scatter(value) => scatter::render_free(value, &mut context, false),
        Series::EffectScatter(value) => scatter::render_free(value, &mut context, true),
        Series::Pie(value) => pie::render(value, &mut context),
        Series::Radar(value) => radar::render(value, &mut context),
        Series::Gauge(value) => gauge::render(value, &mut context),
        Series::Heatmap(value) => {
            if geo_index(series).is_some() {
                heatmap::render_geo(value, &mut context);
            } else {
                heatmap::render_calendar(value, &mut context);
            }
        }
        Series::Funnel(value) => funnel::render(value, &mut context),
        Series::Tree(value) => tree::render(value, &mut context),
        Series::Treemap(value) => treemap::render(value, &mut context),
        Series::Graph(value) => graph::render(value, &mut context),
        Series::Sankey(value) => sankey::render(value, &mut context),
        Series::Map(value) => map::render(value, &mut context),
        Series::Lines(value) => lines::render(value, &mut context),
        Series::Sunburst(value) => sunburst::render(value, &mut context),
        Series::Parallel(value) => parallel::render(value, &mut context),
        Series::ThemeRiver(value) => theme_river::render(value, &mut context),
        Series::Custom(value) => custom::render(value, &mut context),
        Series::Candlestick(_) | Series::Boxplot(_) | Series::PictorialBar(_) => {}
    }
    super::label_layout::clear_policy();
}

fn label_layout_options(series: &Series) -> &LabelLayoutOptions {
    label_layout_options_optional(series).expect("non-custom series has labelLayout")
}

fn label_layout_options_optional(series: &Series) -> Option<&LabelLayoutOptions> {
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
        | Series::Treemap(value) => &value.options.label_layout,
        Series::Tree(value) | Series::Graph(value) => &value.options.label_layout,
        Series::Sankey(value) => &value.options.label_layout,
        Series::Map(value) => &value.options.label_layout,
        Series::Lines(value) => &value.options.label_layout,
        Series::Sunburst(value) => &value.options.label_layout,
        Series::Custom(_) => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stacked_bars_share_layout_and_gap_options_center_groups() {
        let stacked = |name: &str| {
            let mut series = Series::bar(name, [1.0]);
            let Series::Bar(value) = &mut series else {
                unreachable!();
            };
            value.options.stack = Some(String::from("total"));
            value.options.bar_width = Some(Length::Percent(30.0));
            series
        };
        let mut adjacent = Series::bar("adjacent", [1.0]);
        let Series::Bar(value) = &mut adjacent else {
            unreachable!();
        };
        value.options.bar_gap = 0.25;
        value.options.bar_category_gap = Length::Percent(20.0);
        let option = ChartOption::new()
            .push_series(stacked("first"))
            .push_series(stacked("second"))
            .push_series(adjacent);

        let layouts = bar_layouts(&option, &[0, 1, 2], 100.0);
        assert_eq!(layouts[&0], layouts[&1]);
        assert_eq!(layouts[&0].width, 30.0);
        assert_eq!(layouts[&0].offset, -40.0);
        assert_eq!(layouts[&2].width, 40.0);
        assert_eq!(layouts[&2].offset, 0.0);
    }

    #[test]
    fn polar_bar_and_scatter_are_dispatched_as_free_series() {
        let option = ChartOption::from_json_str(
            r#"{"series":[
                {"type":"bar","coordinateSystem":"polar","data":[1]},
                {"type":"scatter","coordinateSystem":"polar","data":[[1,0]]},
                {"type":"effectScatter","coordinateSystem":"polar","data":[[1,0]]}
            ]}"#,
        )
        .unwrap();
        assert!(option.series.iter().all(|series| !is_cartesian(series)));
    }

    #[test]
    fn single_axis_scatter_is_dispatched_as_free_series() {
        let option = ChartOption::from_json_str(
            r#"{"singleAxis":{},"series":[
                {"type":"scatter","coordinateSystem":"singleAxis","data":[1,2]},
                {"type":"effectScatter","coordinateSystem":"singleAxis","data":[3]}
            ]}"#,
        )
        .unwrap();
        assert!(option.series.iter().all(|series| !is_cartesian(series)));
    }

    #[test]
    fn pictorial_bars_participate_in_bar_group_layout() {
        let option = ChartOption::new()
            .push_series(Series::bar("bar", [1.0]))
            .push_series(Series::pictorial_bar("picture", [1.0]));
        let layouts = bar_layouts(&option, &[0, 1], 100.0);
        assert_eq!(layouts.len(), 2);
        assert!(layouts[&0].offset < layouts[&1].offset);
    }
}
