use arkit::entry;
use arkit::prelude::*;
use arkit::{application, Element, Task};
use arkit_chart::{
    chart, Axis, ChartEvent, ChartOption, DataPoint, LinkData, MapFeature, NodeData, Series,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Demo {
    Typed,
    Json,
    Gallery,
    Tooltip,
}

#[derive(Debug, Clone)]
enum Message {
    Select(Demo),
    ChartSelected(ChartEvent),
}

#[derive(Debug, Clone)]
struct AppState {
    active: Demo,
    selected: Option<ChartEvent>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            active: Demo::Typed,
            selected: None,
        }
    }
}

fn update(state: &mut AppState, message: Message) -> Task<Message> {
    match message {
        Message::Select(active) => {
            state.active = active;
            state.selected = None;
        }
        Message::ChartSelected(event) => {
            state.selected = Some(event);
        }
    }
    Task::none()
}

fn view(state: &AppState) -> Element<Message> {
    Element::new(ChartExample {
        state: state.clone(),
    })
}

struct ChartExample {
    state: AppState,
}

impl arkit::advanced::Widget<Message, arkit::Theme, arkit::Renderer> for ChartExample {
    fn body(
        &self,
        _tree: &mut arkit::advanced::widget::Tree,
        _renderer: &arkit::Renderer,
    ) -> Element<Message> {
        column_component()
            .percent_width(1.0)
            .percent_height(1.0)
            .background_color(0xFFF8FAFC)
            .padding(14.0)
            .children(vec![
                header(self.state.active),
                status(self.state.selected.as_ref()),
                chart_panel(self.state.active),
            ])
            .into()
    }
}

fn header(active: Demo) -> Element<Message> {
    let tab = |label: &'static str, demo: Demo| {
        let selected = active == demo;
        button(label)
            .font_size(13.0)
            .font_color(if selected { 0xFFFFFFFF } else { 0xFF0F172A })
            .background_color(if selected { 0xFF2563EB } else { 0xFFFFFFFF })
            .border_radius(6.0)
            .padding([7.0, 10.0, 7.0, 10.0])
            .on_press(Message::Select(demo))
    };

    column_component()
        .percent_width(1.0)
        .children(vec![
            text("arkit_chart native drawing")
                .font_size(22.0)
                .font_weight(FontWeight::W700)
                .line_height(26.0)
                .font_color(0xFF0F172A)
                .into(),
            row_component()
                .margin_top(10.0)
                .children(vec![
                    tab("Typed", Demo::Typed).into(),
                    tab("JSON", Demo::Json).margin_left(8.0).into(),
                    tab("Gallery", Demo::Gallery).margin_left(8.0).into(),
                    tab("Tooltip", Demo::Tooltip).margin_left(8.0).into(),
                ])
                .into(),
        ])
        .into()
}

fn status(selected: Option<&ChartEvent>) -> Element<Message> {
    let label = selected
        .map(|event| {
            format!(
                "selected {}[{}] value {:?}",
                event.component_type, event.data_index, event.value
            )
        })
        .unwrap_or_else(|| String::from("tap a data point to test tooltip hit detection"));

    text(label)
        .margin_top(10.0)
        .font_size(12.0)
        .line_height(16.0)
        .font_color(0xFF475569)
        .into()
}

fn chart_panel(active: Demo) -> Element<Message> {
    let charts = match active {
        Demo::Typed => vec![typed_option()],
        Demo::Json => vec![json_option()],
        Demo::Gallery => gallery_options(),
        Demo::Tooltip => vec![tooltip_option()],
    };

    let children = charts
        .into_iter()
        .map(|option| {
            container(chart(option).on_select(Message::ChartSelected))
                .height(Length::Fill)
                .background_color(0xFFFFFFFF)
                .border_radius(8.0)
                .border_color(0xFFE2E8F0)
                .border_width(1.0)
                .margin_top(12.0)
                .into()
        })
        .collect();

    scroll(column_component().children(children)).into()
}

fn typed_option() -> ChartOption {
    ChartOption::new()
        .title("Typed line/bar/scatter")
        .x_axis(Axis::category(["Mon", "Tue", "Wed", "Thu", "Fri"]))
        .push_series(Series::line("Revenue", [120.0, 200.0, 150.0, 260.0, 310.0]))
        .push_series(Series::bar("Orders", [80.0, 130.0, 100.0, 170.0, 210.0]))
        .push_series(Series::scatter(
            "Signals",
            [
                DataPoint::values([0.0, 90.0]),
                DataPoint::values([1.0, 180.0]),
                DataPoint::values([3.0, 230.0]),
            ],
        ))
}

