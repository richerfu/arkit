//! Interactive probes for the chart input and controller contracts.

use arkit::echarts::{
    ChartAppendData, ChartController, ChartCoordinateFinder, ChartCoordinatePoint, ChartOption,
    DataPoint, DataValue, ECharts, Series,
};
use arkit::prelude::*;

#[component]
pub(crate) fn ChartContracts() -> Element {
    let mut mounted = use_signal(|| true);
    let mut use_second = use_signal(|| false);
    let mut duplicate = use_signal(|| false);
    let mut status = use_signal(|| String::from("CC result=READY_TO_CHECK"));
    let mut source = use_signal(contract_option);
    let first = use_hook(ChartController::new);
    let second = use_hook(ChartController::new);
    let active = if use_second() {
        second.clone()
    } else {
        first.clone()
    };

    let unbound_controller = active.clone();
    let invalid_controller = active.clone();
    let inspect_controller = active.clone();
    let hit_controller = active.clone();
    let active_status = active.clone();
    let first_status = first.clone();
    let second_status = second.clone();

    rsx! {
        column {
            width: "100%",
            margin_top: 10.0,
            margin_bottom: 72.0,
            padding: 12.0,
            background_color: "#FFFFFFFF",
            border_width: 1.0,
            border_color: "#FFE2E8F0",
            border_radius: 10.0,
            text {
                font_size: 16.0,
                font_weight: 700,
                font_color: "#FF0F172A",
                "5. Chart controller contracts"
            }
            text {
                margin_top: 4.0,
                font_size: 11.0,
                line_height: 16.0,
                font_color: "#FF0369A1",
                "{status}"
            }
            row {
                width: "100%",
                margin_top: 8.0,
                button {
                    width: 142.0,
                    height: 38.0,
                    font_size: 12.0,
                    onclick: move |_| {
                        mounted.set(false);
                        status.set(String::from("CC result=UNMOUNTED_PENDING"));
                    },
                    "CC UNMOUNT"
                }
                button {
                    width: 142.0,
                    height: 38.0,
                    margin_left: 8.0,
                    font_size: 12.0,
                    onclick: move |_| {
                        let result = unbound_controller.clear();
                        status.set(format!("CC UNBOUND result={result:?}"));
                    },
                    "CC UNBOUND CMD"
                }
            }
            row {
                width: "100%",
                margin_top: 6.0,
                button {
                    width: 142.0,
                    height: 38.0,
                    font_size: 12.0,
                    onclick: move |_| {
                        mounted.set(true);
                        status.set(String::from("CC result=REMOUNT_PENDING"));
                    },
                    "CC REMOUNT"
                }
                button {
                    width: 142.0,
                    height: 38.0,
                    margin_left: 8.0,
                    font_size: 12.0,
                    onclick: move |_| {
                        use_second.toggle();
                        status.set(String::from("CC result=SWAP_PENDING"));
                    },
                    "CC SWAP CTRL"
                }
            }
            row {
                width: "100%",
                margin_top: 6.0,
                button {
                    width: 142.0,
                    height: 38.0,
                    font_size: 12.0,
                    onclick: move |_| {
                        let next = if source.read().datasets[0].source[1][1]
                            == DataValue::Number(12.0)
                        {
                            30.0
                        } else {
                            12.0
                        };
                        source.write().datasets[0].source[1][1] = DataValue::Number(next);
                        status.set(format!("CC DATA expected={next}"));
                    },
                    "CC MUTATE DATA"
                }
                button {
                    width: 142.0,
                    height: 38.0,
                    margin_left: 8.0,
                    font_size: 12.0,
                    onclick: move |_| {
                        let result = invalid_controller.append_data(ChartAppendData::scatter(
                            0,
                            [DataPoint::values([1.0, 2.0])],
                        ));
                        status.set(format!("CC APPEND result={result:?}"));
                    },
                    "CC BAD APPEND"
                }
            }
            row {
                width: "100%",
                margin_top: 6.0,
                button {
                    width: 142.0,
                    height: 38.0,
                    font_size: 12.0,
                    onclick: move |_| {
                        let raw_empty = inspect_controller
                            .get_source_option()
                            .ok()
                            .and_then(|option| match option.series.first() {
                                Some(Series::Bar(series)) => Some(series.data.is_empty()),
                                _ => None,
                            });
                        let resolved = inspect_controller
                            .get_option()
                            .ok()
                            .and_then(|snapshot| match snapshot.option().series.first() {
                                Some(Series::Bar(series)) => {
                                    series.data.first().map(|point| point.values[0].clone())
                                }
                                _ => None,
                            });
                        status.set(format!(
                            "CC INSPECT raw_empty={raw_empty:?} resolved={resolved:?} ready={}",
                            inspect_controller.is_ready(),
                        ));
                    },
                    "CC INSPECT"
                }
                button {
                    width: 142.0,
                    height: 38.0,
                    margin_left: 8.0,
                    font_size: 12.0,
                    onclick: move |_| {
                        let result = hit_controller
                            .convert_to_pixel(
                                ChartCoordinateFinder::series(0),
                                ChartCoordinatePoint::values("Mon", 6.0),
                            )
                            .and_then(|point| hit_controller.hit_test(point));
                        let label = match result {
                            Ok(Some(event)) => event.name.unwrap_or_else(|| String::from("ITEM")),
                            Ok(None) => String::from("NONE"),
                            Err(error) => format!("ERR_{error:?}"),
                        };
                        status.set(format!("CC HIT result={label}"));
                    },
                    "CC HIT CACHE"
                }
            }
            row {
                width: "100%",
                margin_top: 6.0,
                button {
                    width: 142.0,
                    height: 38.0,
                    font_size: 12.0,
                    onclick: move |_| {
                        duplicate.toggle();
                        status.set(String::from("CC result=DUP_PENDING"));
                    },
                    "CC DUPLICATE"
                }
                button {
                    width: 142.0,
                    height: 38.0,
                    margin_left: 8.0,
                    font_size: 12.0,
                    onclick: move |_| {
                        status.set(format!(
                            "CC BIND first={}/{} second={}/{} height={:?} errors={:?}/{:?}",
                            first_status.is_bound(),
                            first_status.is_ready(),
                            second_status.is_bound(),
                            second_status.is_ready(),
                            active_status.get_height(),
                            first_status.last_binding_error(),
                            second_status.last_binding_error(),
                        ));
                    },
                    "CC BIND CHECK"
                }
            }
            if mounted() {
                ECharts {
                    option: source(),
                    height: "180",
                    controller: Some(active.clone()),
                }
                if duplicate() {
                    ECharts {
                        option: duplicate_option(),
                        height: "100",
                        controller: Some(active),
                    }
                }
            }
        }
    }
}

fn contract_option() -> ChartOption {
    ChartOption::from_json_str(
        r#"{
            "animation":false,
            "dataset":{"source":[["day","value"],["Mon",12],["Tue",20]]},
            "xAxis":{"type":"category"},
            "yAxis":{"type":"value"},
            "series":[{"type":"bar","name":"Orders"}]
        }"#,
    )
    .expect("static chart contract option")
}

fn duplicate_option() -> ChartOption {
    ChartOption::from_json_str(
        r#"{
            "animation":false,
            "title":{"text":"DUPLICATE"},
            "dataset":{"source":[["day","value"],["Mon",99]]},
            "xAxis":{"type":"category"},
            "yAxis":{"type":"value"},
            "series":[{"type":"bar","name":"Duplicate"}]
        }"#,
    )
    .expect("static duplicate chart option")
}
