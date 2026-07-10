//! Dioxus-native ECharts example.
//!
//! The first chart is rebuilt from a signal every second; the remaining
//! cards exercise the supported series families and ECharts-like JSON parser.

use std::time::Duration;

use arkit::entry;
use arkit::prelude::*;

#[entry]
fn app() -> Element {
    let mut tick = use_signal(|| 0_u32);
    let mut selected = use_signal(|| String::from("Tap a chart item to inspect it"));
    let handle = arkit::tokio_handle();

    use_future(move || {
        let handle = handle.clone();
        async move {
            loop {
                let _ = handle
                    .spawn(async {
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    })
                    .await;
                tick += 1;
            }
        }
    });

    let realtime_option = realtime_option(tick());
    let selected_label = selected();
    let cards: Vec<Element> = gallery_options()
        .into_iter()
        .map(|(title, option)| {
            rsx! {
                DemoChart {
                    key: "{title}",
                    title,
                    option,
                    on_select: move |event: ChartEvent| {
                        selected.set(selection_label(&event));
                    },
                }
            }
        })
        .collect();

    rsx! {
        column {
            percent_width: 1.0,
            percent_height: 1.0,
            background_color: 0xFFF1F5F9u32,

            column {
                padding: 16.0,
                background_color: 0xFFFFFFFFu32,
                text {
                    font_size: 24.0,
                    line_height: 30.0,
                    font_weight: 700,
                    "arkit ECharts"
                }
                text {
                    margin_top: 4.0,
                    font_size: 13.0,
                    line_height: 18.0,
                    font_color: 0xFF475569u32,
                    "Dioxus props + native ArkUI canvas; live tick #{tick}"
                }
                text {
                    margin_top: 6.0,
                    font_size: 13.0,
                    line_height: 18.0,
                    font_color: 0xFF2563EBu32,
                    "{selected_label}"
                }
                button {
                    margin_top: 10.0,
                    width: 160.0,
                    onclick: move |_| tick += 1,
                    "Update now"
                }
            }

            scroll {
                percent_width: 1.0,
                percent_height: 1.0,
                scroll_bar: true,
                column {
                    percent_width: 1.0,
                    padding: 12.0,
                    DemoChart {
                        title: "Realtime line / bar / scatter",
                        option: realtime_option,
                        on_select: move |event: ChartEvent| {
                            selected.set(selection_label(&event));
                        },
                    }
                    {cards.into_iter()}
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct DemoChartProps {
    title: String,
    option: ChartOption,
    on_select: EventHandler<ChartEvent>,
}

#[component]
fn DemoChart(props: DemoChartProps) -> Element {
    rsx! {
        column {
            percent_width: 1.0,
            margin_bottom: 12.0,
            padding: 10.0,
            background_color: 0xFFFFFFFFu32,
            border_width: 1.0,
            border_color: 0xFFE2E8F0u32,
            border_radius: 10.0,
            text {
                margin_bottom: 6.0,
                font_size: 15.0,
                line_height: 20.0,
                font_weight: 600,
                font_color: 0xFF0F172Au32,
                "{props.title}"
            }
            ECharts {
                option: props.option,
                height: 300.0,
                on_select: props.on_select,
            }
        }
    }
}

fn selection_label(event: &ChartEvent) -> String {
    format!(
        "selected {} / series {} / item {} / {:?}",
        event.component_type, event.series_index, event.data_index, event.value
    )
}

fn realtime_option(tick: u32) -> ChartOption {
    let wave = |offset: u32| {
        (0..8)
            .map(|index| 25.0 + ((index as u32 * 7 + tick * 5 + offset) % 36) as f64)
            .collect::<Vec<_>>()
    };
    ChartOption::new()
        .title(format!("Realtime update #{tick}"))
        .x_axis(Axis::category([
            "00:00", "03:00", "06:00", "09:00", "12:00", "15:00", "18:00", "21:00",
        ]))
        .push_series(Series::line("Load", wave(0)))
        .push_series(Series::bar("Requests", wave(11)))
        .push_series(Series::scatter(
            "Alerts",
            [
                DataPoint::values([1.0, 48.0 + (tick % 8) as f64]),
                DataPoint::values([5.0, 35.0 + (tick % 12) as f64]),
            ],
        ))
}

fn gallery_options() -> Vec<(String, ChartOption)> {
    vec![
        (
            String::from("ECharts-like JSON option"),
            ChartOption::from_json_str(
                r##"{
                    "title": {"text": "JSON option"},
                    "xAxis": {"type": "category", "data": ["Mon", "Tue", "Wed", "Thu"]},
                    "yAxis": {"type": "value"},
                    "color": ["#2563eb", "#f97316"],
                    "series": [
                        {"type": "bar", "name": "Orders", "data": [12, 20, 15, 28]},
                        {"type": "line", "name": "Revenue", "data": [8, 18, 22, 30]}
                    ]
                }"##,
            )
            .expect("valid chart option"),
        ),
        (
            String::from("dataZoom / axisPointer / markers"),
            ChartOption::from_json_str(
                r##"{
                    "title":{"text":"Interactive Cartesian"},
                    "tooltip":{"trigger":"axis","axisPointer":{"type":"cross","snap":true}},
                    "grid":{"left":"10%","right":"8%","top":60,"bottom":55},
                    "xAxis":{"type":"category","data":["Jan","Feb","Mar","Apr","May","Jun","Jul","Aug","Sep","Oct","Nov","Dec"]},
                    "yAxis":{"type":"value"},
                    "dataZoom":[
                        {"type":"slider","startValue":"Feb","endValue":"Sep","height":16,"bottom":10},
                        {"type":"inside","start":0,"end":100}
                    ],
                    "series":[{
                        "type":"line","name":"Revenue","smooth":true,
                        "data":[18,24,22,31,38,35,46,52,49,61,67,72],
                        "areaStyle":{"opacity":0.18},
                        "markPoint":{"data":[{"type":"max","name":"Max"},{"type":"min","name":"Min"}]},
                        "markLine":{"lineStyle":{"color":"#ee6666"},"data":[{"type":"average","name":"Average"}]},
                        "markArea":{"itemStyle":{"color":"rgba(250,200,88,0.24)"},"data":[[{"xAxis":"Apr"},{"xAxis":"Jun"}]]}
                    }]
                }"##,
            )
            .expect("valid data zoom and marker option"),
        ),
        (
            String::from("Pie"),
            ChartOption::new().title("Pie").push_series(Series::pie(
                "Share",
                [
                    DataPoint::named("Search", 40.0),
                    DataPoint::named("Direct", 24.0),
                    DataPoint::named("Social", 18.0),
                    DataPoint::named("Other", 12.0),
                ],
            )),
        ),
        (
            String::from("Radar"),
            ChartOption::from_json_str(
                r##"{
                    "title": {"text": "Radar"},
                    "radar": {
                        "indicator": [
                            {"name": "Speed", "max": 100},
                            {"name": "Reliability", "max": 100},
                            {"name": "Comfort", "max": 100},
                            {"name": "Safety", "max": 100},
                            {"name": "Efficiency", "max": 100}
                        ]
                    },
                    "series": [{
                        "type": "radar",
                        "name": "Quality",
                        "areaStyle": {"opacity": 0.28},
                        "data": [{"name": "Score", "value": [42, 64, 35, 76, 58]}]
                    }]
                }"##,
            )
            .expect("valid radar option"),
        ),
        (
            String::from("Gauge"),
            ChartOption::from_json_str(
                r##"{
                    "title": {"text": "Gauge"},
                    "series": [{
                        "type": "gauge",
                        "name": "Completion",
                        "progress": {"show": true},
                        "axisLine": {"lineStyle": {"width": 14, "color": [[0.3,"#67e0e3"],[0.7,"#37a2da"],[1,"#fd666d"]]}},
                        "detail": {"formatter": "{value}%", "fontSize": 18},
                        "data": [{"name": "Done", "value": 68}]
                    }]
                }"##,
            )
            .expect("valid gauge option"),
        ),
        (
            String::from("Funnel"),
            ChartOption::from_json_str(
                r##"{
                    "title": {"text": "Conversion"},
                    "series": [{
                        "type": "funnel",
                        "name": "Conversion",
                        "gap": 2,
                        "label": {"show": true, "position": "inside", "formatter": "{b}"},
                        "data": [
                            {"name": "Visit", "value": 100},
                            {"name": "Lead", "value": 60},
                            {"name": "Deal", "value": 26}
                        ]
                    }]
                }"##,
            )
            .expect("valid funnel option"),
        ),
        (
            String::from("Heatmap"),
            ChartOption::new()
                .title("Heatmap")
                .x_axis(Axis::category(["Mon", "Tue", "Wed"]))
                .y_axis(Axis::category(["Morning", "Noon", "Evening"]))
                .push_series(Series::heatmap(
                    "Heat",
                    [
                        DataPoint::values([0.0, 0.0, 4.0]),
                        DataPoint::values([1.0, 0.0, 8.0]),
                        DataPoint::values([2.0, 0.0, 2.0]),
                        DataPoint::values([0.0, 1.0, 5.0]),
                        DataPoint::values([1.0, 1.0, 10.0]),
                        DataPoint::values([2.0, 1.0, 7.0]),
                        DataPoint::values([0.0, 2.0, 8.0]),
                        DataPoint::values([1.0, 2.0, 6.0]),
                        DataPoint::values([2.0, 2.0, 9.0]),
                    ],
                )),
        ),
        (
            String::from("Candlestick"),
            ChartOption::new()
                .title("OHLC")
                .x_axis(Axis::category(["Mon", "Tue", "Wed", "Thu"]))
                .push_series(Series::candlestick(
                    "OHLC",
                    [
                        DataPoint::values([20.0, 32.0, 18.0, 36.0]),
                        DataPoint::values([32.0, 28.0, 24.0, 35.0]),
                        DataPoint::values([28.0, 40.0, 26.0, 44.0]),
                        DataPoint::values([40.0, 36.0, 31.0, 46.0]),
                    ],
                )),
        ),
        (
            String::from("Boxplot / effectScatter / pictorialBar"),
            ChartOption::from_json_str(
                r##"{
                    "title":{"text":"More Cartesian series"},
                    "xAxis":{"type":"category","data":["A","B","C","D"]},
                    "yAxis":{"type":"value"},
                    "series":[
                        {"type":"boxplot","name":"Distribution","data":[[8,12,18,24,31],[10,15,20,27,35],[6,11,16,22,29],[12,17,23,30,38]]},
                        {"type":"effectScatter","name":"Outlier","symbolSize":12,"data":[[1,35],[3,40]]},
                        {"type":"pictorialBar","name":"Count","symbolRepeat":true,"symbolSize":8,"itemStyle":{"color":"#91cc75"},"data":[10,14,8,12]}
                    ]
                }"##,
            )
            .expect("valid extended cartesian option"),
        ),
        (
            String::from("Parallel"),
            ChartOption::from_json_str(
                r##"{
                    "title":{"text":"Parallel"},
                    "parallelAxis":[{"name":"Price"},{"name":"Quality"},{"name":"Speed"},{"name":"Safety"}],
                    "series":[{"type":"parallel","lineStyle":{"width":2},"data":[[42,80,65,92],[68,55,88,70],[54,72,74,84]]}]
                }"##,
            )
            .expect("valid parallel option"),
        ),
        (
            String::from("ThemeRiver"),
            ChartOption::from_json_str(
                r##"{
                    "title":{"text":"ThemeRiver"},
                    "series":[{"type":"themeRiver","data":[
                        ["2026-01-01",12,"Search"],["2026-01-02",18,"Search"],["2026-01-03",14,"Search"],
                        ["2026-01-01",8,"Direct"],["2026-01-02",10,"Direct"],["2026-01-03",16,"Direct"]
                    ]}]
                }"##,
            )
            .expect("valid theme river option"),
        ),
        (
            String::from("Lines"),
            ChartOption::from_json_str(
                r##"{
                    "title":{"text":"Lines"},
                    "series":[{"type":"lines","lineStyle":{"color":"#5470c6","width":2},"data":[
                        {"name":"A → B","coords":[[0,0],[2,1]],"value":1},
                        {"name":"B → C","coords":[[2,1],[3,3]],"value":1.5},
                        {"name":"A → C","coords":[[0,0],[3,3]],"value":0.8}
                    ]}]
                }"##,
            )
            .expect("valid lines option"),
        ),
        (
            String::from("Sunburst"),
            ChartOption::from_json_str(
                r##"{
                    "title":{"text":"Sunburst"},
                    "series":[{"type":"sunburst","radius":["12%","78%"],"data":[
                        {"name":"A","value":6,"children":[{"name":"A1","value":2},{"name":"A2","value":4}]},
                        {"name":"B","value":4,"children":[{"name":"B1","value":1},{"name":"B2","value":3}]}
                    ]}]
                }"##,
            )
            .expect("valid sunburst option"),
        ),
        (
            String::from("Tree"),
            ChartOption::from_json_str(
                r#"{
                    "title": {"text": "Tree"},
                    "series": [{"type":"tree","orient":"LR","top":"10%","data":[{
                        "name":"Root","children":[
                            {"name":"A","children":[{"name":"A1"},{"name":"A2"}]},
                            {"name":"B","children":[{"name":"B1"}]}
                        ]
                    }]}]
                }"#,
            )
            .expect("valid tree option"),
        ),
        (
            String::from("Graph"),
            ChartOption::from_json_str(
                r#"{
                    "title": {"text": "Graph"},
                    "series": [{
                        "type":"graph","layout":"force","label":{"show":true},
                        "force":{"repulsion":90,"edgeLength":55},
                        "data":[{"name":"A"},{"name":"B"},{"name":"C"},{"name":"D"}],
                        "links":[
                            {"source":"A","target":"B"},{"source":"A","target":"C"},
                            {"source":"B","target":"D"},{"source":"C","target":"D"}
                        ]
                    }]
                }"#,
            )
            .expect("valid graph option"),
        ),
        (
            String::from("Sankey"),
            ChartOption::from_json_str(
                r#"{
                    "title": {"text": "Sankey"},
                    "series": [{
                        "type":"sankey","top":"12%","data":[{"name":"Visit"},{"name":"Search"},{"name":"Cart"},{"name":"Order"}],
                        "links":[
                            {"source":"Visit","target":"Search","value":8},
                            {"source":"Visit","target":"Cart","value":3},
                            {"source":"Search","target":"Order","value":5},
                            {"source":"Cart","target":"Order","value":2}
                        ]
                    }]
                }"#,
            )
            .expect("valid sankey option"),
        ),
        (
            String::from("Treemap"),
            ChartOption::new().title("Treemap").push_series(Series::treemap(
                "Treemap",
                [
                    DataPoint::named("Alpha", 30.0),
                    DataPoint::named("Beta", 18.0),
                    DataPoint::named("Gamma", 12.0),
                    DataPoint::named("Delta", 9.0),
                ],
            )),
        ),
        (
            String::from("Map"),
            ChartOption::new()
                .title("Map")
                .push_series(Series::map("Map", map_features())),
        ),
    ]
}

fn map_features() -> Vec<MapFeature> {
    vec![
        MapFeature {
            name: String::from("West"),
            value: 12.0,
            polygons: vec![vec![(0.0, 0.0), (1.2, 0.1), (1.0, 1.1), (0.0, 1.0)]],
        },
        MapFeature {
            name: String::from("East"),
            value: 22.0,
            polygons: vec![vec![(1.1, 0.0), (2.2, 0.2), (2.0, 1.2), (1.0, 1.1)]],
        },
    ]
}
