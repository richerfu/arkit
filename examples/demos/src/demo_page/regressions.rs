//! Interactive acceptance probes for renderer and typed-chart regressions.
//!
//! Labels intentionally use stable ASCII prefixes so an emulator smoke test
//! can locate controls and assert the expected state without relying on
//! coordinates or localized prose.

use std::collections::BTreeMap;

use arkit::echarts::{Axis, ChartController, ChartOption, DataValue, Dataset, ECharts, Series};
use arkit::prelude::*;

const PAGE_BACKGROUND: &str = "#FFF1F5F9";
const CARD_BACKGROUND: &str = "#FFFFFFFF";
const BORDER_COLOR: &str = "#FFE2E8F0";
const TITLE_COLOR: &str = "#FF0F172A";
const MUTED_COLOR: &str = "#FF475569";
const STATUS_COLOR: &str = "#FF0369A1";

#[derive(Clone, Copy)]
pub(crate) struct RegressionRootState {
    pub active: Signal<bool>,
    pub marker: Signal<bool>,
}

#[component]
pub(super) fn RegressionPage() -> Element {
    let mut keyed_step = use_signal(|| 0_u8);
    let mut keyed_actions = use_signal(|| 0_u32);
    let mut merged_tail = use_signal(|| true);
    let mut text_actions = use_signal(|| 0_u32);
    let root_state = use_context::<RegressionRootState>();
    let mut root_active = root_state.active;
    let mut root_marker = root_state.marker;
    let mut portal_actions = use_signal(|| 0_u32);
    let mut dataset_version = use_signal(|| 0_u8);
    let mut chart_actions = use_signal(|| 0_u32);
    let mut chart_status = use_signal(|| String::from("CHART result=PRESS_INSPECT"));
    let typed_controller = use_hook(ChartController::new);
    let json_controller = use_hook(ChartController::new);

    let step = keyed_step();
    let text_expected = if merged_tail() { "AB" } else { "A" };
    let dataset = dataset_version();
    let typed_option = typed_dataset_option(dataset);
    let json_option = json_dataset_option(dataset);
    let inspect_typed = typed_controller.clone();
    let inspect_json = json_controller.clone();

    use_effect(move || {
        root_marker.set(false);
        root_active.set(true);
    });
    use_drop(move || {
        root_marker.set(false);
        root_active.set(false);
    });

    rsx! {
        RegressionBody {
            keyed_step: step,
            keyed_actions: keyed_actions(),
            on_keyed_next: move |_| {
                keyed_step.set((keyed_step() + 1) % 6);
                keyed_actions += 1;
            },
            on_keyed_reset: move |_| {
                keyed_step.set(0);
                keyed_actions += 1;
            },
            merged_tail: merged_tail(),
            text_expected,
            text_actions: text_actions(),
            on_text_toggle: move |_| {
                merged_tail.toggle();
                text_actions += 1;
            },
            root_marker: root_marker(),
            portal_actions: portal_actions(),
            on_portal_toggle: move |_| {
                root_marker.toggle();
                portal_actions += 1;
            },
            dataset_version: dataset,
            chart_actions: chart_actions(),
            chart_status: chart_status(),
            typed_option,
            json_option,
            typed_controller: typed_controller.clone(),
            json_controller: json_controller.clone(),
            on_chart_update: move |_| {
                dataset_version.set((dataset_version() + 1) % 2);
                chart_actions += 1;
                chart_status.set(String::from("CHART result=PENDING_INSPECT"));
            },
            on_chart_inspect: move |_| {
                let typed = inspect_typed
                    .get_option()
                    .map(|snapshot| chart_signature(snapshot.option()));
                let json = inspect_json
                    .get_option()
                    .map(|snapshot| chart_signature(snapshot.option()));
                let expected = expected_chart_signature(dataset_version());
                let result = match (&typed, &json) {
                    (Ok(typed), Ok(json)) if typed == json && typed == expected => "PASS",
                    (Ok(_), Ok(_)) => "FAIL",
                    _ => "NOT_BOUND",
                };
                chart_status.set(format!(
                    "CHART result={result} expected={expected} typed={} json={}",
                    typed.as_deref().unwrap_or("none"),
                    json.as_deref().unwrap_or("none"),
                ));
            },
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct RegressionBodyProps {
    keyed_step: u8,
    keyed_actions: u32,
    on_keyed_next: EventHandler<()>,
    on_keyed_reset: EventHandler<()>,
    merged_tail: bool,
    text_expected: &'static str,
    text_actions: u32,
    on_text_toggle: EventHandler<()>,
    root_marker: bool,
    portal_actions: u32,
    on_portal_toggle: EventHandler<()>,
    dataset_version: u8,
    chart_actions: u32,
    chart_status: String,
    typed_option: ChartOption,
    json_option: ChartOption,
    typed_controller: ChartController,
    json_controller: ChartController,
    on_chart_update: EventHandler<()>,
    on_chart_inspect: EventHandler<()>,
}

#[component]
fn RegressionBody(props: RegressionBodyProps) -> Element {
    let order = keyed_order(props.keyed_step);
    let expected = keyed_expected(props.keyed_step);
    let replacement_column = props.keyed_step == 5;
    // Two separate dynamic text nodes are required here. Combining these into
    // one formatted string would not exercise merged native TextContent.
    let merged_chunks = if props.merged_tail {
        vec!["A", "B"]
    } else {
        vec!["A"]
    };
    let replacement_key = "replace-slot";
    let root_state = if props.root_marker { "ON" } else { "OFF" };

    rsx! {
        column {
            width: "100%",
            height: "100%",
            background_color: PAGE_BACKGROUND,

            column {
                width: "100%",
                padding_top: 24.0,
                padding_right: 14.0,
                padding_bottom: 10.0,
                padding_left: 14.0,
                background_color: CARD_BACKGROUND,
                text {
                    font_size: 24.0,
                    line_height: 30.0,
                    font_weight: 700,
                    font_color: TITLE_COLOR,
                    "Refactor regression"
                }
                text {
                    margin_top: 3.0,
                    font_size: 12.0,
                    line_height: 17.0,
                    font_color: MUTED_COLOR,
                    "Deterministic renderer + typed dataset acceptance probes"
                }
            }

            scroll {
                width: "100%",
                layout_weight: 1.0,
                scroll_bar: "on",
                column {
                    width: "100%",
                    padding: 12.0,

                    column {
                        width: "100%",
                        padding: 12.0,
                        background_color: CARD_BACKGROUND,
                        border_width: 1.0,
                        border_color: BORDER_COLOR,
                        border_radius: 10.0,
                        text {
                            font_size: 16.0,
                            font_weight: 700,
                            font_color: TITLE_COLOR,
                            "1. Keyed move / remove / replace"
                        }
                        text {
                            margin_top: 4.0,
                            font_size: 12.0,
                            font_color: STATUS_COLOR,
                            "KEYED step={props.keyed_step} expected={expected} actions={props.keyed_actions}"
                        }
                        row {
                            width: "100%",
                            height: 44.0,
                            margin_top: 8.0,
                            for label in order {
                                row {
                                    key: "{label}",
                                    width: 54.0,
                                    height: 36.0,
                                    margin_right: 7.0,
                                    align_items: "center",
                                    justify_content: "center",
                                    background_color: keyed_color(label),
                                    border_radius: 8.0,
                                    text {
                                        font_size: 15.0,
                                        font_weight: 700,
                                        font_color: "#FFFFFFFF",
                                        "{label}"
                                    }
                                }
                            }
                        }
                        if replacement_column {
                            column {
                                key: "{replacement_key}",
                                width: "100%",
                                height: 26.0,
                                align_items: "center",
                                justify_content: "center",
                                background_color: "#FFDCFCE7",
                                text { font_size: 11.0, font_color: "#FF166534", "REPLACE node=COLUMN" }
                            }
                        } else {
                            row {
                                key: "{replacement_key}",
                                width: "100%",
                                height: 26.0,
                                align_items: "center",
                                justify_content: "center",
                                background_color: "#FFFFEDD5",
                                text { font_size: 11.0, font_color: "#FF9A3412", "REPLACE node=ROW" }
                            }
                        }
                        row {
                            width: "100%",
                            margin_top: 8.0,
                            button {
                                width: 128.0,
                                height: 38.0,
                                font_size: 13.0,
                                onclick: move |_| props.on_keyed_next.call(()),
                                "KEYED NEXT"
                            }
                            button {
                                width: 100.0,
                                height: 38.0,
                                margin_left: 8.0,
                                font_size: 13.0,
                                onclick: move |_| props.on_keyed_reset.call(()),
                                "KEYED RESET"
                            }
                        }
                    }

                    column {
                        width: "100%",
                        margin_top: 10.0,
                        padding: 12.0,
                        background_color: CARD_BACKGROUND,
                        border_width: 1.0,
                        border_color: BORDER_COLOR,
                        border_radius: 10.0,
                        text {
                            font_size: 16.0,
                            font_weight: 700,
                            font_color: TITLE_COLOR,
                            "2. Merged text removal"
                        }
                        text {
                            margin_top: 4.0,
                            font_size: 12.0,
                            font_color: STATUS_COLOR,
                            "TEXT expected={props.text_expected} actions={props.text_actions}"
                        }
                        text {
                            width: "100%",
                            height: 38.0,
                            margin_top: 7.0,
                            font_size: 24.0,
                            font_weight: 700,
                            font_color: TITLE_COLOR,
                            for chunk in merged_chunks {
                                "{chunk}"
                            }
                        }
                        button {
                            width: 150.0,
                            height: 38.0,
                            font_size: 13.0,
                            onclick: move |_| props.on_text_toggle.call(()),
                            "TEXT TOGGLE B"
                        }
                    }

                    column {
                        width: "100%",
                        margin_top: 10.0,
                        padding: 12.0,
                        background_color: CARD_BACKGROUND,
                        border_width: 1.0,
                        border_color: BORDER_COLOR,
                        border_radius: 10.0,
                        text {
                            font_size: 16.0,
                            font_weight: 700,
                            font_color: TITLE_COLOR,
                            "3. Root placeholder + Portal"
                        }
                        text {
                            margin_top: 4.0,
                            font_size: 12.0,
                            font_color: STATUS_COLOR,
                            "PORTAL marker={root_state} expected=RED actions={props.portal_actions}"
                        }
                        text {
                            margin_top: 4.0,
                            font_size: 11.0,
                            line_height: 16.0,
                            font_color: MUTED_COLOR,
                            "ON 后右下角必须显示红色 PASS；若显示蓝色 FAIL 则层级错误。"
                        }
                        button {
                            width: 160.0,
                            height: 38.0,
                            margin_top: 8.0,
                            font_size: 13.0,
                            onclick: move |_| props.on_portal_toggle.call(()),
                            "PORTAL TOGGLE"
                        }
                    }

                    column {
                        width: "100%",
                        margin_top: 10.0,
                        margin_bottom: 72.0,
                        padding: 12.0,
                        background_color: CARD_BACKGROUND,
                        border_width: 1.0,
                        border_color: BORDER_COLOR,
                        border_radius: 10.0,
                        text {
                            font_size: 16.0,
                            font_weight: 700,
                            font_color: TITLE_COLOR,
                            "4. Typed dataset = JSON"
                        }
                        text {
                            margin_top: 4.0,
                            font_size: 12.0,
                            font_color: STATUS_COLOR,
                            "CHART version={props.dataset_version} expected={dataset_expected(props.dataset_version)} actions={props.chart_actions}"
                        }
                        text {
                            margin_top: 3.0,
                            font_size: 11.0,
                            line_height: 16.0,
                            font_color: MUTED_COLOR,
                            "{props.chart_status}"
                        }
                        row {
                            width: "100%",
                            margin_top: 8.0,
                            button {
                                width: 142.0,
                                height: 38.0,
                                font_size: 13.0,
                                onclick: move |_| props.on_chart_update.call(()),
                                "CHART UPDATE"
                            }
                            button {
                                width: 142.0,
                                height: 38.0,
                                margin_left: 8.0,
                                font_size: 13.0,
                                onclick: move |_| props.on_chart_inspect.call(()),
                                "CHART INSPECT"
                            }
                        }
                        text {
                            margin_top: 10.0,
                            font_size: 12.0,
                            font_weight: 600,
                            font_color: TITLE_COLOR,
                            "TYPED (empty series)"
                        }
                        ECharts {
                            option: props.typed_option,
                            height: "180",
                            controller: Some(props.typed_controller.clone()),
                        }
                        text {
                            margin_top: 8.0,
                            font_size: 12.0,
                            font_weight: 600,
                            font_color: TITLE_COLOR,
                            "JSON reference"
                        }
                        ECharts {
                            option: props.json_option,
                            height: "180",
                            controller: Some(props.json_controller.clone()),
                        }
                    }
                    super::chart_contracts::ChartContracts {}
                }
            }
        }
    }
}

fn keyed_order(step: u8) -> &'static [&'static str] {
    match step {
        1 => &["C", "A", "B"],
        3 => &["A", "C"],
        4 => &["A", "X", "C"],
        _ => &["A", "B", "C"],
    }
}

fn keyed_expected(step: u8) -> &'static str {
    match step {
        1 => "CAB/R0",
        2 => "ABC/R0(move-back)",
        3 => "AC/R0(remove-B)",
        4 => "AXC/R0(insert-X)",
        5 => "ABC/R1(replace-row-column)",
        _ => "ABC/R0",
    }
}

fn keyed_color(label: &str) -> &'static str {
    match label {
        "A" => "#FF2563EB",
        "B" => "#FF7C3AED",
        "C" => "#FF0891B2",
        _ => "#FFEA580C",
    }
}

