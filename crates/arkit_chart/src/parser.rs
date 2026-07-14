//! Parser for the supported ECharts-like JSON option subset.

use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

use serde_json::Value;

use crate::model::*;

pub(crate) fn parse_option_str(input: &str) -> Result<ChartOption, ChartParseError> {
    let value = serde_json::from_str(input).map_err(|error| ChartParseError {
        message: error.to_string(),
    })?;
    parse_option_value(value)
}

pub(crate) fn parse_option_value(value: Value) -> Result<ChartOption, ChartParseError> {
    let mut option = ChartOption::default();
    let Value::Object(mut object) = value else {
        return Err(ChartParseError {
            message: String::from("chart option must be a JSON object"),
        });
    };

    if object.contains_key("baseOption")
        || object.contains_key("options")
        || object.contains_key("media")
    {
        return parse_composite_option(object);
    }

    if let Some(value) = object.remove("title") {
        option.title = parse_title(value);
    }
    if let Some(value) = object.remove("legend") {
        option.legend = parse_legend(value);
    }
    if let Some(value) = object.remove("grid") {
        option.grid = parse_grid_list(value);
    }
    if let Some(value) = object.remove("tooltip") {
        option.tooltip = parse_tooltip(value);
    }
    if let Some(value) = object.remove("xAxis").or_else(|| object.remove("x_axis")) {
        option.x_axis = parse_axis_list(value, AxisOrientation::X);
    }
    if let Some(value) = object.remove("yAxis").or_else(|| object.remove("y_axis")) {
        option.y_axis = parse_axis_list(value, AxisOrientation::Y);
    }
    if let Some(value) = object.remove("radar") {
        option.radar = parse_radar_list(value);
    }
    if let Some(value) = object.remove("dataset") {
        option.datasets = parse_dataset_list(value);
        option.dataset = option.datasets.first().cloned();
    }
    if let Some(value) = object.remove("visualMap") {
        option.visual_maps = parse_visual_map_list(value);
        option.visual_map = option.visual_maps.first().cloned();
    }
    if let Some(value) = object.remove("dataZoom") {
        option.data_zoom = parse_data_zoom_list(value);
    }
    if let Some(value) = object.remove("timeline") {
        option.timeline = parse_timeline(value);
    }
    if let Some(value) = object.remove("brush") {
        option.brush = parse_brush(value);
    } else if toolbox_has_brush(&object) {
        option.brush = Some(BrushOptions::default());
    }
    option.animation = parse_animation_options(&object);
    for key in [
        "animation",
        "animationThreshold",
        "animationDuration",
        "animationEasing",
        "animationDelay",
        "animationDurationUpdate",
        "animationEasingUpdate",
        "animationDelayUpdate",
        "stateAnimation",
    ] {
        object.remove(key);
    }
    append_toolbox_data_zoom(&mut option, &object);
    if let Some(value) = object
        .remove("color")
        .or_else(|| object.remove("colors"))
        .and_then(parse_color_palette)
    {
        option.visual_style.palette = value;
    }
    if let Some(color) = object.get("backgroundColor").and_then(parse_color) {
        option.visual_style.background_color = color;
    }
    if let Some(color) = object
        .get("textStyle")
        .and_then(Value::as_object)
        .and_then(|style| style.get("color"))
        .and_then(parse_color)
    {
        option.visual_style.text_color = color;
    }
    if let Some(value) = object.remove("series") {
        let (series, diagnostics) = parse_series_list(value);
        option.series = series;
        option.diagnostics.extend(diagnostics);
    }

    apply_dataset(&mut option);

    option.extra = object.into_iter().collect();
    Ok(option)
}

fn parse_animation_options(object: &serde_json::Map<String, Value>) -> AnimationOptions {
    let mut animation = AnimationOptions::default();
    animation.enabled = match object.get("animation") {
        Some(Value::Bool(enabled)) => *enabled,
        Some(Value::String(value)) => value != "false",
        _ => animation.enabled,
    };
    animation.threshold = object
        .get("animationThreshold")
        .and_then(Value::as_u64)
        .unwrap_or(animation.threshold as u64) as usize;
    animation.initial.duration = object
        .get("animationDuration")
        .and_then(Value::as_u64)
        .unwrap_or(animation.initial.duration);
    animation.initial.easing = object
        .get("animationEasing")
        .and_then(Value::as_str)
        .unwrap_or(&animation.initial.easing)
        .to_string();
    animation.initial.delay = object
        .get("animationDelay")
        .and_then(Value::as_u64)
        .unwrap_or(animation.initial.delay);
    animation.update.duration = object
        .get("animationDurationUpdate")
        .and_then(Value::as_u64)
        .unwrap_or(animation.update.duration);
    animation.update.easing = object
        .get("animationEasingUpdate")
        .and_then(Value::as_str)
        .unwrap_or(&animation.update.easing)
        .to_string();
    animation.update.delay = object
        .get("animationDelayUpdate")
        .and_then(Value::as_u64)
        .unwrap_or(animation.update.delay);
    if let Some(state) = object.get("stateAnimation").and_then(Value::as_object) {
        animation.state.duration = state
            .get("duration")
            .and_then(Value::as_u64)
            .unwrap_or(animation.state.duration);
        animation.state.easing = state
            .get("easing")
            .and_then(Value::as_str)
            .unwrap_or(&animation.state.easing)
            .to_string();
        animation.state.delay = state
            .get("delay")
            .and_then(Value::as_u64)
            .unwrap_or(animation.state.delay);
    }
    animation
}

fn parse_composite_option(
    mut root: serde_json::Map<String, Value>,
) -> Result<ChartOption, ChartParseError> {
    let frame_values = root
        .remove("options")
        .and_then(|value| value.as_array().cloned())
        .unwrap_or_default();
    let media_values = root
        .remove("media")
        .and_then(|value| value.as_array().cloned())
        .unwrap_or_default();
    let base_value = root
        .remove("baseOption")
        .unwrap_or_else(|| Value::Object(root.clone()));
    let Value::Object(mut base) = base_value else {
        return Err(ChartParseError {
            message: String::from("baseOption must be a JSON object"),
        });
    };
    let root_timeline = root.remove("timeline");
    if !base.contains_key("timeline") {
        if let Some(timeline) = root_timeline {
            base.insert(String::from("timeline"), timeline);
        }
    }

    for (key, value) in root {
        base.entry(key).or_insert(value);
    }
    let base_value = Value::Object(base.clone());
    let mut base_option = parse_option_value(base_value.clone())?;
    let has_timeline = base.contains_key("timeline") || !frame_values.is_empty();
    if has_timeline {
        let mut timeline = base
            .get("timeline")
            .cloned()
            .and_then(parse_timeline)
            .unwrap_or_default();
        let raw_frames = if frame_values.is_empty() {
            vec![Value::Object(Default::default())]
        } else {
            frame_values.clone()
        };
        let mut frames = Vec::with_capacity(raw_frames.len());
        for frame in &raw_frames {
            let mut merged = base_value.clone();
            deep_merge(&mut merged, frame.clone());
            frames.push(parse_option_value(merged)?);
        }
        if timeline.data.is_empty() {
            timeline.data = (0..frames.len()).map(|index| index.to_string()).collect();
        }
        timeline.current_index = timeline.current_index.min(frames.len().saturating_sub(1));
        base_option.timeline = Some(timeline.clone());
        base_option.timeline_options = frames;
    }
    if !media_values.is_empty() {
        let rules = media_values
            .iter()
            .filter_map(parse_media_rule)
            .collect::<Vec<_>>();
        if !rules.is_empty() {
            base_option.media = Some(MediaOptions {
                base_option: base_value,
                timeline_options: frame_values,
                rules,
            });
        }
    }
    if has_timeline {
        let current_index = base_option
            .timeline
            .as_ref()
            .map_or(0, |timeline| timeline.current_index);
        base_option.apply_timeline_index(current_index);
    }
    Ok(base_option)
}

fn parse_media_rule(value: &Value) -> Option<MediaRule> {
    let object = value.as_object()?;
    let option = object.get("option")?.clone();
    let query = object.get("query").and_then(Value::as_object).map(|query| {
        let number = |key: &str| {
            query
                .get(key)
                .and_then(Value::as_f64)
                .map(|value| value as f32)
        };
        MediaQuery {
            min_width: number("minWidth"),
            max_width: number("maxWidth"),
            min_height: number("minHeight"),
            max_height: number("maxHeight"),
            min_aspect_ratio: number("minAspectRatio"),
            max_aspect_ratio: number("maxAspectRatio"),
        }
    });
    Some(MediaRule { query, option })
}

pub(crate) fn media_signature(option: &ChartOption, width: f32, height: f32) -> Vec<isize> {
    let Some(media) = option.media.as_ref() else {
        return Vec::new();
    };
    let mut matched = media
        .rules
        .iter()
        .enumerate()
        .filter_map(|(index, rule)| {
            rule.query
                .as_ref()
                .is_some_and(|query| query.matches(width, height))
                .then_some(index as isize)
        })
        .collect::<Vec<_>>();
    if matched.is_empty() {
        if let Some(index) = media.rules.iter().position(|rule| rule.query.is_none()) {
            matched.push(index as isize);
        }
    }
    matched
}

pub(crate) fn resolve_media_option(
    option: &ChartOption,
    width: f32,
    height: f32,
    timeline_index: usize,
) -> Result<ChartOption, ChartParseError> {
    let Some(media) = option.media.as_ref() else {
        return Ok(option.clone());
    };
    let mut merged = media.base_option.clone();
    if let Some(frame) = media.timeline_options.get(timeline_index) {
        deep_merge(&mut merged, frame.clone());
    }
    for index in media_signature(option, width, height) {
        if let Some(rule) = media.rules.get(index as usize) {
            deep_merge(&mut merged, rule.option.clone());
        }
    }
    let mut resolved = parse_option_value(merged)?;
    resolved.media = Some(media.clone());
    if !option.timeline_options.is_empty() {
        let mut timeline = option.timeline.clone().unwrap_or_default();
        timeline.current_index = timeline_index.min(option.timeline_options.len() - 1);
        resolved.timeline = Some(timeline);
        resolved.timeline_options = option.timeline_options.clone();
    }
    Ok(resolved)
}

fn deep_merge(target: &mut Value, source: Value) {
    match (target, source) {
        (Value::Object(target), Value::Object(source)) => {
            for (key, value) in source {
                if let Some(current) = target.get_mut(&key) {
                    deep_merge(current, value);
                } else {
                    target.insert(key, value);
                }
            }
        }
        (Value::Array(target), Value::Array(source)) => {
            for (index, value) in source.into_iter().enumerate() {
                if let Some(current) = target.get_mut(index) {
                    deep_merge(current, value);
                } else {
                    target.push(value);
                }
            }
        }
        (target, source) => *target = source,
    }
}

fn toolbox_has_brush(object: &serde_json::Map<String, Value>) -> bool {
    object
        .get("toolbox")
        .and_then(Value::as_object)
        .and_then(|toolbox| toolbox.get("feature"))
        .and_then(Value::as_object)
        .and_then(|features| features.get("brush"))
        .is_some_and(|value| !matches!(value, Value::Bool(false)))
}

fn append_toolbox_data_zoom(option: &mut ChartOption, object: &serde_json::Map<String, Value>) {
    let Some(feature) = object
        .get("toolbox")
        .and_then(Value::as_object)
        .and_then(|toolbox| toolbox.get("feature"))
        .and_then(Value::as_object)
        .and_then(|features| features.get("dataZoom"))
        .filter(|value| !matches!(value, Value::Bool(false)))
    else {
        return;
    };
    if feature
        .get("show")
        .and_then(Value::as_bool)
        .is_some_and(|show| !show)
    {
        return;
    }
    let filter_mode = feature
        .get("filterMode")
        .and_then(Value::as_str)
        .unwrap_or("filter")
        .to_string();
    let x_indices = toolbox_axis_indices(feature.get("xAxisIndex"), option.x_axis.len());
    let y_indices = toolbox_axis_indices(feature.get("yAxisIndex"), option.y_axis.len());
    for (indices, horizontal) in [(x_indices, true), (y_indices, false)] {
        if indices.is_empty() {
            continue;
        }
        let mut extra = BTreeMap::new();
        extra.insert(String::from("toolboxInternal"), Value::Bool(true));
        option.data_zoom.push(DataZoom {
            show: false,
            kind: String::from("select"),
            x_axis_index: if horizontal {
                indices.clone()
            } else {
                Vec::new()
            },
            y_axis_index: if horizontal { Vec::new() } else { indices },
            filter_mode: filter_mode.clone(),
            orient: if horizontal {
                String::from("horizontal")
            } else {
                String::from("vertical")
            },
            extra,
            ..DataZoom::default()
        });
    }
}

fn toolbox_axis_indices(value: Option<&Value>, count: usize) -> Vec<usize> {
    match value {
        Some(Value::Bool(false)) => Vec::new(),
        Some(Value::Number(value)) => value
            .as_u64()
            .map(|value| vec![value as usize])
            .unwrap_or_default(),
        Some(Value::Array(values)) => values
            .iter()
            .filter_map(Value::as_u64)
            .map(|value| value as usize)
            .filter(|index| *index < count)
            .collect(),
        _ => (0..count).collect(),
    }
}

fn apply_dataset(option: &mut ChartOption) {
    let datasets = if option.datasets.is_empty() {
        option.dataset.iter().cloned().collect::<Vec<_>>()
    } else {
        option.datasets.clone()
    };
    let axis_dataset_index = option
        .series
        .iter()
        .find(|series| matches!(dataset_series_kind(series), DatasetSeriesKind::LineOrBar))
        .and_then(series_options_extra)
        .and_then(|extra| extra.get("datasetIndex"))
        .and_then(Value::as_u64)
        .unwrap_or(0) as usize;
    let Some(dataset) = datasets
        .get(axis_dataset_index)
        .or_else(|| datasets.first())
    else {
        return;
    };
    if dataset.source.is_empty() {
        return;
    }
    let rows = &dataset.source[usize::from(dataset.source_header).min(dataset.source.len())..];
    if option
        .x_axis
        .first()
        .is_some_and(|axis| matches!(axis.axis_type, AxisType::Category) && axis.data.is_empty())
    {
        if let Some(axis) = option.x_axis.first_mut() {
            axis.data = rows
                .iter()
                .filter_map(|row| row.first())
                .map(data_value_label)
                .collect();
        }
    }

    for (series_index, series) in option.series.iter_mut().enumerate() {
        let dataset_index = series_options_extra(series)
            .and_then(|extra| extra.get("datasetIndex"))
            .and_then(Value::as_u64)
            .unwrap_or(0) as usize;
        let Some(dataset) = datasets.get(dataset_index).or_else(|| datasets.first()) else {
            continue;
        };
        let headers = if !dataset.dimensions.is_empty() {
            dataset.dimensions.clone()
        } else if dataset.source_header {
            dataset
                .source
                .first()
                .map(|row| row.iter().map(data_value_label).collect::<Vec<_>>())
                .unwrap_or_default()
        } else {
            Vec::new()
        };
        let rows = &dataset.source[usize::from(dataset.source_header).min(dataset.source.len())..];
        let kind = dataset_series_kind(series);
        let Some(basic) = basic_series_mut(series) else {
            continue;
        };
        if !basic.data.is_empty() {
            continue;
        }
        let encode = basic.options.extra.get("encode").and_then(Value::as_object);
        let default_value =
            (series_index + 1).min(dataset.source.first().map_or(1, Vec::len).saturating_sub(1));
        let x_dimension = encode
            .and_then(|encode| encode.get("x"))
            .and_then(|value| resolve_dimension(value, &headers))
            .unwrap_or(0);
        let y_dimension = encode
            .and_then(|encode| encode.get("y"))
            .or_else(|| encode.and_then(|encode| encode.get("value")))
            .and_then(|value| resolve_dimension(value, &headers))
            .unwrap_or(default_value);
        let name_dimension = encode
            .and_then(|encode| encode.get("itemName"))
            .and_then(|value| resolve_dimension(value, &headers))
            .unwrap_or(0);

        basic.data = rows
            .iter()
            .filter_map(|row| match kind {
                DatasetSeriesKind::LineOrBar => {
                    row.get(y_dimension).cloned().map(DataPoint::scalar)
                }
                DatasetSeriesKind::ScatterOrHeatmap => Some(DataPoint::values([
                    row.get(x_dimension)?.clone(),
                    row.get(y_dimension)?.clone(),
                ])),
                DatasetSeriesKind::Named => Some(DataPoint::named(
                    data_value_label(row.get(name_dimension)?),
                    row.get(y_dimension)?.clone(),
                )),
                DatasetSeriesKind::Vector => Some(DataPoint::values(row.iter().skip(1).cloned())),
                DatasetSeriesKind::Unsupported => None,
            })
            .collect();
    }
}

