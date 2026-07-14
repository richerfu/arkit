//! Shared normal/emphasis/blur/select state resolution.

use std::collections::BTreeSet;

use crate::model::*;

#[derive(Clone, Copy, PartialEq, Eq)]
enum ActiveState {
    Normal,
    Emphasis,
    Blur,
    Select,
}

struct FocusPlan {
    series_index: usize,
    focus: String,
    blur_scope: String,
    coordinate_key: String,
    focused_items: BTreeSet<usize>,
}

pub(crate) fn apply_states(
    option: &mut ChartOption,
    hovered: Option<&ChartEvent>,
    selected_items: &BTreeSet<(usize, usize)>,
) {
    let hovered_item = hovered
        .filter(|event| event.series_index < option.series.len())
        .map(|event| (event.series_index, event.data_index));
    let focus = hovered_item.and_then(|(series_index, data_index)| {
        let series = &option.series[series_index];
        let state = &series_options(series)?.emphasis;
        let focus = state.focus.as_deref().unwrap_or("none");
        (!matches!(focus, "" | "none")).then(|| FocusPlan {
            series_index,
            focus: focus.to_owned(),
            blur_scope: state.blur_scope.clone(),
            coordinate_key: coordinate_key(series_index, series),
            focused_items: focused_items(series, data_index, focus),
        })
    });
    for (series_index, series) in option.series.iter_mut().enumerate() {
        let series_state = focus.as_ref().map_or(ActiveState::Normal, |focus| {
            let same_series = focus.series_index == series_index;
            let same_coordinate = focus.coordinate_key == coordinate_key(series_index, series);
            let outside_scope = match focus.blur_scope.as_str() {
                "series" => !same_series,
                "global" => false,
                _ => !same_coordinate,
            };
            if outside_scope || (focus.focus == "series" && same_series) {
                ActiveState::Normal
            } else {
                ActiveState::Blur
            }
        });
        let focused_items = focus
            .as_ref()
            .filter(|focus| focus.series_index == series_index)
            .map(|focus| &focus.focused_items);
        apply_series_state(
            series,
            series_index,
            series_state,
            hovered_item,
            selected_items,
            focused_items,
        );
    }
}

