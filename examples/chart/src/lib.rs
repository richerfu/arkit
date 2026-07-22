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
    let mut appended = use_signal(|| 0_u32);
    let mut selected = use_signal(|| String::from("Tap a chart item to inspect it"));
    let controller = use_hook(ChartController::new);
    let stream_controller = use_hook(ChartController::new);
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
    let highlight_controller = controller.clone();
    let select_controller = controller.clone();
    let downplay_controller = controller.clone();
    let append_controller = stream_controller.clone();
    let clear_controller = stream_controller.clone();
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
            width: "100%",
            height: "100%",
            background_color: "#FFF1F5F9",

            column {
                padding: 16.0,
                background_color: "#FFFFFFFF",
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
                    font_color: "#FF475569",
                    "Dioxus props + native ArkUI canvas; live tick #{tick}"
                }
                text {
                    margin_top: 6.0,
                    font_size: 13.0,
                    line_height: 18.0,
                    font_color: "#FF2563EB",
                    "{selected_label}"
                }
                row {
                    margin_top: 10.0,
                    button {
                        width: 160.0,
                        onclick: move |_| tick += 1,
                        "Update"
                    }
                    button {
                        margin_left: 8.0,
                        width: 160.0,
                        onclick: move |_| highlight_controller.dispatch_action(ChartAction::new(
                            ChartActionKind::Highlight(ChartActionTarget::item(1, 0)),
                        )),
                        "Highlight"
                    }
                }
                row {
                    margin_top: 6.0,
                    button {
                        width: 160.0,
                        onclick: move |_| select_controller.dispatch_actions([
                            ChartAction::new(ChartActionKind::ToggleSelect(
                                ChartActionTarget::item(1, 0),
                            )),
                            ChartAction::new(ChartActionKind::ToggleSelect(
                                ChartActionTarget::item(1, 1),
                            )),
                        ]),
                        "Select"
                    }
                    button {
                        margin_left: 8.0,
                        width: 160.0,
                        onclick: move |_| downplay_controller.dispatch_action(ChartAction::new(
                            ChartActionKind::Downplay(ChartActionTarget::item(1, 0)),
                        )),
                        "Downplay"
                    }
                }
                row {
                    margin_top: 6.0,
                    button {
                        width: 160.0,
                        onclick: move |_| {
                            appended += 1;
                            let index = appended();
                            let point = ChartCoordinatePoint::numbers(
                                5.0 + f64::from(index),
                                3.0 + f64::from((index * 3) % 7),
                            );
                            append_controller.append_data(ChartAppendData::scatter(
                                0,
                                [DataPoint::values([point.x.clone(), point.y.clone()])],
                            ));
                            let count = append_controller
                                .get_option()
                                .and_then(|option| match option.series.first() {
                                    Some(Series::Scatter(series)) => Some(series.data.len()),
                                    _ => None,
                                })
                                .unwrap_or_default();
                            let finder = ChartCoordinateFinder::series(0);
                            let pixel = append_controller
                                .convert_to_pixel(finder.clone(), point)
                                .unwrap_or_default();
                            let roundtrip = append_controller.convert_from_pixel(
                                finder.clone(),
                                pixel,
                            );
                            let inside = append_controller.contain_pixel(finder, pixel);
                            selected.set(format!(
                                "appendData points={count} pixel={pixel:?} inside={inside:?} back={roundtrip:?}",
                            ));
                        },
                        "Append data"
                    }
                    button {
                        margin_left: 8.0,
                        width: 160.0,
                        onclick: move |_| {
                            clear_controller.clear();
                            selected.set(format!(
                                "clear series={} size={:?}",
                                clear_controller
                                    .get_option()
                                    .map_or(0, |option| option.series.len()),
                                clear_controller.get_size()
                            ));
                        },
                        "Clear stream"
                    }
                }
            }

            column {
                width: "100%",
                layout_weight: 1.0,
                scroll {
                    width: "100%",
                    height: "100%",
                    scroll_bar: "on",
                    column {
                        width: "100%",
                        padding: 12.0,
                        DemoChart {
                            title: "Realtime line / bar / scatter",
                            option: realtime_option,
                            controller: Some(controller.clone()),
                            on_select: move |event: ChartEvent| {
                                selected.set(selection_label(&event));
                            },
                            on_event: move |event: ChartRuntimeEvent| {
                                selected.set(format!(
                                    "event {} / action {} / batch {} / selected {:?}",
                                    event.event_type,
                                    event.from_action.as_deref().unwrap_or("pointer"),
                                    event.batch.len(),
                                    event.selected
                                ));
                            },
                        }
                        DemoChart {
                            title: "Incremental appendData scatter",
                            option: stream_option(),
                            controller: Some(stream_controller.clone()),
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
}

#[derive(Props, Clone, PartialEq)]
struct DemoChartProps {
    title: String,
    option: ChartOption,
    on_select: EventHandler<ChartEvent>,
    #[props(default)]
    controller: Option<ChartController>,
    #[props(default)]
    on_event: Option<EventHandler<ChartRuntimeEvent>>,
}

#[component]
fn DemoChart(props: DemoChartProps) -> Element {
    rsx! {
        column {
            width: "100%",
            margin_bottom: 12.0,
            padding: 10.0,
            background_color: "#FFFFFFFF",
            border_width: 1.0,
            border_color: "#FFE2E8F0",
            border_radius: 10.0,
            text {
                margin_bottom: 6.0,
                font_size: 15.0,
                line_height: 20.0,
                font_weight: 600,
                font_color: "#FF0F172A",
                "{props.title}"
            }
            ECharts {
                option: props.option,
                height: "300",
                on_select: props.on_select,
                controller: props.controller,
                on_event: props.on_event,
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

fn stream_option() -> ChartOption {
    ChartOption::new()
        .title("appendData keeps existing points")
        .x_axis(Axis::value())
        .y_axis(Axis::value())
        .push_series(Series::scatter(
            "Stream",
            [
                DataPoint::values([1.0, 2.0]),
                DataPoint::values([3.0, 5.0]),
                DataPoint::values([5.0, 4.0]),
            ],
        ))
}

fn realtime_option(tick: u32) -> ChartOption {
    let wave = |offset: u32| {
        (0..8)
            .map(|index| 25.0 + ((index as u32 * 7 + tick * 5 + offset) % 36) as f64)
            .collect::<Vec<_>>()
    };
    let mut option = ChartOption::new()
        .title(format!("Realtime update #{tick}"))
        .legend(Legend {
            top: 34.into(),
            icon: String::from("roundRect"),
            formatter: Some(String::from("{name}")),
            selected: [(String::from("Alerts"), false)].into_iter().collect(),
            data_icons: [
                (String::from("Load"), String::from("line")),
                (String::from("Alerts"), String::from("circle")),
            ]
            .into_iter()
            .collect(),
            ..Legend::default()
        })
        .x_axis(Axis::category([
            "00:00", "03:00", "06:00", "09:00", "12:00", "15:00", "18:00", "21:00",
        ]))
        .push_series(Series::line("Load", wave(0)))
        // Keep the draggable labels stationary while the line continues to
        // exercise realtime setOption updates every second.
        .push_series(Series::bar(
            "Requests",
            [50.0, 57.0, 28.0, 35.0, 42.0, 49.0, 56.0, 27.0],
        ))
        .push_series(Series::scatter(
            "Alerts",
            [
                DataPoint::values([1.0, 48.0 + (tick % 8) as f64]),
                DataPoint::values([5.0, 35.0 + (tick % 12) as f64]),
            ],
        ));
    option.animation.initial.duration = 900;
    option.animation.update.duration = 800;
    option.animation.update.easing = String::from("cubicInOut");
    for series in &mut option.series {
        let options = match series {
            Series::Line(series) | Series::Bar(series) | Series::Scatter(series) => {
                &mut series.options
            }
            _ => continue,
        };
        options.emphasis.focus = Some(String::from("series"));
        options.emphasis.scale = Some(1.15);
        options.emphasis.item_style.color = Some(0xFFEE6666);
        options
            .emphasis
            .item_style
            .specified
            .insert(String::from("color"));
        options.blur.item_style.opacity = 0.22;
        options
            .blur
            .item_style
            .specified
            .insert(String::from("opacity"));
    }
    if let Series::Bar(series) = &mut option.series[1] {
        series.options.selected_mode = Some(String::from("multiple"));
        series.options.select.item_style.color = Some(0xFFEE6666);
        series
            .options
            .select
            .item_style
            .specified
            .insert(String::from("color"));
        series.options.label.show = true;
        series.options.label.formatter = Some(String::from("{c}"));
        series.options.label_layout.hide_overlap = true;
        series.options.label_layout.move_overlap = Some(String::from("shiftY"));
        series.options.label_layout.draggable = true;
        series.options.label_layout = std::mem::take(&mut series.options.label_layout)
            .with_callback(|params| LabelLayoutCallbackResult {
                dy: (params.data_index == Some(7)).then_some(-4.0),
                ..LabelLayoutCallbackResult::default()
            });
    }
    option
}

fn gallery_options() -> Vec<(String, ChartOption)> {
    vec![
        (
            String::from("Dataset filter + sort transform"),
            ChartOption::from_json_str(
                r##"{
                    "title":{"text":"2025 sales, descending"},
                    "dataset":[
                        {
                            "id":"raw",
                            "source":[
                                ["Product","Sales","Year"],
                                ["Cake",120,2024],
                                ["Tea",260,2025],
                                ["Tofu",180,2025],
                                ["Milk",310,2025],
                                ["Bread",220,2024]
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
                    "grid":{"left":48,"right":20,"top":58,"bottom":42},
                    "xAxis":{"type":"category"},
                    "yAxis":{"type":"value"},
                    "series":[{
                        "type":"bar","name":"Sales","datasetIndex":1,
                        "encode":{"x":"Product","y":"Sales"},
                        "itemStyle":{"color":"#5470c6","borderRadius":[5,5,0,0]},
                        "label":{"show":true,"position":"top"}
                    }]
                }"##,
            )
            .expect("valid transformed dataset option"),
        ),
        (
            String::from("Responsive baseOption + media"),
            ChartOption::from_json_str(
                r##"{
                    "baseOption":{
                        "title":{"text":"Base layout"},
                        "grid":{"left":42,"right":20,"top":58,"bottom":42},
                        "xAxis":{"type":"category","data":["Mon","Tue","Wed","Thu","Fri"]},
                        "yAxis":{"type":"value"},
                        "series":[{"type":"line","name":"Revenue","data":[12,20,15,28,24]}]
                    },
                    "media":[
                        {"query":{"maxWidth":400},"option":{"title":{"text":"Compact media → bar"},"series":[{"type":"bar","itemStyle":{"color":"#5470c6","borderRadius":[4,4,0,0]}}]}},
                        {"query":{"minAspectRatio":1.4},"option":{"grid":{"left":58,"right":36}}},
                        {"option":{"title":{"text":"Default media → line"},"series":[{"type":"line"}]}}
                    ]
                }"##,
            )
            .expect("valid responsive media option"),
        ),
        (
            String::from("Core timeline + brush interaction"),
            ChartOption::from_json_str(
                r##"{
                    "baseOption":{
                        "timeline":{"currentIndex":0,"autoPlay":false,"data":["Before","After"]},
                        "title":{"text":"Timeline nodes + native brush"},
                        "toolbox":{"show":true,"right":12,"top":10,"itemSize":18,"itemGap":10,"iconStyle":{"borderColor":"#5470c6","borderWidth":1.6},"feature":{"brush":{"type":["rect","clear"]},"restore":{}}},
                        "brush":{"brushType":"rect","brushMode":"single","brushStyle":{"color":"rgba(84,112,198,0.18)","borderColor":"#5470c6"},"inBrush":{"color":"#ee6666"}},
                        "grid":{"left":42,"right":24,"top":58,"bottom":76},
                        "xAxis":{"type":"value","min":0,"max":10},
                        "yAxis":{"type":"value","min":0,"max":10},
                        "series":[{"type":"scatter","name":"Samples","symbolSize":14,"data":[[1,2],[2,7],[4,4],[6,8],[8,3],[9,7]]}]
                    },
                    "options":[
                        {"series":[{"type":"scatter","name":"Samples","symbolSize":14,"data":[[1,2],[2,7],[4,4],[6,8],[8,3],[9,7]]}]},
                        {"series":[{"type":"scatter","name":"Samples","symbolSize":14,"data":[[1,5],[3,8],[4,2],[6,6],[7,3],[9,9]]}]}
                    ]
                }"##,
            )
            .expect("valid integrated timeline brush option"),
        ),
        (
            String::from("Toolbox dataZoom + magicType"),
            ChartOption::from_json_str(
                r##"{
                    "title":{"text":"Native toolbox actions"},
                    "toolbox":{"show":true,"left":18,"top":34,"itemSize":16,"itemGap":5,"padding":5,"iconStyle":{"borderColor":"#5470c6","borderWidth":1.5},"feature":{"dataZoom":{"xAxisIndex":[0],"yAxisIndex":[0]},"magicType":{"type":["line","bar","stack"]},"dataView":{"readOnly":true},"saveAsImage":{"type":"png","name":"arkit-chart"},"restore":{}}},
                    "grid":{"left":42,"right":20,"top":68,"bottom":42},
                    "xAxis":{"type":"category","data":["Mon","Tue","Wed","Thu","Fri","Sat"]},
                    "yAxis":{"type":"value"},
                    "series":[
                        {"type":"line","name":"Visits","smooth":true,"data":[12,20,15,28,24,31]},
                        {"type":"line","name":"Orders","data":[8,14,11,19,17,23]}
                    ]
                }"##,
            )
            .expect("valid toolbox action option"),
        ),
        (
            String::from("ECharts Cartesian axes"),
            ChartOption::from_json_str(
                r##"{
                    "title": {"text": "Dual axes / label formatter"},
                    "grid": {"left": 52, "right": 52, "top": 60, "bottom": 58},
                    "xAxis": {
                        "type": "category",
                        "data": ["January", "February", "March", "April", "May", "June"],
                        "axisTick": {"alignWithLabel": true},
                        "axisLabel": {"rotate": 22, "interval": 0, "color": "#475569"}
                    },
                    "yAxis": [
                        {"type": "value", "name": "Orders", "axisLabel": {"formatter": "{value} pcs"}},
                        {"type": "value", "name": "Revenue", "position": "right", "axisLine": {"onZero": false, "lineStyle": {"color": "#f97316"}}, "axisLabel": {"formatter": "¥{value}", "color": "#c2410c"}}
                    ],
                    "color": ["#2563eb", "#f97316"],
                    "series": [
                        {"type": "bar", "name": "Orders", "data": [12, 20, 15, 28, 24, 31]},
                        {"type": "line", "name": "Revenue", "yAxisIndex": 1, "smooth": true, "data": [8, 18, 22, 30, 36, 42]}
                    ]
                }"##,
            )
            .expect("valid chart option"),
        ),
        (
            String::from("Horizontal bar layout parity"),
            ChartOption::from_json_str(
                r##"{
                    "title":{"text":"Horizontal bar / background / radius"},
                    "grid":{"left":72,"right":48,"top":54,"bottom":38},
                    "xAxis":{"type":"value"},
                    "yAxis":{"type":"category","data":["Search","Direct","Email","Affiliate"]},
                    "series":[{
                        "type":"bar","name":"Visits","data":[2,8,5,11],
                        "barWidth":"42%","barMinWidth":6,"barMaxWidth":"55%","barMinHeight":6,
                        "barGap":"10%","barCategoryGap":"30%","showBackground":true,
                        "backgroundStyle":{"color":"rgba(148,163,184,0.18)","borderRadius":6},
                        "itemStyle":{"color":"#5470c6","borderRadius":[0,6,6,0]},
                        "label":{"show":true,"position":"right","distance":6}
                    }]
                }"##,
            )
            .expect("valid horizontal bar option"),
        ),
        (
            String::from("Scatter symbol / visualMap parity"),
            ChartOption::from_json_str(
                r##"{
                    "title":{"text":"Bubble size / data symbol / ripple"},
                    "grid":{"left":48,"right":28,"top":58,"bottom":42},
                    "xAxis":{"type":"value","min":0,"max":10},
                    "yAxis":{"type":"value","min":0,"max":10},
                    "visualMap":{"show":false,"min":10,"max":50,"dimension":2,
                        "inRange":{"color":["#91cc75","#ee6666"],"symbolSize":[10,34]}},
                    "series":[
                        {"type":"scatter","name":"Bubble","symbol":"roundRect","symbolRotate":18,
                         "label":{"show":true,"position":"top","formatter":"{c}"},
                         "data":[[2,3,10],[4,7,22],{"value":[6,4,36],"symbol":"diamond","symbolSize":[30,18],"symbolRotate":35},[8,8,50]]},
                        {"type":"effectScatter","name":"Pulse","symbol":"circle","symbolSize":[18,12],
                         "rippleEffect":{"period":2,"number":3,"scale":2.8,"brushType":"stroke","color":"#5470c6"},
                         "itemStyle":{"color":"#5470c6"},"data":[[7,2,28]]}
                    ]
                }"##,
            )
            .expect("valid scatter parity option"),
        ),
        (
            String::from("Pie angle / label / selection parity"),
            ChartOption::from_json_str(
                r##"{
                    "title":{"text":"Partial doughnut / percent / select"},
                    "tooltip":{"trigger":"item","formatter":"{b}: {c} ({d}%)"},
                    "series":[{
                        "type":"pie","name":"Traffic","center":["50%","52%"],"radius":["28%","58%"],
                        "startAngle":90,"endAngle":-210,"clockwise":true,"padAngle":2,"minAngle":10,
                        "minShowLabelAngle":5,"percentPrecision":1,"selectedMode":"multiple","selectedOffset":12,
                        "avoidLabelOverlap":true,
                        "itemStyle":{"borderColor":"#ffffff","borderWidth":2},
                        "label":{"show":true,"position":"outside","formatter":"{b} {d}%","distance":4},
                        "labelLine":{"show":true,"length":10,"length2":12,"lineStyle":{"width":1,"type":"solid"}},
                        "data":[
                            {"name":"Search","value":42},
                            {"name":"Direct","value":25,"selected":true},
                            {"name":"Email","value":17},
                            {"name":"Social","value":9},
                            {"name":"Other","value":2}
                        ]
                    }]
                }"##,
            )
            .expect("valid pie parity option"),
        ),
        (
            String::from("Missing data / connectNulls"),
            ChartOption::from_json_str(
                r##"{
                    "title":{"text":"Null stays missing"},
                    "legend":{"top":34},
                    "grid":{"left":46,"right":24,"top":76,"bottom":48},
                    "xAxis":{"type":"category","data":["Mon","Tue","Wed","Thu","Fri","Sat","Sun"],"axisTick":{"alignWithLabel":true}},
                    "yAxis":{"type":"value"},
                    "series":[
                        {"type":"line","name":"Gapped","showSymbol":true,"data":[12,null,19,15,"-",24,21]},
                        {"type":"line","name":"Connected","connectNulls":true,"lineStyle":{"color":"#f97316"},"itemStyle":{"color":"#f97316"},"data":[8,null,14,12,"-",18,17]}
                    ]
                }"##,
            )
            .expect("valid missing data option"),
        ),
        (
            String::from("Line step / symbol / endLabel"),
            ChartOption::from_json_str(
                r##"{
                    "title":{"text":"Line option parity"},
                    "grid":{"left":42,"right":62,"top":55,"bottom":45},
                    "xAxis":{"type":"category","data":["A","B","C","D","E"]},
                    "yAxis":{"type":"value"},
                    "series":[
                        {"type":"line","name":"Start","step":"start","symbol":"emptyCircle","symbolSize":8,
                         "lineStyle":{"type":"dashed"},"endLabel":{"show":true,"formatter":"start {c}"},"data":[2,5,3,7,4]},
                        {"type":"line","name":"Middle","step":"middle","symbol":"diamond","symbolSize":9,"symbolRotate":15,
                         "label":{"show":true,"position":"top","fontSize":9},"data":[4,2,6,4,8]},
                        {"type":"line","name":"Smooth","smooth":0.5,"smoothMonotone":"x","symbol":"triangle","symbolSize":8,
                         "areaStyle":{"opacity":0.12,"origin":"start"},"data":[1,4,2,5,3]}
                    ]
                }"##,
            )
            .expect("valid line parity option"),
        ),
        (
            String::from("Line polar coordinate"),
            ChartOption::from_json_str(
                r##"{
                    "title":{"text":"Polar line"},
                    "polar":{"center":["50%","55%"],"radius":"62%"},
                    "angleAxis":{"type":"category","startAngle":90,"clockwise":true,"data":["A","B","C","D","E","F"]},
                    "radiusAxis":{"type":"value","min":0,"max":10,"splitNumber":5},
                    "series":[{
                        "type":"line","coordinateSystem":"polar","name":"Score","smooth":0.35,
                        "symbol":"diamond","symbolSize":8,"areaStyle":{"opacity":0.18},
                        "label":{"show":true,"fontSize":9},"data":[3,7,5,9,6,4]
                    }]
                }"##,
            )
            .expect("valid polar line option"),
        ),
        (
            String::from("Polar bar / scatter / effectScatter"),
            ChartOption::from_json_str(
                r##"{
                    "title":{"text":"Polar series parity"},
                    "polar":{"center":["50%","54%"],"radius":["8%","68%"]},
                    "angleAxis":{"type":"category","data":["Mon","Tue","Wed","Thu","Fri"],"startAngle":90},
                    "radiusAxis":{"min":0,"max":100,"splitNumber":4},
                    "series":[
                        {"type":"bar","coordinateSystem":"polar","name":"Load","barCategoryGap":"35%","itemStyle":{"color":"#91cc75"},"data":[[35,0],[58,1],[42,2],[76,3],[64,4]]},
                        {"type":"scatter","coordinateSystem":"polar","name":"Target","symbol":"diamond","symbolSize":[12,9],"itemStyle":{"color":"#ee6666"},"data":[[72,0],[68,2],[88,4]]},
                        {"type":"effectScatter","coordinateSystem":"polar","name":"Live","symbolSize":10,"rippleEffect":{"period":2,"number":3,"scale":2.4,"brushType":"stroke"},"data":[[52,1],[82,3]]}
                    ]
                }"##,
            )
            .expect("valid polar series parity option"),
        ),
        (
            String::from("Single axis scatter"),
            ChartOption::from_json_str(
                r##"{
                    "title":{"text":"SingleAxis coordinate"},
                    "graphic":{"type":"group","left":"68%","top":"20%","children":[
                        {"type":"rect","shape":{"x":0,"y":0,"width":70,"height":24,"r":6},"style":{"fill":"#eff6ff","stroke":"#60a5fa","lineWidth":1}},
                        {"type":"text","x":8,"y":3,"style":{"text":"native","fill":"#1d4ed8","fontSize":11}}
                    ]},
                    "singleAxis":{"left":"12%","top":"55%","width":"76%","type":"value","min":0,"max":100,"splitNumber":5},
                    "visualMap":{"show":false,"min":0,"max":100,"inRange":{"color":["#91cc75","#ee6666"],"symbolSize":[8,22]}},
                    "series":[
                        {"type":"scatter","coordinateSystem":"singleAxis","name":"Events","data":[12,28,46,63,82]},
                        {"type":"effectScatter","coordinateSystem":"singleAxis","name":"Live","symbol":"diamond","rippleEffect":{"period":2,"number":3,"brushType":"stroke"},"data":[72]}
                    ]
                }"##,
            )
            .expect("valid single axis scatter option"),
        ),
        (
            String::from("Geo coordinate overlays"),
            ChartOption::from_json_str(
                r##"{
                    "title":{"text":"geo scatter / heatmap / lines / graph"},
                    "geo":{"left":"10%","right":"10%","top":52,"bottom":28,"aspectScale":1,
                        "geoJson":{"type":"FeatureCollection","features":[
                            {"type":"Feature","properties":{"name":"West"},"geometry":{"type":"Polygon","coordinates":[[[0,0],[5,0],[5,10],[0,10],[0,0]]]}},
                            {"type":"Feature","properties":{"name":"East"},"geometry":{"type":"Polygon","coordinates":[[[5,0],[10,0],[10,10],[5,10],[5,0]]]}}
                        ]}},
                    "visualMap":{"show":false,"min":0,"max":50,"dimension":2,"inRange":{"color":["#91cc75","#ee6666"],"symbolSize":[10,20]}},
                    "series":[
                        {"type":"scatter","coordinateSystem":"geo","name":"Sites","data":[[2,7,20],[7,6,42]]},
                        {"type":"effectScatter","coordinateSystem":"geo","name":"Live","data":[[8,8,48]],"rippleEffect":{"period":2,"number":3}},
                        {"type":"heatmap","coordinateSystem":"geo","name":"Density","symbolSize":[16,16],"data":[[3,3,12],[7,3,38]]},
                        {"type":"lines","coordinateSystem":"geo","name":"Route","symbol":["none","arrow"],"lineStyle":{"color":"#5470c6","type":"dashed"},"effect":{"show":true,"period":3,"symbol":"diamond"},"data":[{"coords":[[1,1],[5,8],[9,2]]}]},
                        {"type":"graph","coordinateSystem":"geo","name":"Network","symbolSize":10,"data":[{"name":"A","x":2,"y":5},{"name":"B","x":8,"y":5}],"links":[{"source":"A","target":"B"}]}
                    ]
                }"##,
            )
            .expect("valid geo overlay option"),
        ),
        (
            String::from("dataZoom / axisPointer / markers"),
            ChartOption::from_json_str(
                r##"{
                    "title":{"text":"Interactive Cartesian"},
                    "toolbox":{"show":true,"right":12,"top":10,"itemSize":18,"feature":{"restore":{}}},
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
            String::from("Timeline baseOption / options"),
            ChartOption::from_json_str(
                r##"{
                    "baseOption":{
                        "timeline":{"currentIndex":1,"autoPlay":true,"playInterval":1800,"loop":true,"data":["2023","2024","2025"]},
                        "title":{"text":"Timeline snapshots"},
                        "grid":{"left":44,"right":20,"top":58,"bottom":72},
                        "xAxis":{"type":"category","data":["Hardware","Cloud","Service","Other"]},
                        "yAxis":{"type":"value"},
                        "series":[{"type":"bar","name":"Revenue","label":{"show":true,"position":"top"},"data":[18,25,16,9]}]
                    },
                    "options":[
                        {"series":[{"type":"bar","name":"Revenue","label":{"show":true,"position":"top"},"data":[18,25,16,9]}]},
                        {"series":[{"type":"bar","name":"Revenue","label":{"show":true,"position":"top"},"data":[24,31,22,14]}]},
                        {"series":[{"type":"bar","name":"Revenue","label":{"show":true,"position":"top"},"data":[32,38,29,19]}]}
                    ]
                }"##,
            )
            .expect("valid timeline option"),
        ),
        (
            String::from("Brush selection / toolbox"),
            ChartOption::from_json_str(
                r##"{
                    "title":{"text":"Drag to brush points"},
                    "toolbox":{"show":true,"orient":"vertical","right":10,"top":48,"itemSize":18,"itemGap":10,"padding":6,"backgroundColor":"rgba(255,255,255,0.82)","borderColor":"#e2e8f0","borderWidth":1,"borderRadius":5,"feature":{"brush":{"type":["rect","clear"]},"restore":{}}},
                    "brush":{"brushType":"rect","brushMode":"multiple","brushStyle":{"color":"rgba(84,112,198,0.18)","borderColor":"#5470c6","borderWidth":1},"inBrush":{"color":"#ee6666"},"outOfBrush":{"opacity":0.25}},
                    "grid":{"left":42,"right":22,"top":56,"bottom":42},
                    "xAxis":{"type":"value","min":0,"max":10},
                    "yAxis":{"type":"value","min":0,"max":10},
                    "series":[{"type":"scatter","name":"Samples","symbolSize":14,"data":[[1,2],[2,7],[3,4],[4,8],[5,3],[6,6],[7,2],[8,7],[9,5]]}]
                }"##,
            )
            .expect("valid brush option"),
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
                        "splitArea":{"show":true,"areaStyle":{"color":["#f8fafc","#eef2ff"]}},
                        "splitLine":{"lineStyle":{"color":"#cbd5e1","width":1}},
                        "axisLine":{"lineStyle":{"color":"#94a3b8"}},
                        "axisName":{"color":"#475569","fontSize":10},
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
                    "title": {"text": "Gauge parity"},
                    "series": [{
                        "type": "gauge",
                        "name": "Temperature",
                        "min": -20,
                        "max": 120,
                        "startAngle": 210,
                        "endAngle": -30,
                        "radius": "68%",
                        "progress": {"show": true, "width": 12, "roundCap": true},
                        "axisLine": {
                            "roundCap": true,
                            "lineStyle": {
                                "width": 14,
                                "color": [[0.3,"#67e0e3"],[0.7,"#37a2da"],[1,"#fd666d"]]
                            }
                        },
                        "axisTick": {
                            "splitNumber": 4,
                            "distance": 10,
                            "length": 5,
                            "lineStyle": {"color": "#64748b", "width": 1}
                        },
                        "splitLine": {
                            "distance": 10,
                            "length": 12,
                            "lineStyle": {"color": "#334155", "width": 2}
                        },
                        "axisLabel": {
                            "distance": 12,
                            "color": "#64748b",
                            "fontSize": 9,
                            "formatter": "{value}°"
                        },
                        "pointer": {
                            "length": "62%",
                            "width": 8,
                            "offsetCenter": [0, "-3%"],
                            "itemStyle": {"color": "#5470c6"}
                        },
                        "anchor": {
                            "show": true,
                            "size": 12,
                            "itemStyle": {"color": "#ffffff", "borderColor": "#5470c6", "borderWidth": 3}
                        },
                        "title": {"offsetCenter": [0, "62%"], "fontSize": 11, "color": "#64748b"},
                        "detail": {"offsetCenter": [0, "38%"], "formatter": "{value}°C", "fontSize": 20, "fontWeight": "bold", "color": "#0f172a"},
                        "data": [{"name": "Room", "value": 68, "itemStyle": {"color": "#5470c6"}}]
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
            String::from("Calendar heatmap"),
            ChartOption::from_json_str(
                r##"{
                    "title":{"text":"Calendar coordinate"},
                    "calendar":{"top":"20%","left":"13%","width":"80%","height":"62%","range":["2026-01-01","2026-03-31"],"itemStyle":{"color":"#f8fafc"},"splitLine":{"lineStyle":{"color":"#cbd5e1"}}},
                    "visualMap":{"show":false,"pieces":[
                        {"max":3,"color":"#dbeafe"},{"min":4,"max":7,"color":"#60a5fa"},{"min":8,"color":"#1d4ed8"}
                    ]},
                    "series":[{"type":"heatmap","coordinateSystem":"calendar","data":[
                        ["2026-01-03",2],["2026-01-12",5],["2026-01-26",9],
                        ["2026-02-08",7],["2026-02-21",3],["2026-03-05",8],["2026-03-22",6]
                    ]}]
                }"##,
            )
            .expect("valid calendar heatmap option"),
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
                        {"type":"pictorialBar","name":"Count","barWidth":16,"symbol":"diamond","symbolRepeat":true,"symbolMargin":2,"symbolClip":true,"symbolBoundingData":42,"symbolSize":[14,8],"itemStyle":{"color":"#91cc75"},"data":[10,14,8,12]}
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
                    "series":[{"type":"lines","symbol":["none","arrow"],"symbolSize":9,"lineStyle":{"color":"#5470c6","width":2,"type":"dashed"},"effect":{"show":true,"period":2.5,"symbol":"diamond","symbolSize":7,"color":"#ee6666"},"data":[
                        {"name":"A → B","coords":[[0,0],[0.8,1.7],[2,1]],"value":1},
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
                r##"{
                    "title": {"text": "Tree"},
                    "series": [{"type":"tree","orient":"LR","top":"10%","data":[{
                        "name":"Root","children":[
                            {"name":"A","children":[{"name":"A1"},{"name":"A2"}]},
                            {"name":"B","children":[{"name":"B1"}]}
                        ]
                    }]}]
                }"##,
            )
            .expect("valid tree option"),
        ),
        (
            String::from("Graph"),
            ChartOption::from_json_str(
                r##"{
                    "title": {"text": "Graph"},
                    "series": [{
                        "type":"graph","layout":"force","label":{"show":true},
                        "force":{"repulsion":90,"edgeLength":55},
                        "data":[
                            {"name":"A","symbol":"diamond","symbolSize":18,"itemStyle":{"color":"#ee6666"},"label":{"show":true,"color":"#7f1d1d"}},
                            {"name":"B","symbol":"roundRect","symbolSize":16,"itemStyle":{"color":"#91cc75"}},
                            {"name":"C","symbol":"triangle","symbolSize":18,"symbolRotate":20,"itemStyle":{"color":"#fac858"}},
                            {"name":"D","symbol":"circle","symbolSize":15,"itemStyle":{"color":"#73c0de"}}
                        ],
                        "links":[
                            {"source":"A","target":"B"},{"source":"A","target":"C"},
                            {"source":"B","target":"D"},{"source":"C","target":"D"}
                        ]
                    }]
                }"##,
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
            ChartOption::from_json_str(
                r##"{
                    "title":{"text":"Hierarchical treemap"},
                    "series":[{"type":"treemap","visibleMin":6,"childrenVisibleMin":20,"label":{"show":true,"formatter":"{b}"},"data":[
                        {"name":"Platform","value":48,"itemStyle":{"color":"#5470c6"},"children":[
                            {"name":"Mobile","value":28,"itemStyle":{"color":"#91cc75"}},
                            {"name":"Desktop","value":20,"itemStyle":{"color":"#73c0de"}}
                        ]},
                        {"name":"Services","value":32,"itemStyle":{"color":"#fac858"},"children":[
                            {"name":"Search","value":18,"itemStyle":{"color":"#ee6666"}},
                            {"name":"Cloud","value":14,"itemStyle":{"color":"#9a60b4"}}
                        ]}
                    ]}]
                }"##,
            )
            .expect("valid hierarchical treemap option"),
        ),
        (
            String::from("Map GeoJSON / holes / no-data / select"),
            ChartOption::from_json_str(
                r##"{
                    "title":{"text":"Map option parity"},
                    "visualMap":{"show":false,"min":0,"max":30,"pieces":[
                        {"max":5,"label":"Low","color":"#dbeafe"},
                        {"min":6,"max":15,"label":"Medium","color":"#60a5fa"},
                        {"min":16,"label":"High","color":"#1d4ed8"}
                    ]},
                    "series":[{
                        "type":"map","name":"Regions","nameProperty":"code","nameMap":{"west":"West","east":"East"},
                        "selectedMode":"multiple","roam":"move","layoutCenter":["50%","54%"],"layoutSize":"72%",
                        "label":{"show":true,"formatter":"{b} {c}"},
                        "itemStyle":{"borderColor":"#334155","borderWidth":1},
                        "select":{"itemStyle":{"color":"#f59e0b"},"label":{"show":true,"color":"#111827"}},
                        "geoJson":{"type":"FeatureCollection","features":[
                            {"type":"Feature","properties":{"code":"west","cp":[4,5]},"geometry":{"type":"Polygon","coordinates":[
                                [[0,0],[8,0],[8,10],[0,10],[0,0]],
                                [[2.5,3],[5.5,3],[5.5,7],[2.5,7],[2.5,3]]
                            ]}},
                            {"type":"Feature","properties":{"code":"east"},"geometry":{"type":"MultiPolygon","coordinates":[
                                [[[9,0],[17,1],[16,9],[9,10],[9,0]]],
                                [[[18,7],[20,7],[20,9],[18,7]]]
                            ]}}
                        ]},
                        "data":[{"name":"west","value":12,"selected":true}]
                    }]
                }"##,
            )
            .expect("valid map parity option"),
        ),
    ]
}
