//! Shared ECharts-style interpolation and redraw clock driven by AnimationHost.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use arkit_animation::{Animatable, Easing, TimeSpan};

use crate::model::*;

thread_local! {
    static ANIMATION_TIME_SECONDS: Cell<f64> = const { Cell::new(0.0) };
}

struct AnimationTimeGuard {
    previous: f64,
}

impl Drop for AnimationTimeGuard {
    fn drop(&mut self) {
        ANIMATION_TIME_SECONDS.with(|time| time.set(self.previous));
    }
}

pub(crate) fn with_animation_time<R>(seconds: f64, render: impl FnOnce() -> R) -> R {
    let previous = ANIMATION_TIME_SECONDS.with(|time| time.replace(seconds));
    let _guard = AnimationTimeGuard { previous };
    render()
}

pub(crate) fn animation_time_seconds() -> f64 {
    ANIMATION_TIME_SECONDS.with(Cell::get)
}

pub(crate) struct ChartTransition {
    from: Rc<ChartOption>,
    to: Rc<ChartOption>,
    snapshot: RefCell<Rc<ChartOption>>,
    plan: TransitionPlan,
    timing: AnimationTiming,
    driver: ChartTransitionDriver,
}

#[derive(Clone)]
pub(crate) struct ChartTransitionDriver {
    progress: Option<Animatable<f32>>,
}

impl ChartTransitionDriver {
    pub(crate) fn new(progress: Animatable<f32>) -> Self {
        Self {
            progress: Some(progress),
        }
    }

    #[cfg(test)]
    pub(crate) fn immediate() -> Self {
        Self { progress: None }
    }

    fn start(&self, timing: &AnimationTiming) {
        if let Some(progress) = &self.progress {
            progress.animate(
                0.0,
                1.0,
                TimeSpan::from_millis(timing.duration),
                TimeSpan::from_millis(timing.delay),
                Easing::Linear,
            );
        }
    }

    fn progress(&self) -> f32 {
        self.progress
            .as_ref()
            .map_or(1.0, |progress| progress.get().clamp(0.0, 1.0))
    }
}

#[derive(Clone)]
pub(crate) struct ChartAnimationClock {
    pulse: Animatable<f32>,
    running: Rc<Cell<bool>>,
}

impl ChartAnimationClock {
    pub(crate) fn new(pulse: Animatable<f32>) -> Self {
        Self {
            pulse,
            running: Rc::new(Cell::new(false)),
        }
    }

    pub(crate) fn set_invalidator(&self, invalidator: impl Fn() + 'static) {
        self.pulse.set_invalidator(invalidator);
    }

    pub(crate) fn on_tick(&self, tick: impl Fn() + 'static) {
        self.pulse.controls().on_loop(move |_| tick());
    }

    pub(crate) fn start(&self) {
        if self.running.replace(true) {
            return;
        }
        self.pulse
            .animate_repeating(0.0, 1.0, TimeSpan::from_millis(33), Easing::Linear);
    }

    pub(crate) fn stop(&self) {
        if self.running.replace(false) {
            self.pulse.controls().pause();
        }
    }

    pub(crate) fn poke(&self) {
        self.pulse.controls().resume();
    }

    pub(crate) fn is_running(&self) -> bool {
        self.running.get()
    }
}

impl ChartTransition {
    pub(crate) fn initial(option: Rc<ChartOption>, driver: ChartTransitionDriver) -> Option<Self> {
        animation_allowed(&option).then(|| {
            driver.start(&option.animation.initial);
            let from = Rc::new(collapsed_option(&option));
            let plan = TransitionPlan::compile(&from, &option);
            Self {
                from,
                snapshot: RefCell::new(Rc::new((*option).clone())),
                plan,
                timing: option.animation.initial.clone(),
                to: option,
                driver,
            }
        })
    }

    pub(crate) fn update(
        from: Rc<ChartOption>,
        to: Rc<ChartOption>,
        driver: ChartTransitionDriver,
    ) -> Option<Self> {
        (animation_allowed(&to) && from != to).then(|| {
            driver.start(&to.animation.update);
            let plan = TransitionPlan::compile(&from, &to);
            Self {
                from,
                snapshot: RefCell::new(Rc::new((*to).clone())),
                plan,
                timing: to.animation.update.clone(),
                to,
                driver,
            }
        })
    }