fn apply_series_state(
    series: &mut Series,
    series_index: usize,
    series_state: ActiveState,
    hovered: Option<(usize, usize)>,
    selected_items: &BTreeSet<(usize, usize)>,
    focused_items: Option<&BTreeSet<usize>>,
) {
    match series {
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
        | Series::Treemap(series) => {
            let options = series.options.clone();
            if series_state != ActiveState::Normal && focused_items.is_none() {
                merge_options_state(&mut series.options, &options, series_state);
            }
            for (data_index, point) in series.data.iter_mut().enumerate() {
                let state = item_state(
                    series_index,
                    data_index,
                    series_state,
                    hovered,
                    selected_items,
                    focused_items,
                );
                if state == ActiveState::Normal {
                    continue;
                }
                let state_options = state_options(&options, state);
                point.item_style =
                    merge_state_item_style(&point.item_style, &state_options.item_style, state);
                point.label = merge_label(&point.label, &state_options.label);
                apply_data_item_state(point, state);
            }
        }
        Series::Tree(series) | Series::Graph(series) => {
            let options = series.options.clone();
            if series_state != ActiveState::Normal && focused_items.is_none() {
                merge_options_state(&mut series.options, &options, series_state);
            }
            for (data_index, node) in series.nodes.iter_mut().enumerate() {
                let state = item_state(
                    series_index,
                    data_index,
                    series_state,
                    hovered,
                    selected_items,
                    focused_items,
                );
                if state != ActiveState::Normal {
                    let state_options = state_options(&options, state);
                    node.item_style =
                        merge_state_item_style(&node.item_style, &state_options.item_style, state);
                    node.label = merge_label(&node.label, &state_options.label);
                    if let Some(scale) = state_options.scale {
                        let base = node.symbol_size.unwrap_or(options.symbol_size);
                        node.symbol_size = Some(base * scale);
                    }
                }
            }
        }
        Series::Sankey(series) => {
            let options = series.options.clone();
            if series_state != ActiveState::Normal && focused_items.is_none() {
                merge_options_state(&mut series.options, &options, series_state);
            }
            for (data_index, node) in series.nodes.iter_mut().enumerate() {
                let state = item_state(
                    series_index,
                    data_index,
                    series_state,
                    hovered,
                    selected_items,
                    focused_items,
                );
                if state != ActiveState::Normal {
                    let state_options = state_options(&options, state);
                    node.item_style =
                        merge_state_item_style(&node.item_style, &state_options.item_style, state);
                    node.label = merge_label(&node.label, &state_options.label);
                }
            }
        }
        Series::Lines(series) => {
            let options = series.options.clone();
            let state = hovered
                .filter(|(hovered, _)| *hovered == series_index)
                .map_or(series_state, |_| ActiveState::Emphasis);
            if state != ActiveState::Normal {
                merge_options_state(&mut series.options, &options, state);
            }
        }
        Series::Sunburst(series) => {
            let options = series.options.clone();
            if series_state != ActiveState::Normal && focused_items.is_none() {
                merge_options_state(&mut series.options, &options, series_state);
            }
            let mut data_index = 0;
            apply_sunburst_states(
                &mut series.data,
                &options,
                series_index,
                &mut data_index,
                series_state,
                hovered,
                selected_items,
                focused_items,
            );
        }
        Series::Map(series) => {
            let options = series.options.clone();
            for (data_index, feature) in series.features.iter_mut().enumerate() {
                let state = item_state(
                    series_index,
                    data_index,
                    series_state,
                    hovered,
                    selected_items,
                    focused_items,
                );
                if state == ActiveState::Normal {
                    continue;
                }
                let state_options = state_options(&options, state);
                feature.item_style =
                    merge_state_item_style(&feature.item_style, &state_options.item_style, state);
                feature.label = merge_label(&feature.label, &state_options.label);
            }
        }
        Series::Custom(_) => {}
    }
}

#[allow(clippy::too_many_arguments)]
fn apply_sunburst_states(
    nodes: &mut [SunburstNode],
    options: &SeriesOptions,
    series_index: usize,
    data_index: &mut usize,
    series_state: ActiveState,
    hovered: Option<(usize, usize)>,
    selected_items: &BTreeSet<(usize, usize)>,
    focused_items: Option<&BTreeSet<usize>>,
) {
    for node in nodes {
        let index = *data_index;
        *data_index += 1;
        let state = item_state(
            series_index,
            index,
            series_state,
            hovered,
            selected_items,
            focused_items,
        );
        if state != ActiveState::Normal {
            node.item_style = merge_state_item_style(
                &node.item_style,
                &state_options(options, state).item_style,
                state,
            );
        }
        apply_sunburst_states(
            &mut node.children,
            options,
            series_index,
            data_index,
            series_state,
            hovered,
            selected_items,
            focused_items,
        );
    }
}

fn item_state(
    series_index: usize,
    data_index: usize,
    series_state: ActiveState,
    hovered: Option<(usize, usize)>,
    selected_items: &BTreeSet<(usize, usize)>,
    focused_items: Option<&BTreeSet<usize>>,
) -> ActiveState {
    if selected_items.contains(&(series_index, data_index)) {
        ActiveState::Select
    } else if hovered == Some((series_index, data_index)) {
        ActiveState::Emphasis
    } else if series_state == ActiveState::Blur {
        if focused_items.is_some_and(|items| items.contains(&data_index)) {
            ActiveState::Normal
        } else {
            ActiveState::Blur
        }
    } else {
        ActiveState::Normal
    }
}

fn state_options(options: &SeriesOptions, state: ActiveState) -> &SeriesState {
    match state {
        ActiveState::Emphasis => &options.emphasis,
        ActiveState::Blur => &options.blur,
        ActiveState::Select => &options.select,
        ActiveState::Normal => &options.emphasis,
    }
}

