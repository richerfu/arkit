//! Chart-level orchestration. This module lays out shared chrome and delegates
//! each series to its own renderer; it contains no series drawing logic.

use ohos_drawing_binding::Canvas;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

use super::cartesian;
use super::chrome::{
    draw_brush, draw_data_view, draw_data_zoom, draw_legend, draw_timeline, draw_title,
    draw_toolbox, draw_tooltip, draw_visual_map,
};
use super::geometry::{effective_palette, Plot};
use super::graphic::draw_graphic;
use super::hit::{rect_hit, HitRegion};
use super::layout::grid_plot;
use super::series;
use super::surface::fill_rect;
use super::viewport::{initial_windows, ZoomWindow};
use crate::model::{ChartEvent, ChartOption, DataPoint, Series};

pub fn hit_test(
    option: &ChartOption,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> Option<ChartEvent> {
    hit_test_with_hidden(
        option,
        x,
        y,
        width,
        height,
        &BTreeSet::new(),
        &initial_windows(option),
    )
}

pub(crate) fn hit_test_with_hidden(
    option: &ChartOption,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    hidden_series: &BTreeSet<usize>,
    zoom_windows: &[ZoomWindow],
) -> Option<ChartEvent> {
    let selected_items = BTreeSet::new();
    render_option(
        option,
        None,
        hidden_series,
        zoom_windows,
        &selected_items,
        None,
        width,
        height,
    )
    .into_iter()
    .rev()
    .filter_map(|region| region.hit(x, y).map(|distance| (distance, region.event)))
    .min_by(|left, right| left.0.total_cmp(&right.0))
    .map(|(_, event)| event)
}

pub(crate) fn nearest_axis_event(
    option: &ChartOption,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    hidden_series: &BTreeSet<usize>,
    zoom_windows: &[ZoomWindow],
) -> Option<ChartEvent> {
    let selected_items = BTreeSet::new();
    render_option(
        option,
        None,
        hidden_series,
        zoom_windows,
        &selected_items,
        None,
        width,
        height,
    )
    .into_iter()
    .map(|region| region.event)
    .filter(|event| {
        let Some(value) = option.series.get(event.series_index) else {
            return false;
        };
        if !series::is_cartesian(value) {
            return false;
        }
        let (x_axis_index, y_axis_index) = series::cartesian_axis_indices(value);
        let grid_index = option
            .x_axis
            .get(x_axis_index)
            .map(|axis| axis.grid_index)
            .or_else(|| option.y_axis.get(y_axis_index).map(|axis| axis.grid_index))
            .unwrap_or(0);
        let plot = grid_plot(option, grid_index, width, height);
        x >= plot.x && x <= plot.x + plot.width && y >= plot.y && y <= plot.y + plot.height
    })
    .min_by(|left, right| {
        let left_distance = (left.x - x).abs() + (left.y - y).abs() * 0.05;
        let right_distance = (right.x - x).abs() + (right.y - y).abs() * 0.05;
        left_distance.total_cmp(&right_distance)
    })
}

pub(super) fn render_option(
    option: &ChartOption,
    selected: Option<&ChartEvent>,
    hidden_series: &BTreeSet<usize>,
    zoom_windows: &[ZoomWindow],
    selected_items: &BTreeSet<(usize, usize)>,
    canvas: Option<&Canvas>,
    width: f32,
    height: f32,
) -> Vec<HitRegion> {
    render_option_with_domain(
        option,
        option,
        selected,
        hidden_series,
        zoom_windows,
        selected_items,
        canvas,
        width,
        height,
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn render_option_with_domain(
    option: &ChartOption,
    domain_option: &ChartOption,
    selected: Option<&ChartEvent>,
    hidden_series: &BTreeSet<usize>,
    zoom_windows: &[ZoomWindow],
    selected_items: &BTreeSet<(usize, usize)>,
    canvas: Option<&Canvas>,
    width: f32,
    height: f32,
) -> Vec<HitRegion> {
    let width = width.max(1.0);
    let height = height.max(1.0);
    let mut hits = Vec::new();
    let palette = effective_palette(option);
    let view = Plot {
        x: 0.0,
        y: 0.0,
        width,
        height,
    };
    super::label_layout::begin_frame(width, height);

    if let Some(canvas) = canvas {
        fill_rect(
            canvas,
            0.0,
            0.0,
            width,
            height,
            option.visual_style.background_color,
        );
        if let Some(title) = &option.title {
            draw_title(canvas, option, title, width, height);
        }
        draw_visual_map(canvas, option, width, height);
    }
    draw_legend(
        canvas,
        option,
        width,
        height,
        &palette,
        hidden_series,
        &mut hits,
    );
    let mut cartesian_groups: BTreeMap<(usize, usize), Vec<usize>> = BTreeMap::new();
    for (series_index, value) in option.series.iter().enumerate() {
        if series::is_cartesian(value) && !hidden_series.contains(&series_index) {
            cartesian_groups
                .entry(series::cartesian_axis_indices(value))
                .or_default()
                .push(series_index);
        }
    }
    let mut drawn_x_axes = BTreeSet::new();
    let mut drawn_y_axes = BTreeSet::new();
    for ((x_axis_index, y_axis_index), series_indices) in cartesian_groups {
        let grid_index = option
            .x_axis
            .get(x_axis_index)
            .map(|axis| axis.grid_index)
            .or_else(|| option.y_axis.get(y_axis_index).map(|axis| axis.grid_index))
            .unwrap_or(0);
        let plot = grid_plot(option, grid_index, width, height);
        cartesian::render(cartesian::CartesianChartRenderContext {
            option,
            domain_option,
            series_indices: &series_indices,
            plot: &plot,
            axis_indices: (x_axis_index, y_axis_index),
            palette: &palette,
            canvas,
            hits: &mut hits,
            zoom_windows,
            selected,
            draw_x_axis: drawn_x_axes.insert(x_axis_index),
            draw_y_axis: drawn_y_axes.insert(y_axis_index),
        });
    }

    let free_series: Vec<(usize, &Series)> = option
        .series
        .iter()
        .enumerate()
        .filter(|(index, value)| !series::is_cartesian(value) && !hidden_series.contains(index))
        .collect();
    for (series_index, value) in free_series {
        series::render_free(
            option,
            series_index,
            value,
            view,
            &palette,
            canvas,
            &mut hits,
            selected,
            selected_items,
        );
    }

    if let Some(canvas) = canvas {
        draw_graphic(canvas, option, width, height);
    }

    draw_brush(canvas, option, width, height, &hits);

    draw_data_zoom(canvas, option, zoom_windows, width, height, &mut hits);
    draw_timeline(canvas, option, width, height, &mut hits);
    draw_toolbox(canvas, option, width, height, &mut hits);

    if let (Some(canvas), Some(selected)) = (canvas, selected) {
        if option.tooltip.show {
            draw_tooltip(canvas, option, selected, hidden_series, width, height);
        }
    }
    draw_data_view(canvas, option, width, height, &mut hits);

    for label in super::label_layout::take_draggable_hits() {
        let point = DataPoint::named(label.text, 0.0);
        hits.push(rect_hit(
            "label",
            label.series_index,
            label.label_index,
            option
                .series
                .get(label.series_index)
                .and_then(Series::name)
                .map(ToOwned::to_owned),
            &point,
            (
                label.bounds.x,
                label.bounds.y,
                label.bounds.width,
                label.bounds.height,
            ),
        ));
    }

    hits
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Series;
    use crate::render::hit::HitShape;

    #[test]
    fn hit_test_returns_data_event() {
        let option = ChartOption::new().push_series(Series::bar("B", [10.0]));
        let hit = hit_test(&option, 210.0, 60.0, 320.0, 180.0).unwrap();
        assert_eq!(hit.component_type, "bar");
        assert_eq!(hit.series_index, 0);
        assert_eq!(hit.data_index, 0);
    }

    #[test]
    fn missing_cartesian_values_do_not_create_hit_regions() {
        let option = ChartOption::from_json_str(
            r#"{
                "xAxis":{"type":"category","data":["A","B","C"]},
                "series":[{"type":"line","data":[10,null,20]}]
            }"#,
        )
        .unwrap();
        let windows = initial_windows(&option);
        let hits = render_option(
            &option,
            None,
            &BTreeSet::new(),
            &windows,
            &BTreeSet::new(),
            None,
            320.0,
            180.0,
        );
        let indices = hits
            .into_iter()
            .filter(|hit| hit.event.component_type == "line")
            .map(|hit| hit.event.data_index)
            .collect::<Vec<_>>();
        assert_eq!(indices, [0, 2]);
    }

    #[test]
    fn horizontal_bar_creates_horizontal_hit_regions() {
        let option = ChartOption::new()
            .x_axis(crate::model::Axis::value())
            .y_axis(crate::model::Axis::category(["A", "B"]))
            .push_series(Series::bar("B", [10.0, 20.0]));
        let windows = initial_windows(&option);
        let hits = render_option(
            &option,
            None,
            &BTreeSet::new(),
            &windows,
            &BTreeSet::new(),
            None,
            360.0,
            220.0,
        );
        let bounds = hits
            .iter()
            .find_map(|hit| match hit.shape {
                crate::render::hit::HitShape::Rect { width, height, .. }
                    if hit.event.component_type == "bar" =>
                {
                    Some((width, height))
                }
                _ => None,
            })
            .expect("horizontal bar hit region");
        assert!(
            bounds.0 > bounds.1,
            "expected a horizontal rectangle: {bounds:?}"
        );
    }

    #[test]
    fn visual_map_symbol_size_controls_scatter_hit_extent() {
        let option = ChartOption::from_json_str(
            r##"{
                "visualMap":{"show":false,"min":0,"max":100,"dimension":2,
                    "inRange":{"symbolSize":[10,50]}},
                "xAxis":{"type":"value"},"yAxis":{"type":"value"},
                "series":[{"type":"scatter","data":[[2,4,100]]}]
            }"##,
        )
        .unwrap();
        let windows = initial_windows(&option);
        let hits = render_option(
            &option,
            None,
            &BTreeSet::new(),
            &windows,
            &BTreeSet::new(),
            None,
            360.0,
            220.0,
        );
        let radius = hits
            .iter()
            .find_map(|hit| match hit.shape {
                crate::render::hit::HitShape::Point { radius, .. }
                    if hit.event.component_type == "scatter" =>
                {
                    Some(radius)
                }
                _ => None,
            })
            .expect("scatter hit region");
        assert_eq!(radius, 25.0);
    }

    #[test]
    fn polar_bar_and_scatter_create_native_hit_regions() {
        let option = ChartOption::from_json_str(
            r#"{
                "polar":{},"angleAxis":{"type":"category","data":["A","B"]},
                "radiusAxis":{"max":100},
                "series":[
                    {"type":"bar","coordinateSystem":"polar","data":[[40,0],[70,1]]},
                    {"type":"scatter","coordinateSystem":"polar","data":[[60,0]]}
                ]
            }"#,
        )
        .unwrap();
        let hits = render_option(
            &option,
            None,
            &BTreeSet::new(),
            &initial_windows(&option),
            &BTreeSet::new(),
            None,
            360.0,
            240.0,
        );
        assert_eq!(
            hits.iter()
                .filter(|hit| hit.event.component_type == "bar")
                .count(),
            2
        );
        assert_eq!(
            hits.iter()
                .filter(|hit| hit.event.component_type == "scatter")
                .count(),
            1
        );
    }

    #[test]
    fn geo_coordinate_projects_overlay_series() {
        let option = ChartOption::from_json_str(
            r#"{
                "geo":{"aspectScale":1,"geoJson":{"type":"FeatureCollection","features":[
                    {"type":"Feature","properties":{"name":"A"},"geometry":{"type":"Polygon","coordinates":[[[0,0],[10,0],[10,10],[0,10],[0,0]]]}}
                ]}},
                "series":[
                    {"type":"scatter","coordinateSystem":"geo","data":[[2,3,1]]},
                    {"type":"effectScatter","coordinateSystem":"geo","data":[[7,8,2]]},
                    {"type":"heatmap","coordinateSystem":"geo","data":[[4,5,3]]},
                    {"type":"lines","coordinateSystem":"geo","data":[{"coords":[[1,1],[9,9]]}]},
                    {"type":"graph","coordinateSystem":"geo","data":[{"name":"N","x":5,"y":6}]}
                ]
            }"#,
        )
        .unwrap();
        assert!(option
            .series
            .iter()
            .all(|series| !series::is_cartesian(series)));
        let hits = render_option(
            &option,
            None,
            &BTreeSet::new(),
            &initial_windows(&option),
            &BTreeSet::new(),
            None,
            360.0,
            240.0,
        );
        for component in ["scatter", "effectScatter", "heatmap", "lines", "graph"] {
            assert!(
                hits.iter().any(|hit| hit.event.component_type == component),
                "missing {component} geo hit"
            );
        }
    }

    #[test]
    fn hierarchical_treemap_renders_parent_and_child_regions() {
        let option = ChartOption::from_json_str(
            r#"{"series":[{"type":"treemap","data":[{
                "name":"root","value":10,"children":[
                    {"name":"a","value":6},{"name":"b","value":4}
                ]
            }]}]}"#,
        )
        .unwrap();
        let hits = render_option(
            &option,
            None,
            &BTreeSet::new(),
            &initial_windows(&option),
            &BTreeSet::new(),
            None,
            360.0,
            240.0,
        );
        assert_eq!(
            hits.iter()
                .filter(|hit| hit.event.component_type == "treemap")
                .count(),
            3
        );
    }

    #[test]
    fn calendar_heatmap_uses_free_coordinate_renderer() {
        let option = ChartOption::from_json_str(
            r#"{
                "calendar":{"range":["2026-01-01","2026-01-31"]},
                "series":[{"type":"heatmap","coordinateSystem":"calendar","data":[
                    ["2026-01-03",2],["2026-01-12",5]
                ]}]
            }"#,
        )
        .unwrap();
        assert!(!series::is_cartesian(&option.series[0]));
        let hits = render_option(
            &option,
            None,
            &BTreeSet::new(),
            &initial_windows(&option),
            &BTreeSet::new(),
            None,
            360.0,
            240.0,
        );
        assert_eq!(
            hits.iter()
                .filter(|hit| hit.event.component_type == "heatmap")
                .count(),
            2
        );
    }

    #[test]
    fn toolbox_restore_creates_a_native_action_region() {
        let option =
            ChartOption::from_json_str(r#"{"toolbox":{"show":true,"feature":{"restore":{}}}}"#)
                .unwrap();
        let hits = render_option(
            &option,
            None,
            &BTreeSet::new(),
            &initial_windows(&option),
            &BTreeSet::new(),
            None,
            360.0,
            240.0,
        );
        let restore = hits
            .iter()
            .find(|hit| hit.event.component_type == "toolbox")
            .expect("toolbox restore hit");
        assert_eq!(restore.event.name.as_deref(), Some("restore"));
        assert!(matches!(
            restore.shape,
            HitShape::Rect {
                x: 335.0,
                y: 10.0,
                width: 15.0,
                height: 15.0
            }
        ));
    }

    #[test]
    fn timeline_nodes_and_controls_create_native_action_regions() {
        let option = ChartOption::from_json_str(
            r#"{
                "baseOption":{"timeline":{"data":["2024","2025"]},"series":[{"type":"bar","data":[1]}]},
                "options":[{"series":[{"type":"bar","data":[2]}]},{"series":[{"type":"bar","data":[3]}]}]
            }"#,
        )
        .unwrap();
        let hits = render_option(
            &option,
            None,
            &BTreeSet::new(),
            &initial_windows(&option),
            &BTreeSet::new(),
            None,
            360.0,
            240.0,
        );
        let timeline = hits
            .iter()
            .filter(|hit| hit.event.component_type == "timeline")
            .collect::<Vec<_>>();
        assert_eq!(timeline.len(), 5);
        assert!(timeline
            .iter()
            .any(|hit| hit.event.name.as_deref() == Some("timeline-play")));
    }

    #[test]
    fn horizontal_legend_wraps_and_disabled_selection_has_no_hits() {
        let option = ChartOption::from_json_str(
            r#"{
                "legend":{"itemGap":12,"formatter":"Series {name}"},
                "series":[
                    {"type":"line","name":"Alpha","data":[1]},
                    {"type":"line","name":"Beta","data":[2]},
                    {"type":"line","name":"Gamma","data":[3]}
                ]
            }"#,
        )
        .unwrap();
        let hits = render_option(
            &option,
            None,
            &BTreeSet::new(),
            &initial_windows(&option),
            &BTreeSet::new(),
            None,
            190.0,
            240.0,
        );
        let legend_hits = hits
            .iter()
            .filter(|hit| hit.event.component_type == "legend")
            .collect::<Vec<_>>();
        assert_eq!(legend_hits.len(), 3);
        let ys = legend_hits
            .iter()
            .map(|hit| match hit.shape {
                HitShape::Rect { y, .. } => y,
                _ => panic!("legend uses rectangular hit regions"),
            })
            .collect::<Vec<_>>();
        assert!(ys.iter().any(|y| *y > ys[0]));

        let disabled = ChartOption::from_json_str(
            r#"{"legend":{"selectedMode":false},"series":[{"type":"line","name":"A","data":[1]}]}"#,
        )
        .unwrap();
        let hits = render_option(
            &disabled,
            None,
            &BTreeSet::new(),
            &initial_windows(&disabled),
            &BTreeSet::new(),
            None,
            190.0,
            240.0,
        );
        assert!(!hits.iter().any(|hit| hit.event.component_type == "legend"));
    }

    #[test]
    fn toolbox_brush_exposes_activate_and_clear_actions() {
        let option = ChartOption::from_json_str(
            r#"{"toolbox":{"feature":{"brush":{"type":["rect","clear"]}}}}"#,
        )
        .unwrap();
        let hits = render_option(
            &option,
            None,
            &BTreeSet::new(),
            &initial_windows(&option),
            &BTreeSet::new(),
            None,
            360.0,
            240.0,
        );
        let names = hits
            .iter()
            .filter(|hit| hit.event.component_type == "toolbox")
            .filter_map(|hit| hit.event.name.as_deref())
            .collect::<Vec<_>>();
        assert_eq!(names, ["brush-rect", "brush-clear"]);
    }

    #[test]
    fn toolbox_honors_vertical_bottom_right_box_layout() {
        let option = ChartOption::from_json_str(
            r#"{
                "toolbox":{
                    "orient":"vertical","right":12,"bottom":18,
                    "itemSize":20,"itemGap":6,"padding":[4,8],
                    "feature":{"brush":{"type":["rect","clear"]},"restore":{}}
                }
            }"#,
        )
        .unwrap();
        let hits = render_option(
            &option,
            None,
            &BTreeSet::new(),
            &initial_windows(&option),
            &BTreeSet::new(),
            None,
            360.0,
            240.0,
        );
        let boxes = hits
            .iter()
            .filter(|hit| hit.event.component_type == "toolbox")
            .map(|hit| match hit.shape {
                HitShape::Rect {
                    x,
                    y,
                    width,
                    height,
                } => (x, y, width, height),
                _ => panic!("toolbox actions use rectangular hit regions"),
            })
            .collect::<Vec<_>>();
        assert_eq!(
            boxes,
            [
                (320.0, 146.0, 20.0, 20.0),
                (320.0, 172.0, 20.0, 20.0),
                (320.0, 198.0, 20.0, 20.0)
            ]
        );
    }

    #[test]
    fn toolbox_exposes_data_zoom_magic_type_and_native_host_actions() {
        let option = ChartOption::from_json_str(
            r#"{
                "toolbox":{"feature":{
                    "dataZoom":{},
                    "magicType":{"type":["line","bar","stack"]},
                    "dataView":{},"saveAsImage":{}
                }},
                "xAxis":{"type":"category","data":["A","B"]},
                "series":[{"type":"line","data":[1,2]}]
            }"#,
        )
        .unwrap();
        let hits = render_option(
            &option,
            None,
            &BTreeSet::new(),
            &initial_windows(&option),
            &BTreeSet::new(),
            None,
            480.0,
            240.0,
        );
        let names = hits
            .iter()
            .filter(|hit| hit.event.component_type == "toolbox")
            .filter_map(|hit| hit.event.name.as_deref())
            .collect::<Vec<_>>();
        assert_eq!(
            names,
            [
                "data-view",
                "data-zoom",
                "data-zoom-back",
                "magic-line",
                "magic-bar",
                "magic-stack",
                "save-as-image"
            ]
        );
    }

    #[test]
    fn visible_data_view_blocks_chart_and_exposes_close_action() {
        let option = ChartOption::from_json_str(
            r#"{
                "toolbox":{"feature":{"dataView":{"readOnly":true,"__visible":true}}},
                "xAxis":{"type":"category","data":["A","B"]},
                "series":[{"type":"line","name":"Revenue","data":[12,18]}]
            }"#,
        )
        .unwrap();
        let hits = render_option(
            &option,
            None,
            &BTreeSet::new(),
            &initial_windows(&option),
            &BTreeSet::new(),
            None,
            360.0,
            240.0,
        );
        let names = hits
            .iter()
            .filter(|hit| hit.event.component_type == "toolbox")
            .filter_map(|hit| hit.event.name.as_deref())
            .collect::<Vec<_>>();
        assert!(names.contains(&"data-view-overlay"));
        assert!(names.contains(&"data-view-close"));
        let overlay = hit_test(&option, 180.0, 120.0, 360.0, 240.0).unwrap();
        assert_eq!(overlay.name.as_deref(), Some("data-view-overlay"));
    }
}