    pub(crate) fn state(
        from: Rc<ChartOption>,
        to: Rc<ChartOption>,
        driver: ChartTransitionDriver,
    ) -> Option<Self> {
        (to.animation.enabled && from != to && to.animation.state.duration > 0).then(|| {
            driver.start(&to.animation.state);
            let plan = TransitionPlan::compile(&from, &to);
            Self {
                from,
                snapshot: RefCell::new(Rc::new((*to).clone())),
                plan,
                timing: to.animation.state.clone(),
                to,
                driver,
            }
        })
    }

    pub(crate) fn snapshot(&self) -> (Rc<ChartOption>, bool) {
        let progress = self.driver.progress();
        let finished = progress >= 1.0;
        if finished {
            return (self.to.clone(), true);
        }
        let progress = easing(&self.timing.easing, progress);
        let mut snapshot = self.snapshot.borrow_mut();
        self.plan
            .apply(&self.from, &self.to, Rc::make_mut(&mut snapshot), progress);
        (snapshot.clone(), false)
    }

    pub(crate) fn current(&self) -> Rc<ChartOption> {
        self.snapshot.borrow().clone()
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

struct TransitionPlan {
    series: Vec<SeriesTransitionPlan>,
}

enum SeriesTransitionPlan {
    Basic {
        source: Option<usize>,
        points: Vec<Option<usize>>,
    },
    Network {
        source: Option<usize>,
        nodes: Vec<Option<usize>>,
    },
    Sankey {
        source: Option<usize>,
        nodes: Vec<Option<usize>>,
    },
    Map {
        source: Option<usize>,
        features: Vec<Option<usize>>,
    },
    Lines {
        source: Option<usize>,
    },
    Sunburst {
        source: Option<usize>,
        nodes: Vec<SunburstTransitionPlan>,
    },
    None,
}

struct SunburstTransitionPlan {
    source: Option<usize>,
    children: Vec<Self>,
}

impl TransitionPlan {
    fn compile(from: &ChartOption, to: &ChartOption) -> Self {
        let series = to
            .series
            .iter()
            .enumerate()
            .map(|(index, target)| {
                let source = matching_series_index(&from.series, target, index);
                match target {
                    Series::Line(target)
                    | Series::Bar(target)
                    | Series::Pie(target)
                    | Series::Scatter(target)
                    | Series::EffectScatter(target)
                    | Series::Radar(target)
                    | Series::Gauge(target)
                    | Series::Funnel(target)
                    | Series::Heatmap(target)
                    | Series::Candlestick(target)
                    | Series::Boxplot(target)
                    | Series::PictorialBar(target)
                    | Series::Parallel(target)
                    | Series::ThemeRiver(target)
                    | Series::Treemap(target) => {
                        let source_data = source
                            .and_then(|index| from.series.get(index))
                            .and_then(basic_series)
                            .map(|series| series.data.as_slice());
                        SeriesTransitionPlan::Basic {
                            source,
                            points: match_data_points(source_data, &target.data),
                        }
                    }
                    Series::Tree(target) | Series::Graph(target) => {
                        let source_nodes = source
                            .and_then(|index| from.series.get(index))
                            .and_then(network_nodes);
                        SeriesTransitionPlan::Network {
                            source,
                            nodes: match_named_nodes(source_nodes, &target.nodes),
                        }
                    }
                    Series::Sankey(target) => {
                        let source_nodes = source
                            .and_then(|index| from.series.get(index))
                            .and_then(|series| match series {
                                Series::Sankey(series) => Some(series.nodes.as_slice()),
                                _ => None,
                            });
                        SeriesTransitionPlan::Sankey {
                            source,
                            nodes: match_named_nodes(source_nodes, &target.nodes),
                        }
                    }
                    Series::Map(target) => {
                        let source_features = source
                            .and_then(|index| from.series.get(index))
                            .and_then(|series| match series {
                                Series::Map(series) => Some(series.features.as_slice()),
                                _ => None,
                            });
                        SeriesTransitionPlan::Map {
                            source,
                            features: match_map_features(source_features, &target.features),
                        }
                    }
                    Series::Lines(_) => SeriesTransitionPlan::Lines { source },
                    Series::Sunburst(target) => {
                        let source_nodes = source
                            .and_then(|index| from.series.get(index))
                            .and_then(|series| match series {
                                Series::Sunburst(series) => Some(series.data.as_slice()),
                                _ => None,
                            });
                        SeriesTransitionPlan::Sunburst {
                            source,
                            nodes: compile_sunburst(source_nodes, &target.data),
                        }
                    }
                    Series::Custom(_) => SeriesTransitionPlan::None,
                }
            })
            .collect();
        Self { series }
    }

    fn apply(&self, from: &ChartOption, to: &ChartOption, output: &mut ChartOption, progress: f32) {
        for ((target, output), plan) in to.series.iter().zip(&mut output.series).zip(&self.series) {
            apply_series_transition(from, target, output, plan, progress);
        }
    }
}

#[cfg(test)]
pub(crate) fn interpolate_option(
    from: &ChartOption,
    to: &ChartOption,
    progress: f32,
) -> ChartOption {
    let plan = TransitionPlan::compile(from, to);
    let mut output = to.clone();
    plan.apply(from, to, &mut output, progress);
    output
}

fn matching_series_index(series: &[Series], target: &Series, index: usize) -> Option<usize> {
    let target_id = series_id(target);
    target_id
        .and_then(|id| {
            series
                .iter()
                .position(|candidate| series_id(candidate) == Some(id))
        })
        .or_else(|| {
            let target_name = target.name()?;
            series
                .iter()
                .position(|candidate| candidate.name() == Some(target_name))
        })
        .or_else(|| (index < series.len()).then_some(index))
        .filter(|source| std::mem::discriminant(&series[*source]) == std::mem::discriminant(target))
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

fn basic_series(series: &Series) -> Option<&BasicSeries> {
    match series {
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
        | Series::Treemap(value) => Some(value),
        _ => None,
    }
}

fn network_nodes(series: &Series) -> Option<&[NodeData]> {
    match series {
        Series::Tree(series) | Series::Graph(series) => Some(&series.nodes),
        _ => None,
    }
}

fn match_data_points(source: Option<&[DataPoint]>, target: &[DataPoint]) -> Vec<Option<usize>> {
    let Some(source) = source else {
        return vec![None; target.len()];
    };
    target
        .iter()
        .enumerate()
        .map(|(index, target)| {
            target
                .extra
                .get("id")
                .and_then(serde_json::Value::as_str)
                .and_then(|id| {
                    source.iter().position(|point| {
                        point.extra.get("id").and_then(serde_json::Value::as_str) == Some(id)
                    })
                })
                .or_else(|| {
                    target.name.as_ref().and_then(|name| {
                        source
                            .iter()
                            .position(|point| point.name.as_ref() == Some(name))
                    })
                })
                .or_else(|| (index < source.len()).then_some(index))
        })
        .collect()
}

fn match_named_nodes(source: Option<&[NodeData]>, target: &[NodeData]) -> Vec<Option<usize>> {
    let Some(source) = source else {
        return vec![None; target.len()];
    };
    target
        .iter()
        .enumerate()
        .map(|(index, target)| {
            source
                .iter()
                .position(|source| source.name == target.name)
                .or_else(|| (index < source.len()).then_some(index))
        })
        .collect()
}

fn match_map_features(source: Option<&[MapFeature]>, target: &[MapFeature]) -> Vec<Option<usize>> {
    let Some(source) = source else {
        return vec![None; target.len()];
    };
    target
        .iter()
        .enumerate()
        .map(|(index, target)| {
            source
                .iter()
                .position(|source| source.name == target.name)
                .or_else(|| (index < source.len()).then_some(index))
        })
        .collect()
}

fn compile_sunburst(
    source: Option<&[SunburstNode]>,
    target: &[SunburstNode],
) -> Vec<SunburstTransitionPlan> {
    target
        .iter()
        .enumerate()
        .map(|(index, target)| {
            let source_index = source.and_then(|source| {
                source
                    .iter()
                    .position(|source| source.name == target.name)
                    .or_else(|| (index < source.len()).then_some(index))
            });
            let source_children = source
                .and_then(|source| source_index.and_then(|index| source.get(index)))
                .map(|source| source.children.as_slice());
            SunburstTransitionPlan {
                source: source_index,
                children: compile_sunburst(source_children, &target.children),
            }
        })
        .collect()
}

fn apply_series_transition(
    from: &ChartOption,
    target: &Series,
    output: &mut Series,
    plan: &SeriesTransitionPlan,
    progress: f32,
) {
    match (target, output, plan) {
        (target, output, SeriesTransitionPlan::Basic { source, points })
            if basic_series(target).is_some() && basic_series(output).is_some() =>
        {
            let source = source
                .and_then(|index| from.series.get(index))
                .and_then(basic_series);
            interpolate_basic_series(
                source,
                basic_series(target).expect("guarded basic target"),
                match output {
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
                    | Series::Treemap(value) => value,
                    _ => unreachable!("guarded basic output"),
                },
                points,
                progress,
            );
        }
        (
            Series::Tree(target),
            Series::Tree(output),
            SeriesTransitionPlan::Network { source, nodes },
        )
        | (
            Series::Graph(target),
            Series::Graph(output),
            SeriesTransitionPlan::Network { source, nodes },
        ) => {
            let source = source.and_then(|index| from.series.get(index));
            let source_nodes = source.and_then(network_nodes);
            interpolate_nodes(
                source_nodes,
                &target.nodes,
                &mut output.nodes,
                nodes,
                progress,
                source.is_none(),
            );
            let source_links = source.and_then(|series| match series {
                Series::Tree(series) | Series::Graph(series) => Some(series.links.as_slice()),
                _ => None,
            });
            interpolate_links(
                source_links,
                &target.links,
                &mut output.links,
                progress,
                source.is_none(),
            );
            interpolate_options(
                source.and_then(series_options).unwrap_or(&target.options),
                &target.options,
                &mut output.options,
                progress,
            );
        }
        (
            Series::Sankey(target),
            Series::Sankey(output),
            SeriesTransitionPlan::Sankey { source, nodes },
        ) => {
            let source = source
                .and_then(|index| from.series.get(index))
                .and_then(|series| match series {
                    Series::Sankey(series) => Some(series),
                    _ => None,
                });
            interpolate_nodes(
                source.map(|value| value.nodes.as_slice()),
                &target.nodes,
                &mut output.nodes,
                nodes,
                progress,
                source.is_none(),
            );
            interpolate_links(
                source.map(|value| value.links.as_slice()),
                &target.links,
                &mut output.links,
                progress,
                source.is_none(),
            );
            interpolate_options(
                source.map_or(&target.options, |value| &value.options),
                &target.options,
                &mut output.options,
                progress,
            );
        }
        (
            Series::Map(target),
            Series::Map(output),
            SeriesTransitionPlan::Map { source, features },
        ) => {
            let source = source
                .and_then(|index| from.series.get(index))
                .and_then(|series| match series {
                    Series::Map(series) => Some(series),
                    _ => None,
                });
            for ((target, output), source_index) in target
                .features
                .iter()
                .zip(&mut output.features)
                .zip(features)
            {
                let source_feature = source
                    .and_then(|source| source_index.and_then(|index| source.features.get(index)));
                if let Some(target_value) = target.value {
                    if let Some(source_value) = source_feature
                        .and_then(|source| source.value)
                        .or_else(|| source.is_none().then_some(0.0))
                    {
                        output.value = Some(lerp(source_value, target_value, progress));
                    }
                }
                if let Some(source_feature) =
                    source_feature.or_else(|| source.is_none().then_some(target))
                {
                    interpolate_item_style_into(
                        &source_feature.item_style,
                        &target.item_style,
                        &mut output.item_style,
                        progress,
                    );
                }
            }
            interpolate_options(
                source.map_or(&target.options, |value| &value.options),
                &target.options,
                &mut output.options,
                progress,
            );
        }
        (Series::Lines(target), Series::Lines(output), SeriesTransitionPlan::Lines { source }) => {
            let source = source
                .and_then(|index| from.series.get(index))
                .and_then(|series| match series {
                    Series::Lines(series) => Some(series),
                    _ => None,
                });
            for (index, (target, output)) in target.data.iter().zip(&mut output.data).enumerate() {
                let source_line = source.and_then(|source| source.data.get(index));
                if let Some(source_line) = source_line {
                    output.from = lerp_point(source_line.from, target.from, progress);
                    output.to = lerp_point(source_line.to, target.to, progress);
                    output.value = lerp(source_line.value, target.value, progress);
                    for (point_index, (target_point, output_point)) in
                        target.coords.iter().zip(&mut output.coords).enumerate()
                    {
                        if let Some(source_point) = source_line.coords.get(point_index) {
                            *output_point = lerp_point(*source_point, *target_point, progress);
                        }
                    }
                } else if source.is_none() {
                    let origin = target.coords.first().copied().unwrap_or(target.from);
                    output.from = lerp_point(origin, target.from, progress);
                    output.to = lerp_point(origin, target.to, progress);
                    output.value = lerp(0.0, target.value, progress);
                    for (target_point, output_point) in target.coords.iter().zip(&mut output.coords)
                    {
                        *output_point = lerp_point(origin, *target_point, progress);
                    }
                }
            }
            interpolate_options(
                source.map_or(&target.options, |value| &value.options),
                &target.options,
                &mut output.options,
                progress,
            );
        }
        (
            Series::Sunburst(target),
            Series::Sunburst(output),
            SeriesTransitionPlan::Sunburst { source, nodes },
        ) => {
            let source = source
                .and_then(|index| from.series.get(index))
                .and_then(|series| match series {
                    Series::Sunburst(series) => Some(series),
                    _ => None,
                });
            interpolate_sunburst(
                source.map(|value| value.data.as_slice()),
                &target.data,
                &mut output.data,
                nodes,
                progress,
                source.is_none(),
            );
            interpolate_options(
                source.map_or(&target.options, |value| &value.options),
                &target.options,
                &mut output.options,
                progress,
            );
        }
        _ => {}
    }
}

fn interpolate_basic_series(
    from: Option<&BasicSeries>,
    target: &BasicSeries,
    output: &mut BasicSeries,
    point_sources: &[Option<usize>],
    progress: f32,
) {
    for ((target, output), source_index) in
        target.data.iter().zip(&mut output.data).zip(point_sources)
    {
        let source = from.and_then(|from| source_index.and_then(|index| from.data.get(index)));
        interpolate_point(source, target, output, progress);
    }
    let source_options = from.map_or(&target.options, |from| &from.options);
    interpolate_options(
        source_options,
        &target.options,
        &mut output.options,
        progress,
    );
}

fn interpolate_point(
    from: Option<&DataPoint>,
    target: &DataPoint,
    output: &mut DataPoint,
    progress: f32,
) {
    for (index, (target, output)) in target.values.iter().zip(&mut output.values).enumerate() {
        let Some(target_number) = target.as_f64() else {
            continue;
        };
        let source = from
            .and_then(|from| from.values.get(index))
            .and_then(DataValue::as_f64)
            .unwrap_or(0.0);
        *output = DataValue::Number(lerp(source, target_number, progress));
    }
    let source = from.unwrap_or(target);
    interpolate_item_style_into(
        &source.item_style,
        &target.item_style,
        &mut output.item_style,
        progress,
    );
    interpolate_label_into(&source.label, &target.label, &mut output.label, progress);
}

fn interpolate_nodes(
    from: Option<&[NodeData]>,
    target: &[NodeData],
    output: &mut [NodeData],
    sources: &[Option<usize>],
    progress: f32,
    collapse_missing: bool,
) {
    for ((target, output), source_index) in target.iter().zip(output).zip(sources) {
        let source = from.and_then(|from| source_index.and_then(|index| from.get(index)));
        if source.is_none() && !collapse_missing {
            continue;
        }
        let source_value = source.map_or(0.0, |source| source.value);
        output.value = lerp(source_value, target.value, progress);
        output.x = interpolate_optional(source.and_then(|source| source.x), target.x, progress);
        output.y = interpolate_optional(source.and_then(|source| source.y), target.y, progress);
        output.symbol_size = interpolate_optional_f32(
            source.and_then(|source| source.symbol_size),
            target.symbol_size,
            progress,
        );
        let source = source.unwrap_or(target);
        interpolate_item_style_into(
            &source.item_style,
            &target.item_style,
            &mut output.item_style,
            progress,
        );
        interpolate_label_into(&source.label, &target.label, &mut output.label, progress);
    }
}

fn interpolate_links(
    from: Option<&[LinkData]>,
    target: &[LinkData],
    output: &mut [LinkData],
    progress: f32,
    collapse_missing: bool,
) {
    for (index, (target, output)) in target.iter().zip(output).enumerate() {
        if let Some(source) = from.and_then(|from| from.get(index)) {
            output.value = lerp(source.value, target.value, progress);
        } else if collapse_missing {
            output.value = lerp(0.0, target.value, progress);
        }
    }
}

fn interpolate_sunburst(
    from: Option<&[SunburstNode]>,
    target: &[SunburstNode],
    output: &mut [SunburstNode],
    plans: &[SunburstTransitionPlan],
    progress: f32,
    collapse_missing: bool,
) {
    for ((target, output), plan) in target.iter().zip(output).zip(plans) {
        let source = from.and_then(|from| plan.source.and_then(|index| from.get(index)));
        if source.is_none() && !collapse_missing {
            continue;
        }
        output.value = lerp(
            source.map_or(0.0, |source| source.value),
            target.value,
            progress,
        );
        let style_source = source.unwrap_or(target);
        interpolate_item_style_into(
            &style_source.item_style,
            &target.item_style,
            &mut output.item_style,
            progress,
        );
        interpolate_sunburst(
            source.map(|source| source.children.as_slice()),
            &target.children,
            &mut output.children,
            &plan.children,
            progress,
            collapse_missing,
        );
    }
}

fn interpolate_options(
    from: &SeriesOptions,
    target: &SeriesOptions,
    output: &mut SeriesOptions,
    progress: f32,
) {
    interpolate_item_style_into(
        &from.item_style,
        &target.item_style,
        &mut output.item_style,
        progress,
    );
    interpolate_line_style_into(
        &from.line_style,
        &target.line_style,
        &mut output.line_style,
        progress,
    );
    interpolate_label_into(&from.label, &target.label, &mut output.label, progress);
    output.symbol_size = lerp_f32(from.symbol_size, target.symbol_size, progress);
    output.symbol_rotate = lerp_f32(from.symbol_rotate, target.symbol_rotate, progress);
    if let (Some(from), Some(target), Some(output)) =
        (&from.area_style, &target.area_style, &mut output.area_style)
    {
        interpolate_item_style_into(from, target, output, progress);
    }
}

fn interpolate_item_style_into(
    from: &ItemStyle,
    target: &ItemStyle,
    output: &mut ItemStyle,
    progress: f32,
) {
    output.color = interpolate_optional_color(from.color, target.color, progress);
    output.color0 = interpolate_optional_color(from.color0, target.color0, progress);
    output.border_color =
        interpolate_optional_color(from.border_color, target.border_color, progress);
    output.border_color0 =
        interpolate_optional_color(from.border_color0, target.border_color0, progress);
    output.border_width = lerp_f32(from.border_width, target.border_width, progress);
    output.opacity = lerp_f32(from.opacity, target.opacity, progress);
    for (index, value) in output.border_radius.iter_mut().enumerate() {
        *value = lerp_f32(
            from.border_radius[index],
            target.border_radius[index],
            progress,
        );
    }
}

fn interpolate_line_style_into(
    from: &LineStyle,
    target: &LineStyle,
    output: &mut LineStyle,
    progress: f32,
) {
    output.color = interpolate_optional_color(from.color, target.color, progress);
    output.width = lerp_f32(from.width, target.width, progress);
    output.opacity = lerp_f32(from.opacity, target.opacity, progress);
}

fn interpolate_label_into(
    from: &LabelStyle,
    target: &LabelStyle,
    output: &mut LabelStyle,
    progress: f32,
) {
    output.color = interpolate_optional_color(from.color, target.color, progress);
    output.font_size = lerp_f32(from.font_size, target.font_size, progress);
    output.distance = lerp_f32(from.distance, target.distance, progress);
    output.rotate = lerp_f32(from.rotate, target.rotate, progress);
    output.offset = [
        lerp_f32(from.offset[0], target.offset[0], progress),
        lerp_f32(from.offset[1], target.offset[1], progress),
    ];
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
