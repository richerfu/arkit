//! Parser for the supported ECharts-like JSON option subset.

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
        option.dataset = parse_dataset(value);
    }
    if let Some(value) = object.remove("visualMap") {
        option.visual_map = parse_visual_map(value);
    }
    if let Some(value) = object.remove("dataZoom") {
        option.data_zoom = parse_data_zoom_list(value);
    }
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

fn apply_dataset(option: &mut ChartOption) {
    let Some(dataset) = option.dataset.as_ref() else {
        return;
    };
    if dataset.source.is_empty() {
        return;
    }
    let has_header = dataset.source.first().is_some_and(|row| {
        row.iter()
            .any(|value| matches!(value, DataValue::String(_)))
    });
    let headers: Vec<String> = if has_header {
        dataset.source[0]
            .iter()
            .map(|value| match value {
                DataValue::String(value) => value.clone(),
                DataValue::Number(value) => value.to_string(),
            })
            .collect()
    } else {
        Vec::new()
    };
    let rows = &dataset.source[usize::from(has_header)..];
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
        Value::Object(object) => {
            let mut legend = Legend::default();
            legend.show = object.get("show").and_then(Value::as_bool).unwrap_or(true);
            legend.orient = object
                .get("orient")
                .and_then(Value::as_str)
                .unwrap_or("horizontal")
                .to_string();
            legend.left = object.get("left").cloned().unwrap_or(legend.left);
            legend.top = object.get("top").cloned().unwrap_or(legend.top);
            legend.data = object
                .get("data")
                .and_then(Value::as_array)
                .map(|values| values.iter().map(label_from_value).collect())
                .unwrap_or_default();
            legend.item_width = object
                .get("itemWidth")
                .and_then(parse_f32)
                .unwrap_or(legend.item_width);
            legend.item_height = object
                .get("itemHeight")
                .and_then(parse_f32)
                .unwrap_or(legend.item_height);
            legend.text_style = object
                .get("textStyle")
                .map(parse_text_options)
                .unwrap_or_default();
            Some(legend)
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
        split_line: object
            .get("splitLine")
            .and_then(Value::as_object)
            .and_then(|value| value.get("show"))
            .and_then(Value::as_bool)
            .unwrap_or(!matches!(orientation, AxisOrientation::X)),
        axis_label: object
            .get("axisLabel")
            .and_then(Value::as_object)
            .and_then(|value| value.get("show"))
            .and_then(Value::as_bool)
            .unwrap_or(true),
        grid_index: object.get("gridIndex").and_then(Value::as_u64).unwrap_or(0) as usize,
    }
}

fn parse_dataset(value: Value) -> Option<Dataset> {
    let source = match value {
        Value::Object(object) => object.get("source")?.clone(),
        value => value,
    };
    let rows = source.as_array()?;
    Some(Dataset {
        source: rows
            .iter()
            .filter_map(|row| {
                row.as_array().map(|cols| {
                    cols.iter()
                        .map(|value| {
                            value
                                .as_f64()
                                .map(DataValue::Number)
                                .unwrap_or_else(|| DataValue::String(label_from_value(value)))
                        })
                        .collect()
                })
            })
            .collect(),
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
    })
}

fn parse_visual_map(value: Value) -> Option<VisualMap> {
    let value = match value {
        Value::Array(mut values) => values.drain(..).next()?,
        value => value,
    };
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
    LinesSeries {
        name: Some(name),
        data,
        options: parse_series_options(object, "lines"),
    }
}