fn merge_options_state(options: &mut SeriesOptions, normal: &SeriesOptions, state: ActiveState) {
    let state_options = state_options(normal, state);
    options.item_style =
        merge_state_item_style(&options.item_style, &state_options.item_style, state);
    options.line_style =
        merge_state_line_style(&options.line_style, &state_options.line_style, state);
    options.label = merge_label(&options.label, &state_options.label);
    if let Some(scale) = state_options.scale {
        options.symbol_size *= scale;
        if let Some(size) = options.symbol_size_dimensions.as_mut() {
            size[0] *= scale;
            size[1] *= scale;
        }
    }
}

fn merge_state_item_style(base: &ItemStyle, state: &ItemStyle, active: ActiveState) -> ItemStyle {
    let mut output = merge_item_style(base, state);
    if active == ActiveState::Blur && !state.specified.contains("opacity") {
        output.opacity = base.opacity * 0.1;
        output.specified.insert(String::from("opacity"));
    }
    output
}

fn merge_state_line_style(base: &LineStyle, state: &LineStyle, active: ActiveState) -> LineStyle {
    let mut output = merge_line_style(base, state);
    if active == ActiveState::Blur && !state.specified.contains("opacity") {
        output.opacity = base.opacity * 0.1;
        output.specified.insert(String::from("opacity"));
    }
    output
}

fn apply_data_item_state(point: &mut DataPoint, state: ActiveState) {
    let key = match state {
        ActiveState::Emphasis => "emphasis",
        ActiveState::Blur => "blur",
        ActiveState::Select => "select",
        ActiveState::Normal => return,
    };
    let Some(state) = point.extra.get(key).and_then(serde_json::Value::as_object) else {
        return;
    };
    if let Some(style) = state.get("itemStyle") {
        point.item_style =
            merge_item_style(&point.item_style, &crate::parser::parse_item_style(style));
    }
    if let Some(label) = state.get("label") {
        point.label = merge_label(&point.label, &crate::parser::parse_label_style(label));
    }
}

fn merge_item_style(base: &ItemStyle, state: &ItemStyle) -> ItemStyle {
    let mut output = base.clone();
    if state.specified.contains("color") {
        output.color = state.color;
    }
    if state.specified.contains("color0") {
        output.color0 = state.color0;
    }
    if state.specified.contains("borderColor") {
        output.border_color = state.border_color;
    }
    if state.specified.contains("borderColor0") {
        output.border_color0 = state.border_color0;
    }
    if state.specified.contains("borderWidth") {
        output.border_width = state.border_width;
    }
    if state.specified.contains("borderRadius") {
        output.border_radius = state.border_radius;
    }
    if state.specified.contains("opacity") {
        output.opacity = state.opacity;
    }
    output.specified.extend(state.specified.iter().cloned());
    output
}

fn merge_line_style(base: &LineStyle, state: &LineStyle) -> LineStyle {
    let mut output = base.clone();
    if state.specified.contains("color") {
        output.color = state.color;
    }
    if state.specified.contains("width") {
        output.width = state.width;
    }
    if state.specified.contains("opacity") {
        output.opacity = state.opacity;
    }
    if state.specified.contains("type") {
        output.kind = state.kind.clone();
    }
    output.specified.extend(state.specified.iter().cloned());
    output
}

fn merge_label(base: &LabelStyle, state: &LabelStyle) -> LabelStyle {
    let mut output = base.clone();
    if state.specified.contains("show") {
        output.show = state.show;
    }
    if state.specified.contains("color") {
        output.color = state.color;
    }
    if state.specified.contains("fontSize") {
        output.font_size = state.font_size;
    }
    if state.specified.contains("fontWeight") {
        output.font_weight = state.font_weight;
    }
    if state.specified.contains("position") {
        output.position = state.position.clone();
    }
    if state.specified.contains("distance") {
        output.distance = state.distance;
    }
    if state.specified.contains("rotate") {
        output.rotate = state.rotate;
    }
    if state.specified.contains("offset") {
        output.offset = state.offset;
    }
    if state.specified.contains("formatter") {
        output.formatter = state.formatter.clone();
    }
    output.specified.extend(state.specified.iter().cloned());
    output
}