fn series_options_extra(series: &Series) -> Option<&std::collections::BTreeMap<String, Value>> {
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
        | Series::Treemap(value) => Some(&value.options.extra),
        Series::Tree(value) | Series::Graph(value) => Some(&value.options.extra),
        Series::Sankey(value) => Some(&value.options.extra),
        Series::Map(value) => Some(&value.options.extra),
        Series::Lines(value) => Some(&value.options.extra),
        Series::Sunburst(value) => Some(&value.options.extra),
        Series::Custom(_) => None,
    }
}

#[derive(Clone, Copy)]
enum DatasetSeriesKind {
    LineOrBar,
    ScatterOrHeatmap,
    Named,
    Vector,
    Unsupported,
}

fn dataset_series_kind(series: &Series) -> DatasetSeriesKind {
    match series {
        Series::Line(_) | Series::Bar(_) | Series::PictorialBar(_) => DatasetSeriesKind::LineOrBar,
        Series::Scatter(_) | Series::EffectScatter(_) | Series::Heatmap(_) => {
            DatasetSeriesKind::ScatterOrHeatmap
        }
        Series::Pie(_) | Series::Funnel(_) | Series::Treemap(_) | Series::Gauge(_) => {
            DatasetSeriesKind::Named
        }
        Series::Radar(_)
        | Series::Candlestick(_)
        | Series::Boxplot(_)
        | Series::Parallel(_)
        | Series::ThemeRiver(_) => DatasetSeriesKind::Vector,
        _ => DatasetSeriesKind::Unsupported,
    }
}

fn basic_series_mut(series: &mut Series) -> Option<&mut BasicSeries> {
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

fn resolve_dimension(value: &Value, headers: &[String]) -> Option<usize> {
    let value = value
        .as_array()
        .and_then(|values| values.first())
        .unwrap_or(value);
    value.as_u64().map(|value| value as usize).or_else(|| {
        value
            .as_str()
            .and_then(|name| headers.iter().position(|header| header == name))
    })
}

fn data_value_label(value: &DataValue) -> String {
    match value {
        DataValue::String(value) => value.clone(),
        DataValue::Number(value) => value.to_string(),
        DataValue::Null => String::new(),
    }
}

fn parse_title(value: Value) -> Option<Title> {
    match value {
        Value::String(text) => Some(Title {
            text,
            ..Title::default()
        }),
        Value::Object(object) => {
            let mut title = Title::default();
            title.text = object
                .get("text")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            title.subtext = object
                .get("subtext")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned);
            title.left = object.get("left").cloned().unwrap_or(title.left);
            title.top = object.get("top").cloned().unwrap_or(title.top);
            title.text_style = object
                .get("textStyle")
                .map(parse_text_options)
                .unwrap_or(title.text_style);
            title.subtext_style = object
                .get("subtextStyle")
                .map(parse_text_options)
                .unwrap_or(title.subtext_style);
            Some(title)
        }
        _ => None,
    }
}

fn parse_legend(value: Value) -> Option<Legend> {
    match value {
        Value::Bool(show) => Some(Legend {
            show,
            ..Legend::default()
        }),
        Value::Object(mut object) => {
            let mut legend = Legend::default();
            legend.show = object.get("show").and_then(Value::as_bool).unwrap_or(true);
            legend.orient = object
                .get("orient")
                .and_then(Value::as_str)
                .unwrap_or("horizontal")
                .to_string();
            legend.align = object
                .get("align")
                .and_then(Value::as_str)
                .unwrap_or("auto")
                .to_string();
            legend.left = object.get("left").cloned().unwrap_or(legend.left);
            legend.top = object.get("top").cloned().unwrap_or(legend.top);
            if let Some(values) = object.get("data").and_then(Value::as_array) {
                for value in values {
                    let name = legend_name_from_value(value);
                    if name.is_empty() {
                        continue;
                    }
                    if let Some(icon) = value
                        .as_object()
                        .and_then(|item| item.get("icon"))
                        .and_then(Value::as_str)
                    {
                        legend.data_icons.insert(name.clone(), icon.to_string());
                    }
                    if !legend.data.contains(&name) {
                        legend.data.push(name);
                    }
                }
            }
            legend.item_width = object
                .get("itemWidth")
                .and_then(parse_f32)
                .unwrap_or(legend.item_width);
            legend.item_height = object
                .get("itemHeight")
                .and_then(parse_f32)
                .unwrap_or(legend.item_height);
            legend.item_gap = object
                .get("itemGap")
                .and_then(parse_f32)
                .unwrap_or(legend.item_gap)
                .max(0.0);
            legend.icon = object
                .get("icon")
                .and_then(Value::as_str)
                .unwrap_or(&legend.icon)
                .to_string();
            legend.inactive_color = object
                .get("inactiveColor")
                .and_then(parse_color)
                .unwrap_or(legend.inactive_color);
            legend.formatter = object
                .get("formatter")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned);
            legend.selected_mode = object
                .get("selectedMode")
                .map(|value| match value {
                    Value::Bool(value) => value.to_string(),
                    Value::String(value) => value.clone(),
                    _ => String::from("true"),
                })
                .unwrap_or_else(|| String::from("true"));
            legend.selected = object
                .get("selected")
                .and_then(Value::as_object)
                .map(|selected| {
                    selected
                        .iter()
                        .filter_map(|(name, value)| Some((name.clone(), value.as_bool()?)))
                        .collect()
                })
                .unwrap_or_default();
            legend.text_style = object
                .get("textStyle")
                .map(parse_text_options)
                .unwrap_or_default();
            for key in [
                "show",
                "orient",
                "align",
                "left",
                "top",
                "data",
                "itemWidth",
                "itemHeight",
                "itemGap",
                "icon",
                "inactiveColor",
                "formatter",
                "selectedMode",
                "selected",
                "textStyle",
            ] {
                object.remove(key);
            }
            legend.extra = object.into_iter().collect();
            Some(legend)
        }
        _ => None,
    }
}

fn legend_name_from_value(value: &Value) -> String {
    value
        .as_object()
        .and_then(|item| item.get("name"))
        .map(label_from_value)
        .unwrap_or_else(|| label_from_value(value))
}

fn parse_timeline(value: Value) -> Option<Timeline> {
    match value {
        Value::Bool(show) => Some(Timeline {
            show,
            ..Timeline::default()
        }),
        Value::Object(object) => {
            let mut timeline = Timeline::default();
            timeline.show = object.get("show").and_then(Value::as_bool).unwrap_or(true);
            timeline.current_index = object
                .get("currentIndex")
                .and_then(Value::as_u64)
                .unwrap_or(0) as usize;
            timeline.auto_play = object
                .get("autoPlay")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            timeline.rewind = object
                .get("rewind")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            timeline.loop_play = object.get("loop").and_then(Value::as_bool).unwrap_or(true);
            timeline.play_interval = object
                .get("playInterval")
                .and_then(Value::as_u64)
                .unwrap_or(2_000)
                .max(100);
            timeline.orient = object
                .get("orient")
                .and_then(Value::as_str)
                .unwrap_or("horizontal")
                .to_string();
            timeline.inverse = object
                .get("inverse")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            timeline.left = object.get("left").cloned().unwrap_or(timeline.left);
            timeline.right = object.get("right").cloned().unwrap_or(timeline.right);
            timeline.top = object.get("top").cloned().unwrap_or(timeline.top);
            timeline.bottom = object.get("bottom").cloned().unwrap_or(timeline.bottom);
            timeline.data = object
                .get("data")
                .and_then(Value::as_array)
                .map(|values| values.iter().map(timeline_label_from_value).collect())
                .unwrap_or_default();
            if let Some(value) = object.get("label") {
                timeline.label = parse_label_style(value);
                if value.get("show").is_none() {
                    timeline.label.show = true;
                }
            }
            timeline.line_style = object
                .get("lineStyle")
                .map(|value| parse_line_style_with_default(value, &timeline.line_style))
                .unwrap_or(timeline.line_style);
            timeline.item_style = object
                .get("itemStyle")
                .map(|value| parse_item_style_with_default(value, &timeline.item_style))
                .unwrap_or(timeline.item_style);
            timeline.checkpoint_style = object
                .get("checkpointStyle")
                .map(|value| parse_item_style_with_default(value, &timeline.checkpoint_style))
                .unwrap_or(timeline.checkpoint_style);
            timeline.control_style = object
                .get("controlStyle")
                .map(|value| parse_item_style_with_default(value, &timeline.control_style))
                .unwrap_or(timeline.control_style);
            let mut extra = object;
            for key in [
                "show",
                "currentIndex",
                "autoPlay",
                "rewind",
                "loop",
                "playInterval",
                "orient",
                "inverse",
                "left",
                "right",
                "top",
                "bottom",
                "data",
                "label",
                "lineStyle",
                "itemStyle",
                "checkpointStyle",
                "controlStyle",
            ] {
                extra.remove(key);
            }
            timeline.extra = extra.into_iter().collect();
            Some(timeline)
        }
        _ => None,
    }
}

fn timeline_label_from_value(value: &Value) -> String {
    value
        .as_object()
        .and_then(|value| value.get("value").or_else(|| value.get("name")))
        .map(label_from_value)
        .unwrap_or_else(|| label_from_value(value))
}

fn parse_brush(value: Value) -> Option<BrushOptions> {
    match value {
        Value::Bool(active) => Some(BrushOptions {
            active,
            ..BrushOptions::default()
        }),
        Value::Object(object) => {
            let mut brush = BrushOptions::default();
            brush.active = object
                .get("brushType")
                .and_then(Value::as_str)
                .is_some_and(|value| value != "none");
            brush.brush_type = object
                .get("brushType")
                .and_then(Value::as_str)
                .unwrap_or("rect")
                .to_string();
            brush.brush_mode = object
                .get("brushMode")
                .and_then(Value::as_str)
                .unwrap_or("single")
                .to_string();
            brush.transformable = object
                .get("transformable")
                .and_then(Value::as_bool)
                .unwrap_or(true);
            brush.remove_on_click = object
                .get("removeOnClick")
                .and_then(Value::as_bool)
                .unwrap_or(true);
            brush.brush_style = object
                .get("brushStyle")
                .map(|value| parse_item_style_with_default(value, &brush.brush_style))
                .unwrap_or(brush.brush_style);
            brush.in_brush_color = object
                .get("inBrush")
                .and_then(Value::as_object)
                .and_then(|style| style.get("color"))
                .and_then(parse_color);
            brush.out_of_brush_opacity = object
                .get("outOfBrush")
                .and_then(Value::as_object)
                .and_then(|style| style.get("opacity"))
                .and_then(Value::as_f64)
                .unwrap_or(0.3)
                .clamp(0.0, 1.0) as f32;
            let mut extra = object;
            for key in [
                "brushType",
                "brushMode",
                "transformable",
                "removeOnClick",
                "brushStyle",
                "inBrush",
                "outOfBrush",
            ] {
                extra.remove(key);
            }
            brush.extra = extra.into_iter().collect();
            Some(brush)
        }
        _ => None,
    }
}

fn parse_tooltip(value: Value) -> Tooltip {
    match value {
        Value::Bool(show) => Tooltip {
            show,
            ..Tooltip::default()
        },
        Value::Object(object) => {
            let mut tooltip = Tooltip::default();
            tooltip.show = object.get("show").and_then(Value::as_bool).unwrap_or(true);
            tooltip.trigger = object
                .get("trigger")
                .and_then(Value::as_str)
                .unwrap_or("item")
                .to_string();
            tooltip.formatter = object
                .get("formatter")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned);
            tooltip.background_color = object
                .get("backgroundColor")
                .and_then(parse_color)
                .unwrap_or(tooltip.background_color);
            tooltip.border_color = object
                .get("borderColor")
                .and_then(parse_color)
                .unwrap_or(tooltip.border_color);
            tooltip.text_color = object
                .get("textStyle")
                .and_then(Value::as_object)
                .and_then(|style| style.get("color"))
                .and_then(parse_color)
                .unwrap_or(tooltip.text_color);
            tooltip.padding = object
                .get("padding")
                .and_then(parse_f32)
                .unwrap_or(tooltip.padding);
            tooltip.axis_pointer = object
                .get("axisPointer")
                .map(parse_axis_pointer)
                .unwrap_or(tooltip.axis_pointer);
            tooltip
        }
        _ => Tooltip::default(),
    }
}

fn parse_axis_pointer(value: &Value) -> AxisPointer {
    let Some(object) = value.as_object() else {
        return AxisPointer::default();
    };
    let mut pointer = AxisPointer::default();
    pointer.show = object.get("show").and_then(Value::as_bool).unwrap_or(true);
    pointer.kind = object
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or("line")
        .to_string();
    pointer.snap = object.get("snap").and_then(Value::as_bool).unwrap_or(false);
    pointer.line_style = object
        .get("lineStyle")
        .map(parse_line_style)
        .unwrap_or(pointer.line_style);
    pointer.label = object
        .get("label")
        .map(parse_label_style)
        .unwrap_or(pointer.label);
    pointer
}

fn parse_text_options(value: &Value) -> TextOptions {
    let Some(object) = value.as_object() else {
        return TextOptions::default();
    };
    TextOptions {
        color: object.get("color").and_then(parse_color),
        font_size: object.get("fontSize").and_then(parse_f32).unwrap_or(12.0),
        font_weight: object
            .get("fontWeight")
            .and_then(|value| match value {
                Value::Number(value) => value.as_i64().map(|value| value as i32),
                Value::String(value) if value == "bold" || value == "bolder" => Some(700),
                Value::String(value) if value == "normal" || value == "lighter" => Some(400),
                _ => None,
            })
            .unwrap_or(400),
    }
}

fn parse_grid_list(value: Value) -> Vec<Grid> {
    match value {
        Value::Array(values) => values.into_iter().map(parse_grid).collect(),
        value => vec![parse_grid(value)],
    }
}

fn parse_grid(value: Value) -> Grid {
    let mut grid = Grid::default();
    let Value::Object(object) = value else {
        return grid;
    };
    grid.left = object
        .get("left")
        .and_then(parse_length)
        .unwrap_or(grid.left);
    grid.right = object
        .get("right")
        .and_then(parse_length)
        .unwrap_or(grid.right);
    grid.top = object.get("top").and_then(parse_length).unwrap_or(grid.top);
    grid.bottom = object
        .get("bottom")
        .and_then(parse_length)
        .unwrap_or(grid.bottom);
    grid.width = object.get("width").and_then(parse_length);
    grid.height = object.get("height").and_then(parse_length);
    grid.contain_label = object
        .get("containLabel")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    grid
}

fn parse_axis_list(value: Value, orientation: AxisOrientation) -> Vec<Axis> {
    match value {
        Value::Array(values) => values
            .into_iter()
            .map(|value| parse_axis(value, orientation))
            .collect(),
        value => vec![parse_axis(value, orientation)],
    }
}