fn dataset_rows(version: u8) -> Vec<Vec<DataValue>> {
    let values = if version == 0 {
        [("Mon", 12.0), ("Tue", 20.0)]
    } else {
        [("Mon", 30.0), ("Tue", 18.0)]
    };
    let mut rows = vec![vec![DataValue::from("day"), DataValue::from("Revenue")]];
    rows.extend(values.map(|(day, value)| vec![DataValue::from(day), DataValue::from(value)]));
    rows
}

fn typed_dataset_option(version: u8) -> ChartOption {
    ChartOption::new()
        .title(format!("Typed dataset v{version}"))
        .x_axis(Axis::category(std::iter::empty::<&str>()))
        .y_axis(Axis::value())
        // Deliberately declare the empty series before the dataset. Dataset
        // projection must not depend on builder call order.
        .push_series(Series::bar("Revenue", []))
        .dataset(Dataset {
            source: dataset_rows(version),
            dimensions: vec![String::from("day"), String::from("Revenue")],
            source_header: true,
            id: Some(String::from("typed-regression")),
            extra: BTreeMap::new(),
        })
}

fn json_dataset_option(version: u8) -> ChartOption {
    let source = if version == 0 {
        r#"[["day","Revenue"],["Mon",12],["Tue",20]]"#
    } else {
        r#"[["day","Revenue"],["Mon",30],["Tue",18]]"#
    };
    ChartOption::from_json_str(&format!(
        r#"{{"title":{{"text":"JSON dataset v{version}"}},"dataset":{{"source":{source}}},"xAxis":{{"type":"category"}},"yAxis":{{"type":"value"}},"series":[{{"name":"Revenue","type":"bar","data":[]}}]}}"#,
    ))
    .unwrap_or_else(|_| ChartOption::new())
}