fn json_option() -> ChartOption {
    ChartOption::from_json_str(
        r##"{
            "title": {"text": "JSON option"},
            "xAxis": {"type": "category", "data": ["A", "B", "C", "D"]},
            "yAxis": {"type": "value"},
            "series": [
                {"type": "bar", "name": "bar", "data": [12, 20, 15, 28]},
                {"type": "line", "name": "line", "data": [8, 18, 22, 30]}
            ]
        }"##,
    )
    .expect("valid chart json")
}

fn tooltip_option() -> ChartOption {
    ChartOption::new()
        .title("Tooltip hit-test")
        .x_axis(Axis::category(["Q1", "Q2", "Q3", "Q4"]))
        .push_series(Series::bar("Actual", [32.0, 54.0, 48.0, 72.0]))
        .push_series(Series::line("Target", [40.0, 50.0, 60.0, 70.0]))
}

fn gallery_options() -> Vec<ChartOption> {
    vec![
        ChartOption::new().title("Pie").push_series(Series::pie(
            "Share",
            [
                DataPoint::named("A", 40.0),
                DataPoint::named("B", 24.0),
                DataPoint::named("C", 18.0),
            ],
        )),
        ChartOption::new()
            .title("Radar / Gauge / Funnel")
            .push_series(Series::radar("Radar", [42.0, 64.0, 35.0, 76.0, 58.0]))
            .push_series(Series::gauge("Gauge", 68.0))
            .push_series(Series::funnel(
                "Funnel",
                [
                    DataPoint::named("Visit", 100.0),
                    DataPoint::named("Lead", 60.0),
                    DataPoint::named("Deal", 26.0),
                ],
            )),
        ChartOption::new()
            .title("Heatmap / Candlestick")
            .push_series(Series::heatmap(
                "Heatmap",
                [
                    DataPoint::values([0.0, 0.0, 4.0]),
                    DataPoint::values([1.0, 0.0, 8.0]),
                    DataPoint::values([0.0, 1.0, 5.0]),
                    DataPoint::values([1.0, 1.0, 10.0]),
                ],
            ))
            .push_series(Series::candlestick(
                "K",
                [
                    DataPoint::values([20.0, 32.0, 18.0, 36.0]),
                    DataPoint::values([32.0, 28.0, 24.0, 35.0]),
                    DataPoint::values([28.0, 40.0, 26.0, 44.0]),
                ],
            )),
        ChartOption::new()
            .title("Tree / Graph / Sankey")
            .push_series(Series::tree(
                "Tree",
                nodes(),
                vec![
                    LinkData {
                        source: 0,
                        target: 1,
                        value: 1.0,
                    },
                    LinkData {
                        source: 0,
                        target: 2,
                        value: 1.0,
                    },
                ],
            ))
            .push_series(Series::graph(
                "Graph",
                nodes(),
                vec![
                    LinkData {
                        source: 0,
                        target: 1,
                        value: 1.0,
                    },
                    LinkData {
                        source: 1,
                        target: 2,
                        value: 1.0,
                    },
                    LinkData {
                        source: 2,
                        target: 0,
                        value: 1.0,
                    },
                ],
            ))
            .push_series(Series::sankey(
                "Sankey",
                nodes(),
                vec![
                    LinkData {
                        source: 0,
                        target: 2,
                        value: 2.0,
                    },
                    LinkData {
                        source: 1,
                        target: 2,
                        value: 1.0,
                    },
                ],
            )),
        ChartOption::new()
            .title("Treemap / Map")
            .push_series(Series::treemap(
                "Treemap",
                [
                    DataPoint::named("Alpha", 30.0),
                    DataPoint::named("Beta", 18.0),
                    DataPoint::named("Gamma", 12.0),
                ],
            ))
            .push_series(Series::map("Map", map_features())),
    ]
}

fn nodes() -> Vec<NodeData> {
    vec![
        NodeData {
            name: String::from("A"),
            value: 1.0,
        },
        NodeData {
            name: String::from("B"),
            value: 1.4,
        },
        NodeData {
            name: String::from("C"),
            value: 1.8,
        },
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

#[entry]
fn app() -> impl arkit::EntryPoint {
    application(AppState::default, update, view)
}