fn parse_axis(value: Value, orientation: AxisOrientation) -> Axis {
    let default = if matches!(orientation, AxisOrientation::X) {
        Axis::category(Vec::<String>::new())
    } else {
        Axis::value()
    };
    let Value::Object(object) = value else {
        return default;
    };
    let axis_type = match object.get("type").and_then(Value::as_str) {
        Some("value") => AxisType::Value,
        Some("time") => AxisType::Time,
        Some("log") => AxisType::Log,
        Some("category") => AxisType::Category,
        _ => default.axis_type,
    };
    let axis_line = parse_axis_line(object.get("axisLine"), &default.axis_line);
    let axis_tick = parse_axis_tick(object.get("axisTick"), &default.axis_tick);
    let (axis_label, axis_label_style) =
        parse_axis_label(object.get("axisLabel"), &default.axis_label_style);
    let (split_line, split_line_style) = parse_split_line(
        object.get("splitLine"),
        orientation,
        &default.split_line_style,
    );
    Axis {
        axis_type,
        name: object
            .get("name")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        data: object
            .get("data")
            .and_then(Value::as_array)
            .map(|data| data.iter().map(label_from_value).collect())
            .unwrap_or_default(),
        min: object.get("min").and_then(Value::as_f64),
        max: object.get("max").and_then(Value::as_f64),
        boundary_gap: object
            .get("boundaryGap")
            .and_then(Value::as_bool)
            .unwrap_or(matches!(axis_type, AxisType::Category)),
        inverse: object
            .get("inverse")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        split_number: object
            .get("splitNumber")
            .and_then(Value::as_u64)
            .map(|value| value as usize)
            .unwrap_or(5)
            .max(1),
        show: object.get("show").and_then(Value::as_bool).unwrap_or(true),
        position: object
            .get("position")
            .and_then(Value::as_str)
            .unwrap_or(if matches!(orientation, AxisOrientation::X) {
                "bottom"
            } else {
                "left"
            })
            .to_string(),
        offset: object
            .get("offset")
            .and_then(parse_f32)
            .unwrap_or_default()
            .max(0.0),
        axis_line,
        axis_tick,
        split_line,
        split_line_style,
        axis_label,
        axis_label_style,
        grid_index: object.get("gridIndex").and_then(Value::as_u64).unwrap_or(0) as usize,
    }
}

fn parse_axis_line(value: Option<&Value>, default: &AxisLine) -> AxisLine {
    let Some(object) = value.and_then(Value::as_object) else {
        return default.clone();
    };
    AxisLine {
        show: object
            .get("show")
            .and_then(Value::as_bool)
            .unwrap_or(default.show),
        on_zero: object
            .get("onZero")
            .and_then(Value::as_bool)
            .unwrap_or(default.on_zero),
        line_style: object
            .get("lineStyle")
            .map(|value| parse_line_style_with_default(value, &default.line_style))
            .unwrap_or_else(|| default.line_style.clone()),
    }
}

fn parse_axis_tick(value: Option<&Value>, default: &AxisTick) -> AxisTick {
    let Some(object) = value.and_then(Value::as_object) else {
        return default.clone();
    };
    AxisTick {
        show: object
            .get("show")
            .and_then(Value::as_bool)
            .unwrap_or(default.show),
        align_with_label: object
            .get("alignWithLabel")
            .and_then(Value::as_bool)
            .unwrap_or(default.align_with_label),
        inside: object
            .get("inside")
            .and_then(Value::as_bool)
            .unwrap_or(default.inside),
        length: object
            .get("length")
            .and_then(parse_f32)
            .unwrap_or(default.length)
            .max(0.0),
        line_style: object
            .get("lineStyle")
            .map(|value| parse_line_style_with_default(value, &default.line_style))
            .unwrap_or_else(|| default.line_style.clone()),
    }
}

fn parse_axis_label(value: Option<&Value>, default: &AxisLabelStyle) -> (bool, AxisLabelStyle) {
    let Some(object) = value.and_then(Value::as_object) else {
        return (true, default.clone());
    };
    let font_weight = object
        .get("fontWeight")
        .and_then(parse_font_weight)
        .unwrap_or(default.font_weight);
    (
        object.get("show").and_then(Value::as_bool).unwrap_or(true),
        AxisLabelStyle {
            color: object.get("color").and_then(parse_color).or(default.color),
            font_size: object
                .get("fontSize")
                .and_then(parse_f32)
                .unwrap_or(default.font_size)
                .max(1.0),
            font_weight,
            rotate: object
                .get("rotate")
                .and_then(parse_f32)
                .unwrap_or(default.rotate),
            margin: object
                .get("margin")
                .and_then(parse_f32)
                .unwrap_or(default.margin)
                .max(0.0),
            interval: object
                .get("interval")
                .and_then(Value::as_u64)
                .map(|value| value as usize),
            formatter: object
                .get("formatter")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned),
        },
    )
}

fn parse_split_line(
    value: Option<&Value>,
    orientation: AxisOrientation,
    default: &LineStyle,
) -> (bool, LineStyle) {
    let default_show = !matches!(orientation, AxisOrientation::X);
    let Some(object) = value.and_then(Value::as_object) else {
        return (default_show, default.clone());
    };
    (
        object
            .get("show")
            .and_then(Value::as_bool)
            .unwrap_or(default_show),
        object
            .get("lineStyle")
            .map(|value| parse_line_style_with_default(value, default))
            .unwrap_or_else(|| default.clone()),
    )
}

fn parse_dataset_list(value: Value) -> Vec<Dataset> {
    let values = match value {
        Value::Array(values) if values.iter().all(Value::is_object) => values,
        value => vec![value],
    };
    let mut datasets = Vec::with_capacity(values.len());
    for value in values {
        if let Some(dataset) = parse_dataset_definition(value, &datasets) {
            datasets.push(dataset);
        }
    }
    datasets
}