fn dataset_expected(version: u8) -> &'static str {
    if version == 0 {
        "Mon:12,Tue:20"
    } else {
        "Mon:30,Tue:18"
    }
}

fn expected_chart_signature(version: u8) -> &'static str {
    if version == 0 {
        "cats=[Mon,Tue] values=[12,20]"
    } else {
        "cats=[Mon,Tue] values=[30,18]"
    }
}

fn chart_signature(option: &ChartOption) -> String {
    let categories = option
        .x_axis
        .first()
        .map(|axis| axis.data.join(","))
        .unwrap_or_default();
    let values = option
        .series
        .first()
        .and_then(|series| match series {
            Series::Line(series) | Series::Bar(series) => Some(&series.data),
            _ => None,
        })
        .map(|points| {
            points
                .iter()
                .map(|point| {
                    point
                        .values
                        .first()
                        .map(data_value_signature)
                        .unwrap_or_else(|| String::from("null"))
                })
                .collect::<Vec<_>>()
                .join(",")
        })
        .unwrap_or_default();
    format!("cats=[{categories}] values=[{values}]")
}

fn data_value_signature(value: &DataValue) -> String {
    match value {
        DataValue::Number(value) if value.fract() == 0.0 => format!("{value:.0}"),
        DataValue::Number(value) => value.to_string(),
        DataValue::String(value) => value.clone(),
        DataValue::Null => String::from("null"),
    }
}