fn focused_items(series: &Series, hovered: usize, focus: &str) -> BTreeSet<usize> {
    let mut items = BTreeSet::from([hovered]);
    match (series, focus) {
        (Series::Graph(series), "adjacency") => {
            for link in &series.links {
                if link.source == hovered {
                    items.insert(link.target);
                } else if link.target == hovered {
                    items.insert(link.source);
                }
            }
        }
        (Series::Sankey(series), "adjacency") => {
            for link in &series.links {
                if link.source == hovered {
                    items.insert(link.target);
                } else if link.target == hovered {
                    items.insert(link.source);
                }
            }
        }
        (Series::Tree(series), "adjacency") => {
            for link in &series.links {
                if link.source == hovered {
                    items.insert(link.target);
                } else if link.target == hovered {
                    items.insert(link.source);
                }
            }
        }
        (Series::Tree(series), "ancestor") => {
            let mut current = hovered;
            while let Some(parent) = series
                .links
                .iter()
                .find_map(|link| (link.target == current).then_some(link.source))
            {
                if !items.insert(parent) {
                    break;
                }
                current = parent;
            }
        }
        (Series::Tree(series), "descendant") => {
            collect_descendants(&series.links, hovered, &mut items);
        }
        (Series::Sunburst(series), "ancestor" | "descendant") => {
            let mut parents = Vec::new();
            collect_sunburst_parents(&series.data, None, &mut parents);
            if focus == "ancestor" {
                let mut current = parents.get(hovered).copied().flatten();
                while let Some(parent) = current {
                    if !items.insert(parent) {
                        break;
                    }
                    current = parents.get(parent).copied().flatten();
                }
            } else {
                collect_parent_descendants(&parents, hovered, &mut items);
            }
        }
        _ => {}
    }
    items
}

fn collect_descendants(links: &[LinkData], parent: usize, output: &mut BTreeSet<usize>) {
    let children = links
        .iter()
        .filter_map(|link| (link.source == parent).then_some(link.target))
        .collect::<Vec<_>>();
    for child in children {
        if output.insert(child) {
            collect_descendants(links, child, output);
        }
    }
}

fn collect_sunburst_parents(
    nodes: &[SunburstNode],
    parent: Option<usize>,
    output: &mut Vec<Option<usize>>,
) {
    for node in nodes {
        let index = output.len();
        output.push(parent);
        collect_sunburst_parents(&node.children, Some(index), output);
    }
}

fn collect_parent_descendants(
    parents: &[Option<usize>],
    parent: usize,
    output: &mut BTreeSet<usize>,
) {
    for (index, candidate) in parents.iter().enumerate() {
        if *candidate == Some(parent) && output.insert(index) {
            collect_parent_descendants(parents, index, output);
        }
    }
}