fn parse_dataset_definition(value: Value, upstreams: &[Dataset]) -> Option<Dataset> {
    let mut object = match value {
        Value::Object(object) => object,
        source => {
            let (source, dimensions, source_header) = parse_dataset_source(&source, &[], None)?;
            return Some(Dataset {
                source,
                dimensions,
                source_header,
                id: None,
                extra: BTreeMap::new(),
            });
        }
    };
    let id = object
        .get("id")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);
    let declared_dimensions = object
        .get("dimensions")
        .and_then(Value::as_array)
        .map(|dimensions| {
            dimensions
                .iter()
                .map(|dimension| {
                    dimension
                        .as_object()
                        .and_then(|dimension| dimension.get("name"))
                        .map(label_from_value)
                        .unwrap_or_else(|| label_from_value(dimension))
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let source_header = object.get("sourceHeader").and_then(Value::as_bool);
    let mut dataset = if let Some(source) = object.get("source") {
        let (source, dimensions, source_header) =
            parse_dataset_source(source, &declared_dimensions, source_header)?;
        Dataset {
            source,
            dimensions,
            source_header,
            id: id.clone(),
            extra: BTreeMap::new(),
        }
    } else {
        let upstream = object
            .get("fromDatasetId")
            .and_then(Value::as_str)
            .and_then(|id| {
                upstreams
                    .iter()
                    .find(|dataset| dataset.id.as_deref() == Some(id))
            })
            .or_else(|| {
                let index = object
                    .get("fromDatasetIndex")
                    .and_then(Value::as_u64)
                    .unwrap_or(0) as usize;
                upstreams.get(index)
            })?
            .clone();
        Dataset {
            id: id.clone(),
            dimensions: if declared_dimensions.is_empty() {
                upstream.dimensions.clone()
            } else {
                declared_dimensions.clone()
            },
            ..upstream
        }
    };
    if let Some(transform) = object.get("transform") {
        apply_dataset_transforms(&mut dataset, transform);
    }
    for key in [
        "id",
        "source",
        "sourceHeader",
        "dimensions",
        "fromDatasetIndex",
        "fromDatasetId",
        "transform",
    ] {
        object.remove(key);
    }
    dataset.extra.extend(object);
    Some(dataset)
}

fn parse_dataset_source(
    source: &Value,
    declared_dimensions: &[String],
    explicit_header: Option<bool>,
) -> Option<(Vec<Vec<DataValue>>, Vec<String>, bool)> {
    let rows = source.as_array()?;
    if rows.first().is_some_and(Value::is_object) {
        let dimensions = if declared_dimensions.is_empty() {
            rows.first()?
                .as_object()?
                .keys()
                .cloned()
                .collect::<Vec<_>>()
        } else {
            declared_dimensions.to_vec()
        };
        let source = rows
            .iter()
            .filter_map(Value::as_object)
            .map(|row| {
                dimensions
                    .iter()
                    .map(|dimension| {
                        row.get(dimension)
                            .map(parse_json_data_value)
                            .unwrap_or(DataValue::Null)
                    })
                    .collect()
            })
            .collect();
        return Some((source, dimensions, false));
    }
    let source = rows
        .iter()
        .filter_map(|row| {
            row.as_array()
                .map(|cols| cols.iter().map(parse_json_data_value).collect::<Vec<_>>())
        })
        .collect::<Vec<_>>();
    let detected_header = source.first().is_some_and(|first| {
        let matches_dimensions = !declared_dimensions.is_empty()
            && first.len() == declared_dimensions.len()
            && first
                .iter()
                .map(data_value_label)
                .eq(declared_dimensions.iter().cloned());
        matches_dimensions
            || (first
                .iter()
                .all(|value| matches!(value, DataValue::String(_)))
                && source.get(1).is_some_and(|row| {
                    row.iter()
                        .any(|value| !matches!(value, DataValue::String(_)))
                }))
    });
    let source_header = explicit_header.unwrap_or(detected_header);
    let dimensions = if declared_dimensions.is_empty() && source_header {
        source
            .first()
            .map(|row| row.iter().map(data_value_label).collect())
            .unwrap_or_default()
    } else {
        declared_dimensions.to_vec()
    };
    Some((source, dimensions, source_header))
}

fn apply_dataset_transforms(dataset: &mut Dataset, value: &Value) {
    if let Some(transforms) = value.as_array() {
        for transform in transforms {
            apply_dataset_transform(dataset, transform);
        }
    } else {
        apply_dataset_transform(dataset, value);
    }
}

fn apply_dataset_transform(dataset: &mut Dataset, value: &Value) {
    let Some(transform) = value.as_object() else {
        return;
    };
    let kind = transform
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let config = transform.get("config").unwrap_or(&Value::Null);
    match kind {
        "filter" => filter_dataset(dataset, config),
        "sort" => sort_dataset(dataset, config),
        _ => {
            dataset.extra.insert(
                String::from("unsupportedTransform"),
                Value::String(kind.to_string()),
            );
        }
    }
}

fn filter_dataset(dataset: &mut Dataset, config: &Value) {
    let header = usize::from(dataset.source_header);
    let mut source = dataset.source[..header.min(dataset.source.len())].to_vec();
    source.extend(
        dataset.source[header.min(dataset.source.len())..]
            .iter()
            .filter(|row| dataset_filter_matches(dataset, row, config))
            .cloned(),
    );
    dataset.source = source;
}

fn dataset_filter_matches(dataset: &Dataset, row: &[DataValue], condition: &Value) -> bool {
    if let Some(value) = condition.as_bool() {
        return value;
    }
    let Some(condition) = condition.as_object() else {
        return false;
    };
    if let Some(conditions) = condition.get("and").and_then(Value::as_array) {
        return conditions
            .iter()
            .all(|condition| dataset_filter_matches(dataset, row, condition));
    }
    if let Some(conditions) = condition.get("or").and_then(Value::as_array) {
        return conditions
            .iter()
            .any(|condition| dataset_filter_matches(dataset, row, condition));
    }
    if let Some(condition) = condition.get("not") {
        return !dataset_filter_matches(dataset, row, condition);
    }
    let Some(dimension) = condition
        .get("dimension")
        .and_then(|dimension| dataset_dimension_index(dataset, dimension))
    else {
        return false;
    };
    let Some(left) = row.get(dimension) else {
        return false;
    };
    let parser = condition.get("parser").and_then(Value::as_str);
    let operators = [
        (["gt", ">"], "gt"),
        (["gte", ">="], "gte"),
        (["lt", "<"], "lt"),
        (["lte", "<="], "lte"),
        (["eq", "="], "eq"),
        (["ne", "!="], "ne"),
        (["ne", "<>"], "ne"),
    ];
    let mut compared = false;
    for (aliases, operator) in operators {
        if let Some(right) = aliases.iter().find_map(|key| condition.get(*key)) {
            compared = true;
            if !dataset_compare(left, right, parser, operator) {
                return false;
            }
        }
    }
    if let Some(right) = condition.get("value") {
        compared = true;
        if !dataset_compare(left, right, parser, "eq") {
            return false;
        }
    }
    if let Some(pattern) = condition.get("reg").and_then(Value::as_str) {
        compared = true;
        if !data_value_label(left).contains(pattern) {
            return false;
        }
    }
    compared
}

fn dataset_compare(left: &DataValue, right: &Value, parser: Option<&str>, operator: &str) -> bool {
    let right = parse_json_data_value(right);
    let numeric = match parser {
        Some("trim") => None,
        Some("number") => Some((loose_number(left), loose_number(&right))),
        Some("time") => Some((left.as_f64(), right.as_f64())),
        _ => Some((left.as_f64(), right.as_f64())),
    };
    if let Some((Some(left), Some(right))) = numeric {
        return match operator {
            "gt" => left > right,
            "gte" => left >= right,
            "lt" => left < right,
            "lte" => left <= right,
            "eq" => (left - right).abs() < 1e-12,
            "ne" => (left - right).abs() >= 1e-12,
            _ => false,
        };
    }
    let left = match (parser, left) {
        (Some("trim"), DataValue::String(value)) => value.trim().to_string(),
        _ => data_value_label(left),
    };
    let right = match (parser, &right) {
        (Some("trim"), DataValue::String(value)) => value.trim().to_string(),
        _ => data_value_label(&right),
    };
    match operator {
        "eq" => left == right,
        "ne" => left != right,
        _ => false,
    }
}

fn loose_number(value: &DataValue) -> Option<f64> {
    match value {
        DataValue::Number(value) => Some(*value),
        DataValue::String(value) => {
            let value = value.trim();
            let end = value
                .char_indices()
                .find_map(|(index, character)| {
                    (!(character.is_ascii_digit()
                        || matches!(character, '+' | '-' | '.' | 'e' | 'E')))
                    .then_some(index)
                })
                .unwrap_or(value.len());
            value[..end].parse().ok()
        }
        DataValue::Null => None,
    }
}

fn sort_dataset(dataset: &mut Dataset, config: &Value) {
    let rules = config
        .as_array()
        .cloned()
        .unwrap_or_else(|| vec![config.clone()]);
    let header = usize::from(dataset.source_header).min(dataset.source.len());
    let dimensions = dataset.dimensions.clone();
    dataset.source[header..].sort_by(|left, right| {
        for rule in &rules {
            let Some(rule) = rule.as_object() else {
                continue;
            };
            let Some(index) = rule
                .get("dimension")
                .and_then(|dimension| dataset_dimension_index_from(&dimensions, dimension))
            else {
                continue;
            };
            let parser = rule.get("parser").and_then(Value::as_str);
            let incomparable = rule
                .get("incomparable")
                .and_then(Value::as_str)
                .unwrap_or("max");
            let order =
                dataset_sort_compare(left.get(index), right.get(index), parser, incomparable);
            if order != std::cmp::Ordering::Equal {
                return if rule.get("order").and_then(Value::as_str) == Some("desc") {
                    order.reverse()
                } else {
                    order
                };
            }
        }
        std::cmp::Ordering::Equal
    });
}

fn dataset_sort_compare(
    left: Option<&DataValue>,
    right: Option<&DataValue>,
    parser: Option<&str>,
    incomparable: &str,
) -> std::cmp::Ordering {
    let number = |value: &DataValue| match parser {
        Some("number") => loose_number(value),
        Some("trim") => None,
        _ => value.as_f64(),
    };
    match (left, right) {
        (Some(left), Some(right)) => match (number(left), number(right)) {
            (Some(left), Some(right)) => left.total_cmp(&right),
            (Some(_), None) => {
                if incomparable == "min" {
                    std::cmp::Ordering::Greater
                } else {
                    std::cmp::Ordering::Less
                }
            }
            (None, Some(_)) => {
                if incomparable == "min" {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Greater
                }
            }
            (None, None) => {
                let left = data_value_label(left);
                let right = data_value_label(right);
                if parser == Some("trim") {
                    left.trim().cmp(right.trim())
                } else {
                    left.cmp(&right)
                }
            }
        },
        (None, None) => std::cmp::Ordering::Equal,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (Some(_), None) => std::cmp::Ordering::Less,
    }
}

fn dataset_dimension_index(dataset: &Dataset, value: &Value) -> Option<usize> {
    dataset_dimension_index_from(&dataset.dimensions, value)
}

fn dataset_dimension_index_from(dimensions: &[String], value: &Value) -> Option<usize> {
    value.as_u64().map(|value| value as usize).or_else(|| {
        value
            .as_str()
            .and_then(|name| dimensions.iter().position(|dimension| dimension == name))
    })
}

fn parse_data_zoom_list(value: Value) -> Vec<DataZoom> {
    match value {
        Value::Array(values) => values.into_iter().filter_map(parse_data_zoom).collect(),
        value => parse_data_zoom(value).into_iter().collect(),
    }
}

fn parse_data_zoom(value: Value) -> Option<DataZoom> {
    let Value::Object(object) = value else {
        return None;
    };
    let kind = object
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or("slider")
        .to_string();
    let mut extra = object.clone();
    for key in [
        "show",
        "type",
        "start",
        "end",
        "startValue",
        "endValue",
        "xAxisIndex",
        "yAxisIndex",
        "filterMode",
        "orient",
        "zoomLock",
        "height",
    ] {
        extra.remove(key);
    }
    Some(DataZoom {
        show: object.get("show").and_then(Value::as_bool).unwrap_or(true),
        kind,
        start: object
            .get("start")
            .and_then(Value::as_f64)
            .unwrap_or(0.0)
            .clamp(0.0, 100.0),
        end: object
            .get("end")
            .and_then(Value::as_f64)
            .unwrap_or(100.0)
            .clamp(0.0, 100.0),
        start_value: object.get("startValue").and_then(parse_data_value),
        end_value: object.get("endValue").and_then(parse_data_value),
        x_axis_index: parse_index_list(object.get("xAxisIndex"), true),
        y_axis_index: parse_index_list(object.get("yAxisIndex"), false),
        filter_mode: object
            .get("filterMode")
            .and_then(Value::as_str)
            .unwrap_or("filter")
            .to_string(),
        orient: object
            .get("orient")
            .and_then(Value::as_str)
            .unwrap_or("horizontal")
            .to_string(),
        zoom_lock: object
            .get("zoomLock")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        height: object
            .get("height")
            .and_then(parse_f32)
            .unwrap_or(20.0)
            .max(4.0),
        extra: extra.into_iter().collect(),
    })
}

fn parse_index_list(value: Option<&Value>, default_zero: bool) -> Vec<usize> {
    let values = match value {
        Some(Value::Array(values)) => values
            .iter()
            .filter_map(Value::as_u64)
            .map(|value| value as usize)
            .collect(),
        Some(value) => value
            .as_u64()
            .map(|value| vec![value as usize])
            .unwrap_or_default(),
        None => Vec::new(),
    };
    if values.is_empty() && default_zero {
        vec![0]
    } else {
        values
    }
}

fn parse_data_value(value: &Value) -> Option<DataValue> {
    value.as_f64().map(DataValue::Number).or_else(|| {
        value
            .as_str()
            .map(|value| DataValue::String(value.to_string()))
    })
}

fn parse_radar_list(value: Value) -> Vec<RadarCoordinate> {
    match value {
        Value::Array(values) => values.into_iter().filter_map(parse_radar).collect(),
        value => parse_radar(value).into_iter().collect(),
    }
}

fn parse_radar(value: Value) -> Option<RadarCoordinate> {
    let object = value.as_object()?;
    let mut extra = object.clone();
    for key in [
        "indicator",
        "center",
        "radius",
        "startAngle",
        "splitNumber",
        "shape",
    ] {
        extra.remove(key);
    }
    let indicators = object
        .get("indicator")
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .enumerate()
                .filter_map(|(index, value)| {
                    let value = value.as_object()?;
                    Some(RadarIndicator {
                        name: value
                            .get("name")
                            .and_then(Value::as_str)
                            .unwrap_or(if index == 0 { "indicator" } else { "" })
                            .to_string(),
                        min: value.get("min").and_then(Value::as_f64).unwrap_or(0.0),
                        max: value.get("max").and_then(Value::as_f64).unwrap_or(100.0),
                        color: value.get("color").and_then(parse_color),
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    let center = object
        .get("center")
        .and_then(Value::as_array)
        .and_then(|values| Some([values.first()?.clone(), values.get(1)?.clone()]))
        .unwrap_or([Value::String("50%".into()), Value::String("50%".into())]);
    Some(RadarCoordinate {
        indicators,
        center,
        radius: object
            .get("radius")
            .cloned()
            .unwrap_or_else(|| Value::String("75%".into())),
        start_angle: object
            .get("startAngle")
            .and_then(Value::as_f64)
            .unwrap_or(90.0) as f32,
        split_number: object
            .get("splitNumber")
            .and_then(Value::as_u64)
            .unwrap_or(5) as usize,
        shape: object
            .get("shape")
            .and_then(Value::as_str)
            .unwrap_or("polygon")
            .to_string(),
        extra: extra.into_iter().collect(),
    })
}

fn parse_visual_map(value: Value) -> Option<VisualMap> {
    let object = value.as_object()?;
    let colors = object
        .get("inRange")
        .and_then(Value::as_object)
        .and_then(|value| value.get("color"))
        .and_then(Value::as_array)
        .map(|values| values.iter().filter_map(parse_color).collect())
        .unwrap_or_else(|| vec![0xFF50A3BA, 0xFFEAC736, 0xFFD94E5D]);
    Some(VisualMap {
        show: object.get("show").and_then(Value::as_bool).unwrap_or(true),
        min: object.get("min").and_then(Value::as_f64).unwrap_or(0.0),
        max: object.get("max").and_then(Value::as_f64).unwrap_or(200.0),
        colors,
        dimension: object
            .get("dimension")
            .and_then(Value::as_u64)
            .map(|value| value as usize),
        symbol_size_range: object
            .get("inRange")
            .and_then(Value::as_object)
            .and_then(|value| value.get("symbolSize"))
            .and_then(parse_symbol_dimensions),
        pieces: object
            .get("pieces")
            .and_then(Value::as_array)
            .map(|pieces| pieces.iter().filter_map(parse_visual_piece).collect())
            .unwrap_or_default(),
        series_indices: parse_index_list(object.get("seriesIndex"), false),
    })
}

fn parse_visual_map_list(value: Value) -> Vec<VisualMap> {
    match value {
        Value::Array(values) => values.into_iter().filter_map(parse_visual_map).collect(),
        value => parse_visual_map(value).into_iter().collect(),
    }
}

fn parse_visual_piece(value: &Value) -> Option<VisualPiece> {
    let object = value.as_object()?;
    let exclusive_min = object
        .get("gt")
        .and_then(Value::as_f64)
        .map(|value| value + f64::EPSILON);
    let exclusive_max = object
        .get("lt")
        .and_then(Value::as_f64)
        .map(|value| value - f64::EPSILON);
    Some(VisualPiece {
        min: object
            .get("min")
            .or_else(|| object.get("gte"))
            .and_then(Value::as_f64)
            .or(exclusive_min),
        max: object
            .get("max")
            .or_else(|| object.get("lte"))
            .and_then(Value::as_f64)
            .or(exclusive_max),
        value: object.get("value").and_then(Value::as_f64),
        color: object.get("color").and_then(parse_color),
        symbol_size: object.get("symbolSize").and_then(parse_f32),
        label: object
            .get("label")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
    })
}

fn parse_color_palette(value: Value) -> Option<Vec<u32>> {
    value
        .as_array()
        .map(|values| values.iter().filter_map(parse_color).collect())
}

fn parse_series_list(value: Value) -> (Vec<Series>, Vec<Diagnostic>) {
    let mut diagnostics = Vec::new();
    let series = value
        .as_array()
        .cloned()
        .unwrap_or_else(|| vec![value])
        .into_iter()
        .filter_map(|value| parse_series(value, &mut diagnostics))
        .collect();
    (series, diagnostics)
}

fn parse_series(value: Value, diagnostics: &mut Vec<Diagnostic>) -> Option<Series> {
    let Value::Object(object) = value else {
        return None;
    };
    let kind = object.get("type").and_then(Value::as_str).unwrap_or("line");
    let name = object
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or(kind)
        .to_string();
    let data: Vec<DataPoint> = object
        .get("data")
        .and_then(Value::as_array)
        .map(|items| items.iter().map(parse_data_point).collect())
        .unwrap_or_default();
    let basic = || BasicSeries {
        name: Some(name.clone()),
        data: data.clone(),
        style: None,
        options: parse_series_options(&object, kind),
    };
    Some(match kind {
        "line" => Series::Line(basic()),
        "bar" => Series::Bar(basic()),
        "pie" => Series::Pie(basic()),
        "scatter" => Series::Scatter(basic()),
        "effectScatter" => Series::EffectScatter(basic()),
        "radar" => Series::Radar(basic()),
        "gauge" => Series::Gauge(basic()),
        "funnel" => Series::Funnel(basic()),
        "heatmap" => Series::Heatmap(basic()),
        "candlestick" => Series::Candlestick(basic()),
        "boxplot" => Series::Boxplot(basic()),
        "pictorialBar" => Series::PictorialBar(basic()),
        "parallel" => Series::Parallel(basic()),
        "themeRiver" => Series::ThemeRiver(basic()),
        "treemap" => Series::Treemap(basic()),
        "tree" => Series::Tree(parse_graph_series(name, &object, kind)),
        "graph" => Series::Graph(parse_graph_series(name, &object, kind)),
        "sankey" => Series::Sankey(parse_sankey_series(name, &object)),
        "map" => Series::Map(parse_map_series(name, &object)),
        "lines" => Series::Lines(parse_lines_series(name, &object)),
        "sunburst" => Series::Sunburst(parse_sunburst_series(name, &object)),
        "custom" => {
            diagnostics.push(Diagnostic {
                field: String::from("series.custom"),
                message: String::from(
                    "JSON custom series is parsed but not executed; use typed Rust custom renderer",
                ),
            });
            Series::Custom(CustomSeries {
                name: Some(name),
                data,
                renderer: Rc::new(|_| {}),
            })
        }
        other => {
            diagnostics.push(Diagnostic {
                field: format!("series.{other}"),
                message: String::from("unsupported series type"),
            });
            return None;
        }
    })
}

fn parse_lines_series(name: String, object: &serde_json::Map<String, Value>) -> LinesSeries {
    let data = object
        .get("data")
        .and_then(Value::as_array)
        .map(|values| values.iter().filter_map(parse_line_segment).collect())
        .unwrap_or_default();
    let mut options = parse_series_options(object, "lines");
    if let Some(symbol) = object.get("symbol") {
        options.extra.insert(String::from("symbol"), symbol.clone());
    }
    LinesSeries {
        name: Some(name),
        data,
        options,
    }
}

fn parse_line_segment(value: &Value) -> Option<LineSegment> {
    let object = value.as_object()?;
    let coords = object.get("coords")?.as_array()?;
    let coords = coords
        .iter()
        .filter_map(|point| {
            let point = point.as_array()?;
            Some((point.first()?.as_f64()?, point.get(1)?.as_f64()?))
        })
        .collect::<Vec<_>>();
    let from = *coords.first()?;
    let to = *coords.last()?;
    if coords.len() < 2 {
        return None;
    }
    Some(LineSegment {
        name: object
            .get("name")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        from,
        to,
        coords,
        value: object.get("value").and_then(Value::as_f64).unwrap_or(1.0),
    })
}

fn parse_sunburst_series(name: String, object: &serde_json::Map<String, Value>) -> SunburstSeries {
    let data = object
        .get("data")
        .and_then(Value::as_array)
        .map(|values| values.iter().filter_map(parse_sunburst_node).collect())
        .unwrap_or_default();
    let mut options = parse_series_options(object, "sunburst");
    if !object.contains_key("label") {
        options.label.show = true;
        options.label.formatter = Some(String::from("{b}"));
    }
    SunburstSeries {
        name: Some(name),
        data,
        options,
    }
}

fn parse_sunburst_node(value: &Value) -> Option<SunburstNode> {
    let object = value.as_object()?;
    let children: Vec<SunburstNode> = object
        .get("children")
        .and_then(Value::as_array)
        .map(|values| values.iter().filter_map(parse_sunburst_node).collect())
        .unwrap_or_default();
    let children_total: f64 = children.iter().map(|child| child.value).sum();
    Some(SunburstNode {
        name: object
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        value: object
            .get("value")
            .and_then(Value::as_f64)
            .unwrap_or(children_total.max(1.0)),
        children,
        item_style: object
            .get("itemStyle")
            .map(parse_item_style)
            .unwrap_or_default(),
    })
}

fn parse_graph_series(
    name: String,
    object: &serde_json::Map<String, Value>,
    kind: &str,
) -> GraphSeries {
    let (nodes, links) = if kind == "tree" {
        parse_tree_data(object.get("data"))
    } else {
        let nodes: Vec<NodeData> = object
            .get("data")
            .or_else(|| object.get("nodes"))
            .and_then(Value::as_array)
            .map(|nodes| nodes.iter().enumerate().map(parse_node_data).collect())
            .unwrap_or_default();
        let links = object
            .get("links")
            .or_else(|| object.get("edges"))
            .and_then(Value::as_array)
            .map(|links| {
                links
                    .iter()
                    .filter_map(|value| parse_link_data(value, &nodes))
                    .collect()
            })
            .unwrap_or_default();
        (nodes, links)
    };
    let mut options = parse_series_options(object, kind);
    if kind == "tree" && !object.contains_key("label") {
        options.label.show = true;
        options.label.formatter = Some(String::from("{b}"));
    }
    GraphSeries {
        name: Some(name),
        nodes,
        links,
        options,
    }
}

fn parse_tree_data(value: Option<&Value>) -> (Vec<NodeData>, Vec<LinkData>) {
    let mut nodes = Vec::new();
    let mut links = Vec::new();
    for root in value.and_then(Value::as_array).into_iter().flatten() {
        flatten_tree_node(root, None, &mut nodes, &mut links);
    }
    (nodes, links)
}

fn flatten_tree_node(
    value: &Value,
    parent: Option<usize>,
    nodes: &mut Vec<NodeData>,
    links: &mut Vec<LinkData>,
) {
    let index = nodes.len();
    nodes.push(parse_node_data((index, value)));
    if let Some(parent) = parent {
        links.push(LinkData {
            source: parent,
            target: index,
            value: 1.0,
        });
    }
    if let Some(children) = value
        .as_object()
        .and_then(|value| value.get("children"))
        .and_then(Value::as_array)
    {
        for child in children {
            flatten_tree_node(child, Some(index), nodes, links);
        }
    }
}

fn parse_sankey_series(name: String, object: &serde_json::Map<String, Value>) -> SankeySeries {
    let graph = parse_graph_series(name, object, "sankey");
    SankeySeries {
        name: graph.name,
        nodes: graph.nodes,
        links: graph.links,
        options: graph.options,
    }
}

fn parse_map_series(name: String, object: &serde_json::Map<String, Value>) -> MapSeries {
    let map_options = parse_map_options(object);
    let mut features = object
        .get("geoJson")
        .or_else(|| object.get("geoJSON"))
        .or_else(|| object.get("features"))
        .and_then(parse_geo_features)
        .or_else(|| {
            object
                .get("map")
                .and_then(Value::as_str)
                .and_then(crate::registry::registered_map)
        })
        .unwrap_or_default();

    for feature in &mut features {
        if map_options.name_property != "name" {
            if let Some(name) = feature
                .properties
                .get(&map_options.name_property)
                .map(label_from_value)
                .filter(|name| !name.is_empty())
            {
                feature.name = name;
            }
        }
        if let Some(mapped) = map_options.name_map.get(&feature.name) {
            feature.name = mapped.clone();
        }
    }

    if let Some(regions) = object.get("regions").and_then(Value::as_array) {
        apply_map_feature_overrides(&mut features, regions, &map_options.name_map);
    }
    if let Some(data) = object.get("data").and_then(Value::as_array) {
        apply_map_feature_overrides(&mut features, data, &map_options.name_map);
    }
    MapSeries {
        name: Some(name),
        features,
        options: parse_series_options(object, "map"),
        map_options: Box::new(map_options),
    }
}

fn apply_map_feature_overrides(
    features: &mut [MapFeature],
    values: &[Value],
    name_map: &std::collections::BTreeMap<String, String>,
) {
    for value in values {
        let Some(object) = value.as_object() else {
            continue;
        };
        let Some(raw_name) = object.get("name").and_then(Value::as_str) else {
            continue;
        };
        let name = name_map
            .get(raw_name)
            .map(String::as_str)
            .unwrap_or(raw_name);
        let Some(feature) = features.iter_mut().find(|feature| feature.name == name) else {
            continue;
        };
        if let Some(value) = object.get("value") {
            feature.value = value.as_f64().filter(|value| value.is_finite());
        }
        if let Some(selected) = object.get("selected").and_then(Value::as_bool) {
            feature.selected = selected;
        }
        if let Some(value) = object.get("itemStyle") {
            feature.item_style = parse_item_style_with_default(value, &feature.item_style);
        }
        if let Some(value) = object.get("label") {
            feature.label = parse_label_style(value);
        }
        apply_map_state(
            object.get("emphasis"),
            &mut feature.emphasis_item_style,
            &mut feature.emphasis_label,
        );
        apply_map_state(
            object.get("select"),
            &mut feature.select_item_style,
            &mut feature.select_label,
        );
    }
}

fn apply_map_state(value: Option<&Value>, item_style: &mut ItemStyle, label: &mut LabelStyle) {
    let Some(object) = value.and_then(Value::as_object) else {
        return;
    };
    if let Some(value) = object.get("itemStyle") {
        *item_style = parse_item_style_with_default(value, item_style);
    }
    if let Some(value) = object.get("label") {
        *label = parse_label_style(value);
    }
}

pub(crate) fn parse_map_options(object: &serde_json::Map<String, Value>) -> MapOptions {
    let mut options = MapOptions::default();
    options.left = object.get("left").cloned().unwrap_or(options.left);
    options.top = object.get("top").cloned().unwrap_or(options.top);
    options.right = object.get("right").cloned();
    options.bottom = object.get("bottom").cloned();
    options.width = object.get("width").cloned();
    options.height = object.get("height").cloned();
    options.layout_center = object
        .get("layoutCenter")
        .and_then(Value::as_array)
        .and_then(|values| Some([values.first()?.clone(), values.get(1)?.clone()]));
    options.layout_size = object.get("layoutSize").cloned();
    options.aspect_scale = object
        .get("aspectScale")
        .and_then(parse_f32)
        .unwrap_or(0.75)
        .max(1e-6);
    options.center = object.get("center").and_then(parse_coordinate);
    options.zoom = object
        .get("zoom")
        .and_then(parse_f32)
        .unwrap_or(1.0)
        .max(1e-6);
    options.scale_limit = object
        .get("scaleLimit")
        .and_then(Value::as_object)
        .map(|limit| {
            let min = limit.get("min").and_then(parse_f32).unwrap_or(0.0);
            let max = limit.get("max").and_then(parse_f32).unwrap_or(f32::MAX);
            (min.min(max), max.max(min))
        });
    options.bounding_coords = object
        .get("boundingCoords")
        .and_then(Value::as_array)
        .and_then(|values| {
            Some([
                parse_coordinate(values.first()?)?,
                parse_coordinate(values.get(1)?)?,
            ])
        });
    options.roam = object
        .get("roam")
        .map(|value| match value {
            Value::Bool(value) => value.to_string(),
            Value::String(value) => value.clone(),
            _ => String::from("false"),
        })
        .unwrap_or_else(|| String::from("false"));
    options.name_property = object
        .get("nameProperty")
        .and_then(Value::as_str)
        .unwrap_or("name")
        .to_string();
    options.name_map = object
        .get("nameMap")
        .and_then(Value::as_object)
        .map(|values| {
            values
                .iter()
                .filter_map(|(key, value)| Some((key.clone(), value.as_str()?.to_string())))
                .collect()
        })
        .unwrap_or_default();
    if let Some(state) = object.get("emphasis").and_then(Value::as_object) {
        if let Some(value) = state.get("itemStyle") {
            options.emphasis_item_style =
                parse_item_style_with_default(value, &options.emphasis_item_style);
        }
        if let Some(value) = state.get("label") {
            options.emphasis_label = parse_label_style(value);
        }
    }
    if let Some(state) = object.get("select").and_then(Value::as_object) {
        if let Some(value) = state.get("itemStyle") {
            options.select_item_style =
                parse_item_style_with_default(value, &options.select_item_style);
        }
        if let Some(value) = state.get("label") {
            options.select_label = parse_label_style(value);
        }
    }
    options
}

fn parse_coordinate(value: &Value) -> Option<(f64, f64)> {
    let values = value.as_array()?;
    Some((values.first()?.as_f64()?, values.get(1)?.as_f64()?))
}

pub(crate) fn parse_geo_features(value: &Value) -> Option<Vec<MapFeature>> {
    let features = match value {
        Value::Object(object) => object.get("features")?.as_array()?,
        Value::Array(values) => values,
        _ => return None,
    };
    Some(features.iter().filter_map(parse_geo_feature).collect())
}

fn parse_geo_feature(value: &Value) -> Option<MapFeature> {
    let object = value.as_object()?;
    let properties: std::collections::BTreeMap<String, Value> = object
        .get("properties")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .collect();
    let name = properties
        .get("name")
        .and_then(Value::as_str)
        .or_else(|| object.get("name").and_then(Value::as_str))
        .unwrap_or("feature")
        .to_string();
    let feature_value = object
        .get("value")
        .and_then(Value::as_f64)
        .or_else(|| properties.get("value").and_then(Value::as_f64));
    let geometry = object.get("geometry").unwrap_or(value);
    let polygons = parse_polygons(geometry)?;
    let center = properties
        .get("cp")
        .or_else(|| properties.get("center"))
        .and_then(parse_coordinate);
    let mut feature = MapFeature::new(name, polygons);
    feature.value = feature_value;
    feature.center = center;
    feature.properties = properties;
    Some(feature)
}

fn parse_polygons(value: &Value) -> Option<Vec<MapPolygon>> {
    let object = value.as_object()?;
    let kind = object.get("type").and_then(Value::as_str)?;
    let coordinates = object.get("coordinates")?;
    match kind {
        "Polygon" => parse_polygon_array(coordinates).map(|polygon| vec![polygon]),
        "MultiPolygon" => Some(
            coordinates
                .as_array()?
                .iter()
                .filter_map(parse_polygon_array)
                .collect(),
        ),
        _ => None,
    }
}

fn parse_polygon_array(value: &Value) -> Option<MapPolygon> {
    let mut rings = value
        .as_array()
        .into_iter()
        .flatten()
        .map(|ring| {
            ring.as_array()
                .into_iter()
                .flatten()
                .filter_map(|point| {
                    let point = point.as_array()?;
                    Some((point.first()?.as_f64()?, point.get(1)?.as_f64()?))
                })
                .collect()
        })
        .filter(|ring: &Vec<(f64, f64)>| ring.len() >= 3);
    Some(MapPolygon {
        exterior: rings.next()?,
        holes: rings.collect(),
    })
}

fn parse_node_data((index, value): (usize, &Value)) -> NodeData {
    match value {
        Value::Object(object) => {
            let mut extra = object.clone();
            for key in [
                "name",
                "value",
                "x",
                "y",
                "category",
                "symbol",
                "symbolSize",
                "symbolRotate",
                "itemStyle",
                "label",
                "children",
            ] {
                extra.remove(key);
            }
            NodeData {
                name: object
                    .get("name")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned)
                    .unwrap_or_else(|| format!("node {index}")),
                value: object
                    .get("value")
                    .and_then(|value| {
                        value.as_f64().or_else(|| {
                            value
                                .as_array()
                                .and_then(|values| values.first())
                                .and_then(Value::as_f64)
                        })
                    })
                    .unwrap_or(1.0),
                x: object.get("x").and_then(Value::as_f64),
                y: object.get("y").and_then(Value::as_f64),
                category: object
                    .get("category")
                    .and_then(Value::as_u64)
                    .map(|value| value as usize),
                symbol_size: object.get("symbolSize").and_then(parse_f32),
                symbol_size_dimensions: object.get("symbolSize").and_then(parse_symbol_dimensions),
                symbol: object
                    .get("symbol")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned),
                symbol_rotate: object
                    .get("symbolRotate")
                    .and_then(parse_f32)
                    .unwrap_or(0.0),
                item_style: object
                    .get("itemStyle")
                    .map(parse_item_style)
                    .unwrap_or_default(),
                label: object
                    .get("label")
                    .map(parse_label_style)
                    .unwrap_or_default(),
                extra: extra.into_iter().collect(),
            }
        }
        Value::String(name) => NodeData {
            name: name.clone(),
            value: 1.0,
            x: None,
            y: None,
            category: None,
            symbol_size: None,
            symbol_size_dimensions: None,
            symbol: None,
            symbol_rotate: 0.0,
            item_style: ItemStyle::default(),
            label: LabelStyle::default(),
            extra: BTreeMap::new(),
        },
        _ => NodeData {
            name: format!("node {index}"),
            value: value.as_f64().unwrap_or(1.0),
            x: None,
            y: None,
            category: None,
            symbol_size: None,
            symbol_size_dimensions: None,
            symbol: None,
            symbol_rotate: 0.0,
            item_style: ItemStyle::default(),
            label: LabelStyle::default(),
            extra: BTreeMap::new(),
        },
    }
}

fn parse_link_data(value: &Value, nodes: &[NodeData]) -> Option<LinkData> {
    let object = value.as_object()?;
    Some(LinkData {
        source: parse_node_reference(object.get("source")?, nodes)?,
        target: parse_node_reference(object.get("target")?, nodes)?,
        value: object.get("value").and_then(Value::as_f64).unwrap_or(1.0),
    })
}

fn parse_node_reference(value: &Value, nodes: &[NodeData]) -> Option<usize> {
    value.as_u64().map(|value| value as usize).or_else(|| {
        value
            .as_str()
            .and_then(|name| nodes.iter().position(|node| node.name == name))
    })
}

pub(crate) fn parse_data_point(value: &Value) -> DataPoint {
    match value {
        Value::Number(number) => DataPoint::scalar(number.as_f64().unwrap_or_default()),
        Value::String(_) => DataPoint::scalar(parse_json_data_value(value)),
        Value::Array(values) => DataPoint::values(values.iter().map(parse_json_data_value)),
        Value::Object(object) => {
            let mut point = object
                .get("value")
                .map(parse_data_point)
                .unwrap_or_else(DataPoint::missing);
            point.name = object
                .get("name")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned);
            point.item_style = object
                .get("itemStyle")
                .map(parse_item_style)
                .unwrap_or_default();
            point.label = object
                .get("label")
                .map(parse_label_style)
                .unwrap_or_default();
            let mut extra = object.clone();
            for key in ["value", "name", "itemStyle", "label"] {
                extra.remove(key);
            }
            point.extra = extra.into_iter().collect();
            point
        }
        Value::Null => DataPoint::missing(),
        Value::Bool(_) => DataPoint::scalar(parse_json_data_value(value)),
    }
}

fn parse_json_data_value(value: &Value) -> DataValue {
    match value {
        Value::Null => DataValue::Null,
        Value::Number(value) => value
            .as_f64()
            .filter(|value| value.is_finite())
            .map(DataValue::Number)
            .unwrap_or(DataValue::Null),
        Value::String(value) if value == "-" => DataValue::Null,
        Value::String(value) => DataValue::String(value.clone()),
        Value::Bool(_) | Value::Array(_) | Value::Object(_) => {
            DataValue::String(label_from_value(value))
        }
    }
}

fn parse_series_options(object: &serde_json::Map<String, Value>, kind: &str) -> SeriesOptions {
    let smooth = match object.get("smooth") {
        Some(Value::Bool(true)) => 0.5,
        Some(Value::Number(value)) => value.as_f64().unwrap_or_default() as f32,
        _ => 0.0,
    }
    .clamp(0.0, 1.0);
    let area_style = object.get("areaStyle").map(|value| {
        let mut style = parse_item_style(value);
        if value
            .as_object()
            .is_none_or(|object| !object.contains_key("opacity"))
        {
            style.opacity = 0.7;
        }
        style
    });
    let mut extra = object.clone();
    for key in [
        "type",
        "name",
        "data",
        "itemStyle",
        "lineStyle",
        "areaStyle",
        "label",
        "smooth",
        "smoothMonotone",
        "connectNulls",
        "showSymbol",
        "showAllSymbol",
        "symbol",
        "symbolSize",
        "symbolRotate",
        "symbolOffset",
        "step",
        "clip",
        "sampling",
        "endLabel",
        "barWidth",
        "barMaxWidth",
        "barMinWidth",
        "barMinHeight",
        "barGap",
        "barCategoryGap",
        "showBackground",
        "backgroundStyle",
        "stack",
        "selectedMode",
        "emphasis",
        "blur",
        "select",
        "labelLayout",
    ] {
        extra.remove(key);
    }
    let mut label = object
        .get("label")
        .map(parse_label_style)
        .unwrap_or_default();
    if !object.contains_key("label") && matches!(kind, "pie" | "funnel" | "treemap") {
        label.show = true;
        label.formatter = Some(String::from("{b}"));
    }
    let mut item_style_default = ItemStyle::default();
    if kind == "map" {
        item_style_default.color = Some(0xFFEEEEEE);
        item_style_default.border_color = Some(0xFF444444);
        item_style_default.border_width = 0.5;
    }
    let item_style = object
        .get("itemStyle")
        .map(|value| parse_item_style_with_default(value, &item_style_default))
        .unwrap_or(item_style_default);
    let end_label = object
        .get("endLabel")
        .map(parse_end_label_style)
        .unwrap_or_else(|| LabelStyle {
            position: String::from("right"),
            distance: 8.0,
            ..LabelStyle::default()
        });
    let symbol_offset = object
        .get("symbolOffset")
        .and_then(Value::as_array)
        .map(|values| {
            [
                values.first().cloned().unwrap_or(Value::from(0)),
                values.get(1).cloned().unwrap_or(Value::from(0)),
            ]
        })
        .unwrap_or([Value::from(0), Value::from(0)]);
    let default_symbol_size = if kind == "line" {
        6.0
    } else if matches!(kind, "scatter" | "effectScatter") {
        10.0
    } else {
        7.0
    };
    let symbol_size_dimensions = object.get("symbolSize").and_then(parse_symbol_dimensions);
    SeriesOptions {
        item_style,
        line_style: object
            .get("lineStyle")
            .map(parse_line_style)
            .unwrap_or_default(),
        area_style,
        label,
        smooth,
        smooth_monotone: object
            .get("smoothMonotone")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        connect_nulls: object
            .get("connectNulls")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        show_symbol: object
            .get("showSymbol")
            .and_then(Value::as_bool)
            .unwrap_or(true),
        show_all_symbol: object.get("showAllSymbol").and_then(Value::as_bool),
        symbol: object
            .get("symbol")
            .and_then(Value::as_str)
            .unwrap_or(if kind == "line" {
                "emptyCircle"
            } else {
                "circle"
            })
            .to_string(),
        symbol_size: object
            .get("symbolSize")
            .and_then(parse_f32)
            .or_else(|| symbol_size_dimensions.map(|size| size[0]))
            .unwrap_or(default_symbol_size)
            .max(0.0),
        symbol_size_dimensions: object
            .get("symbolSize")
            .is_some_and(Value::is_array)
            .then_some(symbol_size_dimensions)
            .flatten(),
        symbol_rotate: object
            .get("symbolRotate")
            .and_then(parse_f32)
            .unwrap_or(0.0),
        symbol_offset,
        step: object.get("step").and_then(|value| match value {
            Value::Bool(true) => Some(String::from("start")),
            Value::String(value) if matches!(value.as_str(), "start" | "middle" | "end") => {
                Some(value.clone())
            }
            _ => None,
        }),
        clip: object.get("clip").and_then(Value::as_bool).unwrap_or(true),
        sampling: object
            .get("sampling")
            .and_then(Value::as_str)
            .unwrap_or("none")
            .to_string(),
        end_label,
        area_origin: object
            .get("areaStyle")
            .and_then(Value::as_object)
            .and_then(|style| style.get("origin"))
            .cloned()
            .unwrap_or_else(|| Value::String(String::from("auto"))),
        bar_width: object.get("barWidth").and_then(parse_length),
        bar_max_width: object.get("barMaxWidth").and_then(parse_length),
        bar_min_width: object.get("barMinWidth").and_then(parse_length),
        bar_min_height: object
            .get("barMinHeight")
            .and_then(parse_f32)
            .unwrap_or(0.0)
            .max(0.0),
        bar_gap: object
            .get("barGap")
            .and_then(parse_percent_ratio)
            .unwrap_or(0.3),
        bar_category_gap: object
            .get("barCategoryGap")
            .and_then(parse_length)
            .unwrap_or(Length::Percent(20.0)),
        show_background: object
            .get("showBackground")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        background_style: object
            .get("backgroundStyle")
            .map(parse_item_style)
            .unwrap_or_else(|| ItemStyle {
                color: Some(0x33B4B4B4),
                ..ItemStyle::default()
            }),
        stack: object
            .get("stack")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        selected_mode: object.get("selectedMode").and_then(|value| match value {
            Value::String(value) => Some(value.clone()),
            Value::Bool(value) => Some(value.to_string()),
            _ => None,
        }),
        emphasis: object
            .get("emphasis")
            .map(parse_series_state)
            .unwrap_or_default(),
        blur: object
            .get("blur")
            .map(parse_series_state)
            .unwrap_or_default(),
        select: object
            .get("select")
            .map(parse_series_state)
            .unwrap_or_default(),
        label_layout: object
            .get("labelLayout")
            .map(parse_label_layout)
            .unwrap_or_default(),
        extra: extra.into_iter().collect(),
    }
}

fn parse_series_state(value: &Value) -> SeriesState {
    let Some(object) = value.as_object() else {
        return SeriesState::default();
    };
    SeriesState {
        item_style: object
            .get("itemStyle")
            .map(parse_item_style)
            .unwrap_or_default(),
        line_style: object
            .get("lineStyle")
            .map(parse_line_style)
            .unwrap_or_default(),
        label: object
            .get("label")
            .map(parse_label_style)
            .unwrap_or_default(),
        focus: object
            .get("focus")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        blur_scope: object
            .get("blurScope")
            .and_then(Value::as_str)
            .unwrap_or("coordinateSystem")
            .to_string(),
        scale: object.get("scale").and_then(|value| match value {
            Value::Bool(true) => Some(1.1),
            Value::Bool(false) => Some(1.0),
            _ => parse_f32(value),
        }),
    }
}

fn parse_label_layout(value: &Value) -> LabelLayoutOptions {
    let Some(object) = value.as_object() else {
        return LabelLayoutOptions::default();
    };
    LabelLayoutOptions {
        hide_overlap: object
            .get("hideOverlap")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        move_overlap: object
            .get("moveOverlap")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        draggable: object
            .get("draggable")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        x: object.get("x").cloned(),
        y: object.get("y").cloned(),
        dx: object.get("dx").and_then(parse_f32),
        dy: object.get("dy").and_then(parse_f32),
        rotate: object.get("rotate").and_then(parse_f32),
        align: object
            .get("align")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        vertical_align: object
            .get("verticalAlign")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        width: object.get("width").and_then(parse_f32),
        height: object.get("height").and_then(parse_f32),
        font_size: object
            .get("fontSize")
            .and_then(parse_f32)
            .map(|value| value.max(1.0)),
        label_line_points: object
            .get("labelLinePoints")
            .and_then(Value::as_array)
            .map(|points| {
                points
                    .iter()
                    .filter_map(|point| {
                        let point = point.as_array()?;
                        Some([
                            point.first().and_then(parse_f32)?,
                            point.get(1).and_then(parse_f32)?,
                        ])
                    })
                    .collect()
            })
            .unwrap_or_default(),
        callback: None,
        drag_offsets: Default::default(),
    }
}

pub(crate) fn parse_item_style(value: &Value) -> ItemStyle {
    let Some(object) = value.as_object() else {
        return ItemStyle::default();
    };
    ItemStyle {
        color: object.get("color").and_then(parse_color),
        color0: object.get("color0").and_then(parse_color),
        border_color: object.get("borderColor").and_then(parse_color),
        border_color0: object.get("borderColor0").and_then(parse_color),
        border_width: object
            .get("borderWidth")
            .and_then(parse_f32)
            .unwrap_or(0.0)
            .max(0.0),
        border_radius: object
            .get("borderRadius")
            .or_else(|| object.get("barBorderRadius"))
            .map(parse_border_radius)
            .unwrap_or([0.0; 4]),
        opacity: object
            .get("opacity")
            .and_then(Value::as_f64)
            .unwrap_or(1.0)
            .clamp(0.0, 1.0) as f32,
        specified: normalized_item_style_keys(object),
    }
}

fn parse_item_style_with_default(value: &Value, default: &ItemStyle) -> ItemStyle {
    let Some(object) = value.as_object() else {
        return default.clone();
    };
    ItemStyle {
        color: object.get("color").and_then(parse_color).or(default.color),
        color0: object
            .get("color0")
            .and_then(parse_color)
            .or(default.color0),
        border_color: object
            .get("borderColor")
            .and_then(parse_color)
            .or(default.border_color),
        border_color0: object
            .get("borderColor0")
            .and_then(parse_color)
            .or(default.border_color0),
        border_width: object
            .get("borderWidth")
            .and_then(parse_f32)
            .unwrap_or(default.border_width)
            .max(0.0),
        border_radius: object
            .get("borderRadius")
            .or_else(|| object.get("barBorderRadius"))
            .map(parse_border_radius)
            .unwrap_or(default.border_radius),
        opacity: object
            .get("opacity")
            .and_then(Value::as_f64)
            .unwrap_or(default.opacity as f64)
            .clamp(0.0, 1.0) as f32,
        specified: {
            let mut specified = default.specified.clone();
            specified.extend(normalized_item_style_keys(object));
            specified
        },
    }
}

fn normalized_item_style_keys(object: &serde_json::Map<String, Value>) -> BTreeSet<String> {
    let mut keys: BTreeSet<String> = object.keys().cloned().collect();
    if keys.contains("barBorderRadius") {
        keys.insert(String::from("borderRadius"));
    }
    keys
}

fn parse_border_radius(value: &Value) -> [f32; 4] {
    if let Some(radius) = parse_f32(value) {
        return [radius.max(0.0); 4];
    }
    let Some(values) = value.as_array() else {
        return [0.0; 4];
    };
    let values: Vec<f32> = values
        .iter()
        .filter_map(parse_f32)
        .map(|value| value.max(0.0))
        .collect();
    match values.as_slice() {
        [] => [0.0; 4],
        [all] => [*all; 4],
        [vertical, horizontal] => [*vertical, *horizontal, *vertical, *horizontal],
        [top_left, horizontal, bottom_right] => {
            [*top_left, *horizontal, *bottom_right, *horizontal]
        }
        [top_left, top_right, bottom_right, bottom_left, ..] => {
            [*top_left, *top_right, *bottom_right, *bottom_left]
        }
    }
}

pub(crate) fn parse_line_style(value: &Value) -> LineStyle {
    let Some(object) = value.as_object() else {
        return LineStyle::default();
    };
    LineStyle {
        color: object.get("color").and_then(parse_color),
        width: object
            .get("width")
            .and_then(parse_f32)
            .unwrap_or(2.0)
            .max(0.0),
        opacity: object
            .get("opacity")
            .and_then(Value::as_f64)
            .unwrap_or(1.0)
            .clamp(0.0, 1.0) as f32,
        kind: object
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or("solid")
            .to_string(),
        specified: object.keys().cloned().collect(),
    }
}

fn parse_line_style_with_default(value: &Value, default: &LineStyle) -> LineStyle {
    let Some(object) = value.as_object() else {
        return default.clone();
    };
    let mut specified = default.specified.clone();
    specified.extend(object.keys().cloned());
    LineStyle {
        color: object.get("color").and_then(parse_color).or(default.color),
        width: object
            .get("width")
            .and_then(parse_f32)
            .unwrap_or(default.width)
            .max(0.0),
        opacity: object
            .get("opacity")
            .and_then(Value::as_f64)
            .unwrap_or(default.opacity as f64)
            .clamp(0.0, 1.0) as f32,
        kind: object
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or(&default.kind)
            .to_string(),
        specified,
    }
}

pub(crate) fn parse_label_style(value: &Value) -> LabelStyle {
    let Some(object) = value.as_object() else {
        return LabelStyle::default();
    };
    LabelStyle {
        show: object.get("show").and_then(Value::as_bool).unwrap_or(false),
        color: object.get("color").and_then(parse_color),
        font_size: object
            .get("fontSize")
            .and_then(parse_f32)
            .unwrap_or(12.0)
            .max(1.0),
        font_weight: object
            .get("fontWeight")
            .and_then(|value| match value {
                Value::Number(value) => value.as_i64().map(|value| value as i32),
                Value::String(value) if value == "bold" || value == "bolder" => Some(700),
                Value::String(value) if value == "normal" || value == "lighter" => Some(400),
                _ => None,
            })
            .unwrap_or(400),
        position: object
            .get("position")
            .and_then(Value::as_str)
            .unwrap_or("top")
            .to_string(),
        distance: object.get("distance").and_then(parse_f32).unwrap_or(5.0),
        rotate: object.get("rotate").and_then(parse_f32).unwrap_or(0.0),
        offset: object
            .get("offset")
            .and_then(Value::as_array)
            .map(|values| {
                [
                    values.first().and_then(parse_f32).unwrap_or(0.0),
                    values.get(1).and_then(parse_f32).unwrap_or(0.0),
                ]
            })
            .unwrap_or([0.0, 0.0]),
        formatter: object
            .get("formatter")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        specified: object.keys().cloned().collect(),
    }
}

fn parse_end_label_style(value: &Value) -> LabelStyle {
    let mut label = parse_label_style(value);
    let object = value.as_object();
    if object.is_none_or(|object| !object.contains_key("position")) {
        label.position = String::from("right");
    }
    if object.is_none_or(|object| !object.contains_key("distance")) {
        label.distance = 8.0;
    }
    label
}

pub(crate) fn parse_color(value: &Value) -> Option<u32> {
    match value {
        Value::Number(number) => number.as_u64().map(|value| value as u32),
        Value::String(text) => parse_hex_color(text),
        _ => None,
    }
}

fn parse_hex_color(value: &str) -> Option<u32> {
    if let Some(value) = value.strip_prefix('#') {
        return match value.len() {
            3 => {
                let color = u16::from_str_radix(value, 16).ok()?;
                let r = ((color >> 8) & 0xF) as u32 * 17;
                let g = ((color >> 4) & 0xF) as u32 * 17;
                let b = (color & 0xF) as u32 * 17;
                Some(0xFF000000 | r << 16 | g << 8 | b)
            }
            4 => {
                let color = u16::from_str_radix(value, 16).ok()?;
                let r = ((color >> 12) & 0xF) as u32 * 17;
                let g = ((color >> 8) & 0xF) as u32 * 17;
                let b = ((color >> 4) & 0xF) as u32 * 17;
                let a = (color & 0xF) as u32 * 17;
                Some(a << 24 | r << 16 | g << 8 | b)
            }
            6 => u32::from_str_radix(value, 16)
                .ok()
                .map(|color| 0xFF000000 | color),
            8 => {
                let color = u32::from_str_radix(value, 16).ok()?;
                let rgb = color >> 8;
                let alpha = color & 0xFF;
                Some(alpha << 24 | rgb)
            }
            _ => None,
        };
    }

    if let Some(arguments) = value
        .strip_prefix("rgb(")
        .and_then(|value| value.strip_suffix(')'))
    {
        let channels: Vec<u32> = arguments
            .split(',')
            .filter_map(|value| value.trim().parse().ok())
            .collect();
        return (channels.len() == 3).then(|| {
            0xFF000000
                | channels[0].min(255) << 16
                | channels[1].min(255) << 8
                | channels[2].min(255)
        });
    }
    if let Some(arguments) = value
        .strip_prefix("rgba(")
        .and_then(|value| value.strip_suffix(')'))
    {
        let channels: Vec<&str> = arguments.split(',').map(str::trim).collect();
        if channels.len() == 4 {
            let r = channels[0].parse::<u32>().ok()?.min(255);
            let g = channels[1].parse::<u32>().ok()?.min(255);
            let b = channels[2].parse::<u32>().ok()?.min(255);
            let alpha = channels[3].parse::<f32>().ok()?.clamp(0.0, 1.0);
            return Some(((alpha * 255.0).round() as u32) << 24 | r << 16 | g << 8 | b);
        }
    }
    Some(match value.to_ascii_lowercase().as_str() {
        "transparent" => 0x00000000,
        "black" => 0xFF000000,
        "white" => 0xFFFFFFFF,
        "red" => 0xFFFF0000,
        "green" => 0xFF008000,
        "blue" => 0xFF0000FF,
        "yellow" => 0xFFFFFF00,
        "gray" | "grey" => 0xFF808080,
        _ => return None,
    })
}

fn parse_f32(value: &Value) -> Option<f32> {
    match value {
        Value::Number(number) => number.as_f64().map(|value| value as f32),
        Value::String(text) => text.trim_end_matches('%').parse().ok(),
        _ => None,
    }
}

fn parse_font_weight(value: &Value) -> Option<i32> {
    match value {
        Value::Number(value) => value.as_i64().map(|value| value as i32),
        Value::String(value) if value == "bold" || value == "bolder" => Some(700),
        Value::String(value) if value == "normal" || value == "lighter" => Some(400),
        _ => None,
    }
}

fn parse_length(value: &Value) -> Option<Length> {
    match value {
        Value::Number(number) => number.as_f64().map(|value| Length::Px(value as f32)),
        Value::String(text) if text.ends_with('%') => {
            text.trim_end_matches('%').parse().ok().map(Length::Percent)
        }
        Value::String(text) => text.parse().ok().map(Length::Px),
        _ => None,
    }
}

fn parse_symbol_dimensions(value: &Value) -> Option<[f32; 2]> {
    if let Some(size) = parse_f32(value) {
        let size = size.max(0.0);
        return Some([size, size]);
    }
    let values = value.as_array()?;
    let width = values.first().and_then(parse_f32)?.max(0.0);
    let height = values.get(1).and_then(parse_f32).unwrap_or(width).max(0.0);
    Some([width, height])
}

fn parse_percent_ratio(value: &Value) -> Option<f32> {
    match value {
        Value::Number(number) => number.as_f64().map(|value| value as f32),
        Value::String(text) if text.ends_with('%') => text
            .trim_end_matches('%')
            .parse::<f32>()
            .ok()
            .map(|value| value / 100.0),
        Value::String(text) => text.parse().ok(),
        _ => None,
    }
}

fn label_from_value(value: &Value) -> String {
    match value {
        Value::String(value) => value.clone(),
        Value::Number(value) => value.to_string(),
        Value::Bool(value) => value.to_string(),
        Value::Null => String::new(),
        Value::Array(_) | Value::Object(_) => value.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_echarts_like_json() {
        let option = parse_option_str(
            r##"{
                "title": {"text": "Sales"},
                "xAxis": {"type": "category", "data": ["Mon", "Tue"]},
                "yAxis": {"type": "value"},
                "color": ["#ff0000"],
                "series": [{"type": "bar", "name": "A", "data": [3, 5]}]
            }"##,
        )
        .unwrap();

        assert_eq!(option.title.unwrap().text, "Sales");
        assert_eq!(option.x_axis[0].data, vec!["Mon", "Tue"]);
        assert_eq!(option.visual_style.palette, vec![0xFFFF0000]);
        assert!(matches!(option.series[0], Series::Bar(_)));
    }

    #[test]
    fn parses_echarts_bar_layout_and_background_options() {
        let option = parse_option_str(
            r##"{
                "xAxis":{"type":"value"},
                "yAxis":{"type":"category","data":["A","B"]},
                "series":[{
                    "type":"bar","data":[2,8],
                    "barWidth":"40%","barMinWidth":3,"barMaxWidth":"50%",
                    "barMinHeight":5,"barGap":"10%","barCategoryGap":"35%",
                    "showBackground":true,
                    "backgroundStyle":{"color":"rgba(180,180,180,0.2)","borderRadius":4},
                    "itemStyle":{"barBorderRadius":[0,6,6,0]}
                }]
            }"##,
        )
        .unwrap();

        let Series::Bar(bar) = &option.series[0] else {
            panic!("expected bar");
        };
        assert_eq!(bar.options.bar_width, Some(Length::Percent(40.0)));
        assert_eq!(bar.options.bar_min_width, Some(Length::Px(3.0)));
        assert_eq!(bar.options.bar_max_width, Some(Length::Percent(50.0)));
        assert_eq!(bar.options.bar_min_height, 5.0);
        assert_eq!(bar.options.bar_gap, 0.1);
        assert_eq!(bar.options.bar_category_gap, Length::Percent(35.0));
        assert!(bar.options.show_background);
        assert_eq!(bar.options.background_style.border_radius, [4.0; 4]);
        assert_eq!(bar.options.item_style.border_radius, [0.0, 6.0, 6.0, 0.0]);
        assert!(bar.options.item_style.specified.contains("borderRadius"));
    }

    #[test]
    fn parses_scatter_symbol_and_visual_size_options() {
        let option = parse_option_str(
            r##"{
                "visualMap":{"show":false,"min":0,"max":100,"dimension":2,
                    "inRange":{"color":["#5470c6","#ee6666"],"symbolSize":[8,32]}},
                "series":[{
                    "type":"effectScatter","symbol":"roundRect","symbolSize":[22,12],
                    "symbolRotate":15,"symbolOffset":["25%",-2],
                    "rippleEffect":{"number":4,"scale":3,"brushType":"stroke"},
                    "data":[{"value":[2,4,75],"symbol":"diamond","symbolSize":[30,18]}]
                }]
            }"##,
        )
        .unwrap();

        let visual_map = option.visual_map.as_ref().expect("visualMap");
        assert_eq!(visual_map.dimension, Some(2));
        assert_eq!(visual_map.symbol_size_range, Some([8.0, 32.0]));
        let Series::EffectScatter(scatter) = &option.series[0] else {
            panic!("expected effectScatter");
        };
        assert_eq!(scatter.options.symbol, "roundRect");
        assert_eq!(scatter.options.symbol_size, 22.0);
        assert_eq!(scatter.options.symbol_size_dimensions, Some([22.0, 12.0]));
        assert_eq!(scatter.options.symbol_rotate, 15.0);
        assert_eq!(scatter.data[0].extra["symbol"], "diamond");
        assert_eq!(
            scatter.data[0].extra["symbolSize"],
            serde_json::json!([30, 18])
        );
        assert!(scatter.options.extra.contains_key("rippleEffect"));
    }

    #[test]
    fn preserves_pie_angle_label_line_and_selection_options() {
        let option = parse_option_str(
            r##"{
                "series":[{
                    "type":"pie","selectedMode":"multiple","selectedOffset":14,
                    "startAngle":90,"endAngle":-210,"minAngle":8,"padAngle":2,
                    "minShowLabelAngle":4,"stillShowZeroSum":false,"percentPrecision":1,
                    "label":{"show":true,"formatter":"{b} {d}%"},
                    "labelLine":{"length":12,"length2":18,"lineStyle":{"type":"dashed"}},
                    "data":[{"name":"A","value":3,"selected":true},{"name":"B","value":7}]
                }]
            }"##,
        )
        .unwrap();
        let Series::Pie(pie) = &option.series[0] else {
            panic!("expected pie");
        };
        assert_eq!(pie.options.selected_mode.as_deref(), Some("multiple"));
        assert_eq!(pie.options.extra["selectedOffset"], 14);
        assert_eq!(pie.options.extra["endAngle"], -210);
        assert_eq!(pie.options.extra["labelLine"]["length2"], 18);
        assert_eq!(pie.data[0].extra["selected"], true);
        assert_eq!(pie.options.label.formatter.as_deref(), Some("{b} {d}%"));
    }

    #[test]
    fn custom_json_reports_diagnostic() {
        let option = parse_option_str(r#"{"series":[{"type":"custom","data":[1]}]}"#).unwrap();
        assert_eq!(option.diagnostics.len(), 1);
    }

    #[test]
    fn parses_geojson_map_features() {
        let option = parse_option_str(
            r#"{"series":[{"type":"map","geoJson":{"type":"FeatureCollection","features":[{"type":"Feature","properties":{"name":"A","value":2},"geometry":{"type":"Polygon","coordinates":[[[0,0],[2,0],[2,1],[0,1],[0,0]]]}}]}}]}"#,
        )
        .unwrap();

        let Series::Map(map) = &option.series[0] else {
            panic!("expected map");
        };
        assert_eq!(map.features[0].name, "A");
        assert_eq!(map.features[0].polygons[0].exterior.len(), 5);
    }

    #[test]
    fn parses_line_geometry_and_label_options() {
        let option = parse_option_str(
            r##"{
                "series":[{
                    "type":"line","step":"middle","smooth":true,"smoothMonotone":"x",
                    "connectNulls":true,"clip":false,"sampling":"lttb",
                    "showAllSymbol":true,"symbol":"diamond","symbolSize":12,
                    "symbolRotate":30,"symbolOffset":["50%",-2],
                    "lineStyle":{"type":"dashed","width":3},
                    "areaStyle":{"origin":"end"},
                    "label":{"show":true,"position":"left","distance":9,"rotate":15,"offset":[2,3]},
                    "endLabel":{"show":true,"formatter":"{c}"},
                    "data":[1,2]
                }]
            }"##,
        )
        .unwrap();
        let Series::Line(line) = &option.series[0] else {
            panic!("expected line");
        };
        assert_eq!(line.options.step.as_deref(), Some("middle"));
        assert_eq!(line.options.smooth_monotone.as_deref(), Some("x"));
        assert!(line.options.connect_nulls);
        assert!(!line.options.clip);
        assert_eq!(line.options.sampling, "lttb");
        assert_eq!(line.options.show_all_symbol, Some(true));
        assert_eq!(line.options.symbol, "diamond");
        assert_eq!(line.options.symbol_size, 12.0);
        assert_eq!(line.options.line_style.kind, "dashed");
        assert_eq!(line.options.label.offset, [2.0, 3.0]);
        assert_eq!(line.options.end_label.position, "right");
        assert_eq!(line.options.area_origin, Value::String(String::from("end")));
    }

    #[test]
    fn map_preserves_holes_no_data_and_region_overrides() {
        let option = parse_option_str(
            r##"{
                "series":[{
                    "type":"map","nameProperty":"code","nameMap":{"A":"Alpha"},
                    "selectedMode":"multiple","layoutCenter":["45%","55%"],"layoutSize":"80%",
                    "geoJson":{"type":"FeatureCollection","features":[
                        {"type":"Feature","properties":{"name":"Ignored","code":"A","cp":[5,5]},
                         "geometry":{"type":"Polygon","coordinates":[
                            [[0,0],[10,0],[10,10],[0,10],[0,0]],
                            [[3,3],[7,3],[7,7],[3,7],[3,3]]
                         ]}},
                        {"type":"Feature","properties":{"code":"B"},
                         "geometry":{"type":"Polygon","coordinates":[[[12,0],[18,0],[18,6],[12,0]]]}}
                    ]},
                    "regions":[{"name":"A","itemStyle":{"borderColor":"#ff0000","borderWidth":2}}],
                    "data":[{"name":"A","value":7,"selected":true}]
                }]
            }"##,
        )
        .unwrap();
        let Series::Map(map) = &option.series[0] else {
            panic!("expected map");
        };
        assert_eq!(map.features[0].name, "Alpha");
        assert_eq!(map.features[0].value, Some(7.0));
        assert!(map.features[0].selected);
        assert_eq!(map.features[0].center, Some((5.0, 5.0)));
        assert_eq!(map.features[0].polygons[0].holes.len(), 1);
        assert_eq!(map.features[0].item_style.border_color, Some(0xFFFF0000));
        assert_eq!(map.features[1].value, None);
        assert_eq!(map.options.selected_mode.as_deref(), Some("multiple"));
        assert_eq!(
            map.map_options.layout_size,
            Some(Value::String(String::from("80%")))
        );
    }

    #[test]
    fn dataset_populates_categories_and_multiple_series() {
        let option = parse_option_str(
            r#"{
                "dataset":{"source":[["day","orders","revenue"],["Mon",12,8],["Tue",20,18]]},
                "xAxis":{"type":"category"},"yAxis":{},
                "series":[{"type":"bar"},{"type":"line"}]
            }"#,
        )
        .unwrap();
        assert_eq!(option.x_axis[0].data, ["Mon", "Tue"]);
        assert_eq!(
            super::basic_series_mut(&mut option.series[0].clone())
                .unwrap()
                .data[1]
                .number(0),
            20.0
        );
        assert_eq!(
            super::basic_series_mut(&mut option.series[1].clone())
                .unwrap()
                .data[1]
                .number(0),
            18.0
        );
    }

    #[test]
    fn graph_links_accept_echarts_node_names() {
        let option = parse_option_str(
            r#"{"series":[{"type":"graph","data":[{"name":"A"},{"name":"B"}],"links":[{"source":"A","target":"B"}]}]}"#,
        )
        .unwrap();
        let Series::Graph(graph) = &option.series[0] else {
            panic!("expected graph");
        };
        assert_eq!((graph.links[0].source, graph.links[0].target), (0, 1));
    }

    #[test]
    fn parses_data_zoom_axis_pointer_and_markers() {
        let option = parse_option_str(
            r#"{
                "tooltip":{"trigger":"axis","axisPointer":{"type":"cross","snap":true}},
                "dataZoom":[{"type":"slider","start":20,"end":80,"xAxisIndex":0}],
                "series":[{"type":"line","data":[1,2],"markPoint":{"data":[{"type":"max"}]}}]
            }"#,
        )
        .unwrap();
        assert_eq!(option.data_zoom.len(), 1);
        assert_eq!(
            (option.data_zoom[0].start, option.data_zoom[0].end),
            (20.0, 80.0)
        );
        assert_eq!(option.tooltip.axis_pointer.kind, "cross");
        assert!(option.tooltip.axis_pointer.snap);
        let Series::Line(series) = &option.series[0] else {
            panic!("expected line");
        };
        assert!(series.options.extra.contains_key("markPoint"));
    }

    #[test]
    fn parses_echarts_cartesian_axis_appearance() {
        let option = parse_option_str(
            r##"{
                "xAxis": {
                    "type": "category",
                    "position": "top",
                    "offset": 6,
                    "axisLine": {"show": true, "onZero": false, "lineStyle": {"color": "#123456", "width": 3}},
                    "axisTick": {"alignWithLabel": true, "inside": true, "length": 9},
                    "axisLabel": {"color": "#654321", "fontSize": 14, "fontWeight": "bold", "rotate": 30, "margin": 12, "interval": 0, "formatter": "{value} kg"},
                    "splitLine": {"show": true, "lineStyle": {"color": "#abcdef", "width": 2}}
                },
                "yAxis": {"position": "right"},
                "series": [{"type": "bar", "data": [1]}]
            }"##,
        )
        .unwrap();

        let x = &option.x_axis[0];
        assert_eq!((x.position.as_str(), x.offset), ("top", 6.0));
        assert!(!x.axis_line.on_zero);
        assert_eq!(x.axis_line.line_style.color, Some(0xFF123456));
        assert_eq!(x.axis_line.line_style.width, 3.0);
        assert!(x.axis_tick.align_with_label);
        assert!(x.axis_tick.inside);
        assert_eq!(x.axis_tick.length, 9.0);
        assert_eq!(x.axis_label_style.color, Some(0xFF654321));
        assert_eq!(x.axis_label_style.font_size, 14.0);
        assert_eq!(x.axis_label_style.font_weight, 700);
        assert_eq!(x.axis_label_style.rotate, 30.0);
        assert_eq!(x.axis_label_style.interval, Some(0));
        assert_eq!(x.axis_label_style.formatter.as_deref(), Some("{value} kg"));
        assert!(x.split_line);
        assert_eq!(x.split_line_style.color, Some(0xFFABCDEF));
        assert_eq!(option.y_axis[0].position, "right");
    }

    #[test]
    fn preserves_echarts_missing_values_and_connect_nulls() {
        let option = parse_option_str(
            r#"{
                "xAxis":{"type":"category","data":["A","B","C","D"]},
                "series":[
                    {"type":"line","data":[12,null,"-",18]},
                    {"type":"line","connectNulls":true,"data":[8,null,14,16]}
                ]
            }"#,
        )
        .unwrap();

        let Series::Line(gapped) = &option.series[0] else {
            panic!("expected line");
        };
        assert_eq!(gapped.data[0].number_opt(0), Some(12.0));
        assert!(matches!(gapped.data[1].values[0], DataValue::Null));
        assert!(matches!(gapped.data[2].values[0], DataValue::Null));
        assert_eq!(gapped.data[3].number_opt(0), Some(18.0));
        assert!(!gapped.options.connect_nulls);

        let Series::Line(connected) = &option.series[1] else {
            panic!("expected line");
        };
        assert!(connected.options.connect_nulls);
    }

    #[test]
    fn preserves_hierarchical_treemap_data_and_polyline_lines() {
        let option = parse_option_str(
            r#"{"series":[
                {"type":"treemap","data":[{"name":"root","children":[{"name":"leaf","value":2}]}]},
                {"type":"lines","symbol":["none","arrow"],"data":[{"coords":[[0,0],[1,2],[3,1]]}]}
            ]}"#,
        )
        .unwrap();
        let Series::Treemap(treemap) = &option.series[0] else {
            panic!("expected treemap");
        };
        assert!(treemap.data[0].extra.contains_key("children"));
        let Series::Lines(lines) = &option.series[1] else {
            panic!("expected lines");
        };
        assert_eq!(lines.data[0].coords, [(0.0, 0.0), (1.0, 2.0), (3.0, 1.0)]);
        assert!(lines.options.extra.contains_key("symbol"));
    }

    #[test]
    fn graph_nodes_keep_item_symbol_and_label_options() {
        let option = parse_option_str(
            r##"{"series":[{"type":"graph","data":[{
                "name":"A","symbol":"diamond","symbolSize":18,"symbolRotate":20,
                "itemStyle":{"color":"#ee6666"},"label":{"show":true,"color":"#334155"}
            }]}]}"##,
        )
        .unwrap();
        let Series::Graph(graph) = &option.series[0] else {
            panic!("expected graph");
        };
        let node = &graph.nodes[0];
        assert_eq!(node.symbol.as_deref(), Some("diamond"));
        assert_eq!(node.symbol_size, Some(18.0));
        assert_eq!(node.symbol_rotate, 20.0);
        assert_eq!(node.item_style.color, Some(0xFFEE6666));
        assert!(node.label.show);
    }

    #[test]
    fn parses_piecewise_visual_map_rules() {
        let option = parse_option_str(
            r##"{"visualMap":{"pieces":[
                {"max":5,"color":"#dbeafe"},
                {"min":6,"max":15,"color":"#60a5fa","symbolSize":18},
                {"value":20,"color":"#1d4ed8"}
            ]}}"##,
        )
        .unwrap();
        let visual_map = option.visual_map.unwrap();
        assert_eq!(visual_map.pieces.len(), 3);
        assert!(visual_map.pieces[1].contains(12.0));
        assert_eq!(visual_map.pieces[1].symbol_size, Some(18.0));
        assert!(visual_map.pieces[2].contains(20.0));
    }

    #[test]
    fn multiple_datasets_respect_series_dataset_index() {
        let option = parse_option_str(
            r#"{
                "dataset":[
                    {"source":[["day","value"],["Mon",1],["Tue",2]]},
                    {"source":[["day","value"],["Mon",10],["Tue",20]]}
                ],
                "series":[
                    {"type":"line","datasetIndex":0,"encode":{"y":"value"}},
                    {"type":"line","datasetIndex":1,"encode":{"y":"value"}}
                ]
            }"#,
        )
        .unwrap();
        assert_eq!(option.datasets.len(), 2);
        let Series::Line(first) = &option.series[0] else {
            panic!("line")
        };
        let Series::Line(second) = &option.series[1] else {
            panic!("line")
        };
        assert_eq!(first.data[1].number_opt(0), Some(2.0));
        assert_eq!(second.data[1].number_opt(0), Some(20.0));
    }

    #[test]
    fn dataset_filter_and_sort_pipeline_feeds_axis_and_series() {
        let option = parse_option_str(
            r#"{
                "dataset":[
                    {
                        "id":"raw",
                        "source":[
                            ["Product","Sales","Year"],
                            ["Cake",120,2024],
                            ["Tea",260,2025],
                            ["Tofu",180,2025],
                            ["Milk",310,2025]
                        ]
                    },
                    {
                        "fromDatasetId":"raw",
                        "transform":[
                            {"type":"filter","config":{"dimension":"Year","=":2025}},
                            {"type":"sort","config":{"dimension":"Sales","order":"desc"}}
                        ]
                    }
                ],
                "xAxis":{"type":"category"},
                "yAxis":{"type":"value"},
                "series":[{
                    "type":"bar","datasetIndex":1,
                    "encode":{"x":"Product","y":"Sales"}
                }]
            }"#,
        )
        .unwrap();
        assert_eq!(option.datasets.len(), 2);
        let transformed = &option.datasets[1];
        assert!(transformed.source_header);
        assert_eq!(transformed.dimensions, ["Product", "Sales", "Year"]);
        assert_eq!(data_value_label(&transformed.source[1][0]), "Milk");
        assert_eq!(data_value_label(&transformed.source[2][0]), "Tea");
        assert_eq!(data_value_label(&transformed.source[3][0]), "Tofu");
        assert_eq!(option.x_axis[0].data, ["Milk", "Tea", "Tofu"]);
        let Series::Bar(series) = &option.series[0] else {
            panic!("bar")
        };
        assert_eq!(series.data[0].number_opt(0), Some(310.0));
        assert_eq!(series.data[2].number_opt(0), Some(180.0));
    }

    #[test]
    fn dataset_filter_supports_nested_conditions_and_object_rows() {
        let option = parse_option_str(
            r#"{
                "dataset":[
                    {
                        "dimensions":["name","score","group"],
                        "source":[
                            {"name":" A ","score":"92pts","group":"core"},
                            {"name":"B","score":"75pts","group":"core"},
                            {"name":"C","score":"88pts","group":"other"}
                        ]
                    },
                    {
                        "transform":{"type":"filter","config":{"and":[
                            {"dimension":"score","parser":"number",">=":80},
                            {"not":{"dimension":"group","value":"other"}}
                        ]}}
                    }
                ],
                "series":[{"type":"pie","datasetIndex":1,"encode":{"itemName":"name","value":"score"}}]
            }"#,
        )
        .unwrap();
        assert!(!option.datasets[0].source_header);
        assert_eq!(option.datasets[1].source.len(), 1);
        let Series::Pie(series) = &option.series[0] else {
            panic!("pie")
        };
        assert_eq!(series.data.len(), 1);
        assert_eq!(series.data[0].name.as_deref(), Some(" A "));
    }

    #[test]
    fn multiple_visual_maps_resolve_by_series_index() {
        let option = parse_option_str(
            r##"{
                "visualMap":[
                    {"seriesIndex":0,"min":0,"max":10,"inRange":{"color":["#000000","#111111"]}},
                    {"seriesIndex":[1,2],"min":0,"max":100,"inRange":{"color":["#eeeeee","#ffffff"]}}
                ]
            }"##,
        )
        .unwrap();
        assert_eq!(option.visual_maps.len(), 2);
        assert_eq!(option.visual_map_for_series(0).unwrap().max, 10.0);
        assert_eq!(option.visual_map_for_series(2).unwrap().max, 100.0);
    }

    #[test]
    fn parses_legend_selection_layout_and_object_entries() {
        let option = parse_option_str(
            r##"{
                "legend":{
                    "selectedMode":"single","itemGap":14,"icon":"diamond",
                    "inactiveColor":"#94a3b8","formatter":"Series: {name}",
                    "selected":{"Orders":false},
                    "data":["Revenue",{"name":"Orders","icon":"circle"}],
                    "selector":true
                },
                "series":[
                    {"type":"line","name":"Revenue","data":[1]},
                    {"type":"bar","name":"Orders","data":[2]}
                ]
            }"##,
        )
        .unwrap();
        let legend = option.legend.unwrap();
        assert_eq!(legend.selected_mode, "single");
        assert_eq!(legend.item_gap, 14.0);
        assert_eq!(legend.icon, "diamond");
        assert_eq!(legend.inactive_color, 0xFF94A3B8);
        assert_eq!(legend.formatter.as_deref(), Some("Series: {name}"));
        assert_eq!(legend.data, ["Revenue", "Orders"]);
        assert_eq!(
            legend.data_icons.get("Orders").map(String::as_str),
            Some("circle")
        );
        assert_eq!(legend.selected.get("Orders"), Some(&false));
        assert_eq!(legend.extra.get("selector"), Some(&Value::Bool(true)));
    }

    #[test]
    fn parses_global_animation_common_states_and_label_layout() {
        let option = parse_option_str(
            r##"{
                "animation":true,
                "animationThreshold":800,
                "animationDuration":900,
                "animationEasing":"cubicOut",
                "animationDelay":40,
                "animationDurationUpdate":450,
                "animationEasingUpdate":"quadraticOut",
                "stateAnimation":{"duration":260,"easing":"cubicInOut","delay":20},
                "series":[{
                    "type":"bar","selectedMode":"multiple","data":[1,2],
                    "emphasis":{"focus":"series","scale":1.2,"itemStyle":{"color":"#ff0000"}},
                    "blur":{"blurScope":"coordinateSystem","itemStyle":{"opacity":0.25}},
                    "select":{"label":{"show":true,"color":"#ffffff"}},
                    "labelLayout":{
                        "hideOverlap":true,"moveOverlap":"shiftY","draggable":true,
                        "x":"50%","y":24,"dx":3,"dy":4,"rotate":15,
                        "align":"center","verticalAlign":"middle",
                        "width":80,"height":20,"fontSize":14,
                        "labelLinePoints":[[1,2],[3,4],[5,6]]
                    }
                }]
            }"##,
        )
        .unwrap();
        assert_eq!(option.animation.threshold, 800);
        assert_eq!(option.animation.initial.duration, 900);
        assert_eq!(option.animation.initial.easing, "cubicOut");
        assert_eq!(option.animation.initial.delay, 40);
        assert_eq!(option.animation.update.duration, 450);
        assert_eq!(option.animation.update.easing, "quadraticOut");
        assert_eq!(option.animation.state.duration, 260);
        assert_eq!(option.animation.state.delay, 20);
        let Series::Bar(series) = &option.series[0] else {
            panic!("bar")
        };
        assert_eq!(series.options.emphasis.focus.as_deref(), Some("series"));
        assert_eq!(series.options.emphasis.scale, Some(1.2));
        assert_eq!(series.options.emphasis.item_style.color, Some(0xFFFF0000));
        assert_eq!(series.options.blur.item_style.opacity, 0.25);
        assert!(series.options.select.label.show);
        assert!(series.options.label_layout.hide_overlap);
        assert_eq!(
            series.options.label_layout.move_overlap.as_deref(),
            Some("shiftY")
        );
        assert!(series.options.label_layout.draggable);
        assert_eq!(series.options.label_layout.x, Some(Value::from("50%")));
        assert_eq!(series.options.label_layout.y, Some(Value::from(24)));
        assert_eq!(series.options.label_layout.dx, Some(3.0));
        assert_eq!(series.options.label_layout.dy, Some(4.0));
        assert_eq!(series.options.label_layout.rotate, Some(15.0));
        assert_eq!(series.options.label_layout.align.as_deref(), Some("center"));
        assert_eq!(
            series.options.label_layout.vertical_align.as_deref(),
            Some("middle")
        );
        assert_eq!(series.options.label_layout.width, Some(80.0));
        assert_eq!(series.options.label_layout.height, Some(20.0));
        assert_eq!(series.options.label_layout.font_size, Some(14.0));
        assert_eq!(
            series.options.label_layout.label_line_points,
            [[1.0, 2.0], [3.0, 4.0], [5.0, 6.0]]
        );
    }

    #[test]
    fn timeline_merges_base_option_and_selects_current_frame() {
        let option = parse_option_str(
            r#"{
                "baseOption":{
                    "timeline":{"currentIndex":1,"data":["2024","2025"]},
                    "title":{"text":"Revenue"},
                    "xAxis":{"type":"category","data":["A","B"]},
                    "series":[{"type":"bar","data":[1,2]}]
                },
                "options":[
                    {"series":[{"type":"bar","data":[3,4]}]},
                    {"series":[{"type":"bar","data":[8,9]}]}
                ]
            }"#,
        )
        .unwrap();
        assert_eq!(option.timeline_options.len(), 2);
        assert_eq!(option.timeline.as_ref().unwrap().current_index, 1);
        assert_eq!(option.title.as_ref().unwrap().text, "Revenue");
        let Series::Bar(series) = &option.series[0] else {
            panic!("bar")
        };
        assert_eq!(series.data[0].number_opt(0), Some(8.0));
    }

    #[test]
    fn parses_native_brush_and_toolbox_brush_defaults() {
        let direct = parse_option_str(
            r##"{"brush":{"brushType":"lineX","brushMode":"multiple","brushStyle":{"color":"#123456"},"outOfBrush":{"opacity":0.2}}}"##,
        )
        .unwrap();
        let brush = direct.brush.unwrap();
        assert!(brush.active);
        assert_eq!(brush.brush_type, "lineX");
        assert_eq!(brush.brush_mode, "multiple");
        assert_eq!(brush.brush_style.color, Some(0xFF123456));
        assert_eq!(brush.out_of_brush_opacity, 0.2);

        let toolbox =
            parse_option_str(r#"{"toolbox":{"feature":{"brush":{"type":["rect","clear"]}}}}"#)
                .unwrap();
        assert!(!toolbox.brush.unwrap().active);
    }

    #[test]
    fn toolbox_data_zoom_creates_internal_axis_windows() {
        let option = parse_option_str(
            r#"{
                "toolbox":{"feature":{"dataZoom":{"xAxisIndex":[0],"yAxisIndex":false}}},
                "xAxis":[{"type":"category","data":["A","B"]},{"type":"value"}],
                "yAxis":{"type":"value"},
                "series":[{"type":"line","data":[1,2]}]
            }"#,
        )
        .unwrap();
        assert_eq!(option.data_zoom.len(), 1);
        let data_zoom = &option.data_zoom[0];
        assert_eq!(data_zoom.kind, "select");
        assert_eq!(data_zoom.x_axis_index, [0]);
        assert!(data_zoom.y_axis_index.is_empty());
        assert!(!data_zoom.show);
        assert_eq!(
            data_zoom.extra.get("toolboxInternal"),
            Some(&Value::Bool(true))
        );
    }

    #[test]
    fn media_rules_merge_in_order_without_becoming_timeline() {
        let option = parse_option_str(
            r#"{
                "baseOption":{
                    "title":{"text":"Base"},
                    "xAxis":{"type":"category","data":["A","B"]},
                    "series":[{"type":"line","name":"Revenue","data":[12,18]}]
                },
                "media":[
                    {"query":{"maxWidth":400},"option":{"title":{"text":"Compact"}}},
                    {"query":{"maxWidth":400},"option":{"series":[{"type":"bar"}]}},
                    {"option":{"title":{"text":"Default"}}}
                ]
            }"#,
        )
        .unwrap();
        assert!(option.timeline.is_none());
        assert!(option.media.is_some());
        let compact = resolve_media_option(&option, 360.0, 240.0, 0).unwrap();
        assert_eq!(compact.title.as_ref().unwrap().text, "Compact");
        let Series::Bar(series) = &compact.series[0] else {
            panic!("second matching media rule switches to bar")
        };
        assert_eq!(series.data.len(), 2);
        let default = resolve_media_option(&option, 720.0, 240.0, 0).unwrap();
        assert_eq!(default.title.as_ref().unwrap().text, "Default");
        assert!(matches!(default.series[0], Series::Line(_)));
    }

    #[test]
    fn media_and_timeline_resolve_the_current_frame_together() {
        let option = parse_option_str(
            r#"{
                "baseOption":{
                    "timeline":{"currentIndex":1,"data":["A","B"]},
                    "xAxis":{"type":"category","data":["X"]},
                    "series":[{"type":"line","data":[1]}]
                },
                "options":[
                    {"series":[{"data":[2]}]},
                    {"series":[{"data":[9]}]}
                ],
                "media":[
                    {"query":{"maxAspectRatio":2},"option":{"series":[{"type":"bar"}]}}
                ]
            }"#,
        )
        .unwrap();
        let resolved = resolve_media_option(&option, 360.0, 240.0, 1).unwrap();
        let Series::Bar(series) = &resolved.series[0] else {
            panic!("media switches current timeline frame to bar")
        };
        assert_eq!(series.data[0].number_opt(0), Some(9.0));
        assert_eq!(resolved.timeline.as_ref().unwrap().current_index, 1);
    }
}
