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
    pub(super) bar_layout: Option<(usize, usize)>,
    pub(super) stack: Option<&'a [(f64, f64)]>,
    pub(super) visual_map: Option<&'a VisualMap>,
    pub(super) palette: &'a [u32],
    pub(super) canvas: Option<&'a Canvas>,
    pub(super) hits: &'a mut Vec<HitRegion>,
}

pub(super) struct FreeRenderContext<'a> {
    pub(super) series_index: usize,
    pub(super) option: &'a ChartOption,
    pub(super) plot: Plot,
    pub(super) palette: &'a [u32],
    pub(super) canvas: Option<&'a Canvas>,
    pub(super) hits: &'a mut Vec<HitRegion>,
}

pub(super) fn is_cartesian(series: &Series) -> bool {
    matches!(
        series,
        Series::Line(_)
            | Series::Bar(_)
            | Series::Scatter(_)
            | Series::EffectScatter(_)
            | Series::Heatmap(_)
            | Series::Candlestick(_)
            | Series::Boxplot(_)
            | Series::PictorialBar(_)
    )
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
    let stacks = stack_layouts(option, series_indices);
    let (bar_offsets, bar_series_count) = bar_layouts(option, series_indices);
    let mut render_order = series_indices.to_vec();
    render_order.sort_by(|left, right| {
        cartesian_z(&option.series[*left])
            .total_cmp(&cartesian_z(&option.series[*right]))
            .then_with(|| left.cmp(right))
    });
    for series_index in &render_order {
        let bar_layout = bar_offsets
            .get(series_index)
            .copied()
            .map(|offset| (offset, bar_series_count));
        let mut context = CartesianRenderContext {
            series_index: *series_index,
            plot,
            layout,
            bar_layout,
            stack: stacks.get(series_index).map(Vec::as_slice),
            visual_map: option.visual_map.as_ref(),
            palette,
            canvas,
            hits,
        };
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
) -> BTreeMap<usize, Vec<(f64, f64)>> {
    let mut output = BTreeMap::new();
    let mut accumulators: BTreeMap<String, Vec<(f64, f64)>> = BTreeMap::new();
    for series_index in series_indices {
        let (data, stack) = match &option.series[*series_index] {
            Series::Line(series) | Series::Bar(series) => {
                (&series.data, series.options.stack.as_deref())
            }
            _ => continue,
        };
        let Some(stack) = stack else { continue };
        let accumulator = accumulators
            .entry(stack.to_string())
            .or_insert_with(|| vec![(0.0, 0.0); data.len()]);
        accumulator.resize(data.len(), (0.0, 0.0));
        let mut values = Vec::with_capacity(data.len());
        for (index, point) in data.iter().enumerate() {
            let value = if point.values.len() > 1 {
                point.number(1)
            } else {
                point.number(0)
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

fn bar_layouts(option: &ChartOption, series_indices: &[usize]) -> (BTreeMap<usize, usize>, usize) {
    let mut groups: BTreeMap<String, usize> = BTreeMap::new();
    let mut output = BTreeMap::new();
    for series_index in series_indices {
        let Series::Bar(series) = &option.series[*series_index] else {
            continue;
        };
        let key = series
            .options
            .stack
            .clone()
            .unwrap_or_else(|| format!("__series_{series_index}"));
        let next = groups.len();
        let offset = *groups.entry(key).or_insert(next);
        output.insert(*series_index, offset);
    }
    let count = groups.len().max(1);
    (output, count)
}

pub(super) fn render_free(
    option: &ChartOption,
    series_index: usize,
    series: &Series,
    plot: Plot,
    palette: &[u32],
    canvas: Option<&Canvas>,
    hits: &mut Vec<HitRegion>,
) {
    let mut context = FreeRenderContext {
        series_index,
        option,
        plot,
        palette,
        canvas,
        hits,
    };
    match series {
        Series::Pie(value) => pie::render(value, &mut context),
        Series::Radar(value) => radar::render(value, &mut context),
        Series::Gauge(value) => gauge::render(value, &mut context),
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
        Series::Line(_)
        | Series::Bar(_)
        | Series::Scatter(_)
        | Series::EffectScatter(_)
        | Series::Heatmap(_)
        | Series::Candlestick(_)
        | Series::Boxplot(_)
        | Series::PictorialBar(_) => {}
    }
}