fn parse_line_segment(value: &Value) -> Option<LineSegment> {
    let object = value.as_object()?;
    let coords = object.get("coords")?.as_array()?;
    let from = coords.first()?.as_array()?;
    let to = coords.get(1)?.as_array()?;
    Some(LineSegment {
        name: object
            .get("name")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        from: (from.first()?.as_f64()?, from.get(1)?.as_f64()?),
        to: (to.first()?.as_f64()?, to.get(1)?.as_f64()?),
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
    if let Some(data) = object.get("data").and_then(Value::as_array) {
        for value in data {
            let Some(value) = value.as_object() else {
                continue;
            };
            let Some(name) = value.get("name").and_then(Value::as_str) else {
                continue;
            };
            let Some(feature) = features.iter_mut().find(|feature| feature.name == name) else {
                continue;
            };
            feature.value = value
                .get("value")
                .and_then(Value::as_f64)
                .unwrap_or(feature.value);
        }
    }
    MapSeries {
        name: Some(name),
        features,
        options: parse_series_options(object, "map"),
    }
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
    let name = object
        .get("properties")
        .and_then(Value::as_object)
        .and_then(|properties| properties.get("name"))
        .and_then(Value::as_str)
        .or_else(|| object.get("name").and_then(Value::as_str))
        .unwrap_or("feature")
        .to_string();
    let feature_value = object
        .get("value")
        .and_then(Value::as_f64)
        .or_else(|| {
            object
                .get("properties")
                .and_then(Value::as_object)
                .and_then(|p| p.get("value"))
                .and_then(Value::as_f64)
        })
        .unwrap_or(1.0);
    let geometry = object.get("geometry").unwrap_or(value);
    let polygons = parse_polygons(geometry)?;
    Some(MapFeature {
        name,
        value: feature_value,
        polygons,
    })
}

fn parse_polygons(value: &Value) -> Option<Vec<Vec<(f64, f64)>>> {
    let object = value.as_object()?;
    let kind = object.get("type").and_then(Value::as_str)?;
    let coordinates = object.get("coordinates")?;
    match kind {
        "Polygon" => Some(parse_polygon_array(coordinates)),
        "MultiPolygon" => Some(
            coordinates
                .as_array()?
                .iter()
                .flat_map(parse_polygon_array)
                .collect(),
        ),
        _ => None,
    }
}

fn parse_polygon_array(value: &Value) -> Vec<Vec<(f64, f64)>> {
    value
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
        .collect()
}

fn parse_node_data((index, value): (usize, &Value)) -> NodeData {
    match value {
        Value::Object(object) => NodeData {
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
        },
        Value::String(name) => NodeData {
            name: name.clone(),
            value: 1.0,
            x: None,
            y: None,
            category: None,
            symbol_size: None,
        },
        _ => NodeData {
            name: format!("node {index}"),
            value: value.as_f64().unwrap_or(1.0),
            x: None,
            y: None,
            category: None,
            symbol_size: None,
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

fn parse_data_point(value: &Value) -> DataPoint {
    match value {
        Value::Number(number) => DataPoint::scalar(number.as_f64().unwrap_or_default()),
        Value::String(text) => DataPoint::scalar(text.as_str()),
        Value::Array(values) => DataPoint::values(values.iter().map(|value| {
            value
                .as_f64()
                .map(DataValue::Number)
                .unwrap_or_else(|| DataValue::String(label_from_value(value)))
        })),
        Value::Object(object) => {
            let mut point = object
                .get("value")
                .map(parse_data_point)
                .unwrap_or_else(|| DataPoint::scalar(0.0));
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
        _ => DataPoint::scalar(0.0),
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
        "showSymbol",
        "symbolSize",
        "barWidth",
        "stack",
        "selectedMode",
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
    SeriesOptions {
        item_style: object
            .get("itemStyle")
            .map(parse_item_style)
            .unwrap_or_default(),
        line_style: object
            .get("lineStyle")
            .map(parse_line_style)
            .unwrap_or_default(),
        area_style,
        label,
        smooth,
        show_symbol: object
            .get("showSymbol")
            .and_then(Value::as_bool)
            .unwrap_or(true),
        symbol_size: object
            .get("symbolSize")
            .and_then(parse_f32)
            .unwrap_or(7.0)
            .max(0.0),
        bar_width: object.get("barWidth").and_then(parse_f32),
        stack: object
            .get("stack")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        selected_mode: object.get("selectedMode").and_then(|value| match value {
            Value::String(value) => Some(value.clone()),
            Value::Bool(value) => Some(value.to_string()),
            _ => None,
        }),
        extra: extra.into_iter().collect(),
    }
}

fn parse_item_style(value: &Value) -> ItemStyle {
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
        opacity: object
            .get("opacity")
            .and_then(Value::as_f64)
            .unwrap_or(1.0)
            .clamp(0.0, 1.0) as f32,
    }
}

fn parse_line_style(value: &Value) -> LineStyle {
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
    }
}

fn parse_label_style(value: &Value) -> LabelStyle {
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
        formatter: object
            .get("formatter")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
    }
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
        assert_eq!(map.features[0].polygons[0].len(), 5);
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
}