fn coordinate_key(series_index: usize, series: &Series) -> String {
    let Some(options) = series_options(series) else {
        return format!("series:{series_index}");
    };
    let explicit = options
        .extra
        .get("coordinateSystem")
        .and_then(serde_json::Value::as_str);
    let coordinate = explicit.unwrap_or(match series {
        Series::Line(_)
        | Series::Bar(_)
        | Series::Scatter(_)
        | Series::EffectScatter(_)
        | Series::Candlestick(_)
        | Series::Boxplot(_)
        | Series::PictorialBar(_)
        | Series::Heatmap(_) => "cartesian2d",
        Series::Radar(_) => "radar",
        Series::Parallel(_) => "parallel",
        Series::ThemeRiver(_) => "singleAxis",
        Series::Lines(_) => "geo",
        Series::Graph(_) => "view",
        _ => "none",
    });
    let index = |key: &str| {
        options
            .extra
            .get(key)
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0)
    };
    match coordinate {
        "cartesian2d" => format!(
            "cartesian2d:{}:{}",
            index("xAxisIndex"),
            index("yAxisIndex")
        ),
        "polar" => format!("polar:{}", index("polarIndex")),
        "radar" => format!("radar:{}", index("radarIndex")),
        "parallel" => format!("parallel:{}", index("parallelIndex")),
        "singleAxis" => format!("singleAxis:{}", index("singleAxisIndex")),
        "calendar" => format!("calendar:{}", index("calendarIndex")),
        "geo" => format!("geo:{}", index("geoIndex")),
        // ECharts falls back to same-series when a series has no coordinate
        // system (pie/tree/graph-view/sankey/treemap/sunburst/etc.).
        _ => format!("series:{series_index}"),
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

pub(crate) fn selected_mode(series: &Series) -> Option<&str> {
    series_options(series)?.selected_mode.as_deref()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emphasis_and_focus_blur_are_applied_across_series() {
        let mut option = ChartOption::from_json_str(
            r##"{
                "series":[
                    {"type":"scatter","data":[[1,2]],"emphasis":{"focus":"series","itemStyle":{"color":"#ff0000"}}},
                    {"type":"scatter","data":[[2,3]],"blur":{"itemStyle":{"opacity":0.2}}}
                ]
            }"##,
        )
        .unwrap();
        apply_states(
            &mut option,
            Some(&ChartEvent {
                series_index: 0,
                data_index: 0,
                series_name: None,
                name: None,
                value: vec![1.0, 2.0],
                x: 0.0,
                y: 0.0,
                component_type: String::from("scatter"),
            }),
            &BTreeSet::new(),
        );
        let Series::Scatter(first) = &option.series[0] else {
            panic!("scatter")
        };
        let Series::Scatter(second) = &option.series[1] else {
            panic!("scatter")
        };
        assert_eq!(first.data[0].item_style.color, Some(0xFFFF0000));
        assert_eq!(second.options.item_style.opacity, 0.2);
    }

    #[test]
    fn self_focus_honors_series_scope_and_uses_echarts_default_blur() {
        let mut option = ChartOption::from_json_str(
            r#"{
                "series":[
                    {"type":"scatter","data":[[1,2],[2,3]],"emphasis":{"focus":"self","blurScope":"series"}},
                    {"type":"scatter","data":[[3,4]]}
                ]
            }"#,
        )
        .unwrap();
        apply_states(
            &mut option,
            Some(&ChartEvent {
                series_index: 0,
                data_index: 0,
                series_name: None,
                name: None,
                value: vec![1.0, 2.0],
                x: 0.0,
                y: 0.0,
                component_type: String::from("scatter"),
            }),
            &BTreeSet::new(),
        );
        let Series::Scatter(first) = &option.series[0] else {
            panic!("scatter")
        };
        let Series::Scatter(second) = &option.series[1] else {
            panic!("scatter")
        };
        assert_eq!(first.data[0].item_style.opacity, 1.0);
        assert_eq!(first.data[1].item_style.opacity, 0.1);
        assert_eq!(second.data[0].item_style.opacity, 1.0);
    }

    #[test]
    fn graph_adjacency_focus_preserves_neighbors_and_blurs_unrelated_nodes() {
        let mut option = ChartOption::from_json_str(
            r#"{
                "series":[{
                    "type":"graph",
                    "data":[{"name":"a"},{"name":"b"},{"name":"c"}],
                    "links":[{"source":0,"target":1}],
                    "emphasis":{"focus":"adjacency","blurScope":"series"}
                }]
            }"#,
        )
        .unwrap();
        apply_states(
            &mut option,
            Some(&ChartEvent {
                series_index: 0,
                data_index: 0,
                series_name: None,
                name: Some(String::from("a")),
                value: vec![0.0],
                x: 0.0,
                y: 0.0,
                component_type: String::from("graph"),
            }),
            &BTreeSet::new(),
        );
        let Series::Graph(graph) = &option.series[0] else {
            panic!("graph")
        };
        assert_eq!(graph.nodes[0].item_style.opacity, 1.0);
        assert_eq!(graph.nodes[1].item_style.opacity, 1.0);
        assert_eq!(graph.nodes[2].item_style.opacity, 0.1);
    }
}
