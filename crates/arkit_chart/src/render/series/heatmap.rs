use super::super::prelude::*;
use super::super::{compat, geometry::Plot};

pub(super) fn render(series: &BasicSeries, context: &mut CartesianRenderContext<'_>) {
    let series_index = context.series_index;
    let plot = context.plot;
    let layout = context.layout;
    let palette = context.palette;
    let canvas = context.canvas;
    let hits = &mut *context.hits;

    let max_x = layout.x.count().max(
        series
            .data
            .iter()
            .filter_map(|point| point.number_opt(0))
            .map(|value| value.max(0.0) as usize + 1)
            .max()
            .unwrap_or(1),
    );
    let max_y = layout.y.count().max(
        series
            .data
            .iter()
            .filter_map(|point| point.number_opt(1))
            .map(|value| value.max(0.0) as usize + 1)
            .max()
            .unwrap_or(1),
    );
    let cell_w = layout.x.band_width(plot, false, max_x);
    let cell_h = layout.y.band_width(plot, true, max_y);
    let data_min = series
        .data
        .iter()
        .filter_map(|point| point.number_opt(2))
        .reduce(f64::min)
        .unwrap_or(0.0);
    let data_max = series
        .data
        .iter()
        .filter_map(|point| point.number_opt(2))
        .reduce(f64::max)
        .unwrap_or(1.0);
    for (index, point) in series.data.iter().enumerate() {
        let (Some(raw_x), Some(raw_y), Some(value)) = (
            point.number_opt(0),
            point.number_opt(1),
            point.number_opt(2),
        ) else {
            continue;
        };
        let x_index = raw_x.max(0.0) as usize;
        let y_index = raw_y.max(0.0) as usize;
        let x_value = (!layout.x.is_category()).then_some(raw_x);
        let y_value = (!layout.y.is_category()).then_some(raw_y);
        if !layout.x.contains(x_value, x_index) || !layout.y.contains(y_value, y_index) {
            continue;
        }
        let x = layout.x.band_start(plot, x_value, x_index, false, max_x);
        let y = layout.y.band_start(plot, y_value, y_index, true, max_y);
        let fill = point
            .item_style
            .color
            .map(|color| with_opacity(color, point.item_style.opacity))
            .unwrap_or_else(|| {
                context.visual_map.map_or_else(
                    || {
                        gradient_color(
                            palette,
                            (value - data_min) / (data_max - data_min).max(1e-12),
                        )
                    },
                    |visual_map| visual_map_color(visual_map, value),
                )
            });
        if let Some(canvas) = canvas {
            fill_rect(
                canvas,
                x,
                y,
                (cell_w - 1.0).max(1.0),
                (cell_h - 1.0).max(1.0),
                fill,
            );
            if let Some((border_color, border_width)) = border(series, Some(point)) {
                stroke_rect(canvas, x, y, cell_w, cell_h, border_color, border_width);
            }
            let label = effective_label(series, point);
            if label.show {
                set_next_data_index(index);
                draw_text(
                    canvas,
                    &format_label(&label, series, point, index),
                    x + 4.0,
                    y + cell_h / 2.0 + label.font_size / 2.0,
                    label.font_size as f64,
                    label.color.unwrap_or(0xFF333333),
                    label.font_weight,
                );
            }
        }
        hits.push(rect_hit(
            "heatmap",
            series_index,
            index,
            series.name.clone(),
            point,
            (x, y, cell_w, cell_h),
        ));
    }
}

pub(super) fn render_geo(series: &BasicSeries, context: &mut FreeRenderContext<'_>) {
    let series_index = context.series_index;
    let geo_index = super::geo_index(&context.option.series[series_index]).unwrap_or(0);
    let Some(transform) =
        super::map::transform_from_geo_component(context.option, context.plot, geo_index)
    else {
        return;
    };
    if super::should_draw_geo_base(context.option, series_index, geo_index) {
        super::map::draw_geo_component(context.option, context.plot, geo_index, context.canvas);
    }
    let data_min = series
        .data
        .iter()
        .filter_map(|point| point.number_opt(2))
        .reduce(f64::min)
        .unwrap_or(0.0);
    let data_max = series
        .data
        .iter()
        .filter_map(|point| point.number_opt(2))
        .reduce(f64::max)
        .unwrap_or(1.0);
    let size = series.options.symbol_size_dimensions.unwrap_or([
        series.options.symbol_size.max(12.0),
        series.options.symbol_size.max(12.0),
    ]);
    for (index, point) in series.data.iter().enumerate() {
        let (Some(longitude), Some(latitude), Some(value)) = (
            point.number_opt(0),
            point.number_opt(1),
            point.number_opt(2),
        ) else {
            continue;
        };
        let center = transform.project((longitude, latitude));
        let fill = point.item_style.color.unwrap_or_else(|| {
            context
                .option
                .visual_map_for_series(series_index)
                .map_or_else(
                    || {
                        gradient_color(
                            context.palette,
                            (value - data_min) / (data_max - data_min).max(1e-12),
                        )
                    },
                    |visual_map| visual_map_color(visual_map, value),
                )
        });
        let bounds = (
            center.0 - size[0] / 2.0,
            center.1 - size[1] / 2.0,
            size[0],
            size[1],
        );
        if let Some(canvas) = context.canvas {
            fill_rect(canvas, bounds.0, bounds.1, bounds.2, bounds.3, fill);
        }
        context.hits.push(rect_hit(
            "heatmap",
            series_index,
            index,
            series.name.clone(),
            point,
            bounds,
        ));
    }
}

pub(super) fn render_calendar(series: &BasicSeries, context: &mut FreeRenderContext<'_>) {
    let calendar_index = series
        .options
        .extra
        .get("calendarIndex")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0) as usize;
    let calendar = option_component(&context.option.extra, "calendar", calendar_index);
    let data_days = series
        .data
        .iter()
        .filter_map(|point| point.number_opt(0))
        .map(timestamp_day)
        .collect::<Vec<_>>();
    let (start_day, end_day, year_label) = calendar_range(calendar)
        .or_else(|| {
            Some((
                *data_days.iter().min()?,
                *data_days.iter().max()?,
                String::new(),
            ))
        })
        .unwrap_or((0, 0, String::new()));
    if end_day < start_day {
        return;
    }
    let start_weekday = weekday(start_day);
    let weeks = (start_weekday + (end_day - start_day) as usize + 1).div_ceil(7);
    let full = context.plot;
    let left = calendar
        .and_then(|calendar| calendar.get("left"))
        .map(|value| compat::position(Some(value), full.x, full.width, full.x + full.width * 0.08))
        .unwrap_or(full.x + full.width * 0.08);
    let top = calendar
        .and_then(|calendar| calendar.get("top"))
        .map(|value| {
            compat::position(
                Some(value),
                full.y,
                full.height,
                full.y + full.height * 0.18,
            )
        })
        .unwrap_or(full.y + full.height * 0.18);
    let width = calendar
        .and_then(|calendar| calendar.get("width"))
        .map(|value| compat::length(Some(value), full.width, full.width * 0.84))
        .unwrap_or(full.width * 0.84);
    let height = calendar
        .and_then(|calendar| calendar.get("height"))
        .map(|value| compat::length(Some(value), full.height, full.height * 0.62))
        .unwrap_or(full.height * 0.62);
    let orient = calendar
        .and_then(|calendar| calendar.get("orient"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or("horizontal");
    let (columns, rows) = if orient == "vertical" {
        (7, weeks)
    } else {
        (weeks, 7)
    };
    let cell_size = calendar
        .and_then(|calendar| calendar.get("cellSize"))
        .and_then(serde_json::Value::as_array);
    let cell_width = cell_size
        .and_then(|values| values.first())
        .and_then(serde_json::Value::as_f64)
        .map(|value| value as f32)
        .unwrap_or(width / columns.max(1) as f32);
    let cell_height = cell_size
        .and_then(|values| values.get(1))
        .and_then(serde_json::Value::as_f64)
        .map(|value| value as f32)
        .unwrap_or(height / rows.max(1) as f32);
    let calendar_plot = Plot {
        x: left,
        y: top,
        width: cell_width * columns as f32,
        height: cell_height * rows as f32,
    };
    let empty_color = nested_color(calendar, &["itemStyle", "color"]).unwrap_or(0xFFF8FAFC);
    let border_color =
        nested_color(calendar, &["splitLine", "lineStyle", "color"]).unwrap_or(0xFFE2E8F0);
    let border_width = nested_number(calendar, &["splitLine", "lineStyle", "width"], 1.0) as f32;
    if let Some(canvas) = context.canvas {
        for day in start_day..=end_day {
            let (column, row) = calendar_cell(day, start_day, start_weekday, orient);
            let x = calendar_plot.x + column as f32 * cell_width;
            let y = calendar_plot.y + row as f32 * cell_height;
            fill_rect(canvas, x, y, cell_width, cell_height, empty_color);
            stroke_rect(
                canvas,
                x,
                y,
                cell_width,
                cell_height,
                border_color,
                border_width,
            );
        }
        let day_names = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
        for (index, name) in day_names.iter().enumerate() {
            let (x, y) = if orient == "vertical" {
                (
                    calendar_plot.x + index as f32 * cell_width + 2.0,
                    calendar_plot.y - 6.0,
                )
            } else {
                (
                    calendar_plot.x - 28.0,
                    calendar_plot.y + index as f32 * cell_height + cell_height * 0.65,
                )
            };
            draw_text(canvas, name, x, y, 9.0, 0xFF64748B, 400);
        }
        if !year_label.is_empty() {
            draw_text(
                canvas,
                &year_label,
                calendar_plot.x,
                calendar_plot.y - 18.0,
                12.0,
                0xFF334155,
                500,
            );
        }
    }

    for (index, point) in series.data.iter().enumerate() {
        let (Some(timestamp), Some(value)) = (point.number_opt(0), point.number_opt(1)) else {
            continue;
        };
        let day = timestamp_day(timestamp);
        if day < start_day || day > end_day {
            continue;
        }
        let (column, row) = calendar_cell(day, start_day, start_weekday, orient);
        let x = calendar_plot.x + column as f32 * cell_width;
        let y = calendar_plot.y + row as f32 * cell_height;
        let fill = point.item_style.color.unwrap_or_else(|| {
            context
                .option
                .visual_map_for_series(context.series_index)
                .map_or_else(
                    || gradient_color(context.palette, value),
                    |visual_map| visual_map_color(visual_map, value),
                )
        });
        if let Some(canvas) = context.canvas {
            fill_rect(
                canvas,
                x + 1.0,
                y + 1.0,
                (cell_width - 2.0).max(1.0),
                (cell_height - 2.0).max(1.0),
                fill,
            );
            let label = effective_label(series, point);
            if label.show {
                set_next_data_index(index);
                draw_text(
                    canvas,
                    &format_label(&label, series, point, index),
                    x + 3.0,
                    y + cell_height * 0.65,
                    label.font_size as f64,
                    label.color.unwrap_or(0xFF1F2937),
                    label.font_weight,
                );
            }
        }
        context.hits.push(rect_hit(
            "heatmap",
            context.series_index,
            index,
            series.name.clone(),
            point,
            (x, y, cell_width, cell_height),
        ));
    }
}

fn option_component<'a>(
    extra: &'a std::collections::BTreeMap<String, serde_json::Value>,
    key: &str,
    index: usize,
) -> Option<&'a serde_json::Map<String, serde_json::Value>> {
    match extra.get(key)? {
        serde_json::Value::Array(values) => values.get(index)?.as_object(),
        value if index == 0 => value.as_object(),
        _ => None,
    }
}

fn calendar_range(
    calendar: Option<&serde_json::Map<String, serde_json::Value>>,
) -> Option<(i64, i64, String)> {
    let range = calendar?.get("range")?;
    if let Some(year) = range.as_str().filter(|value| value.len() == 4) {
        let start = DataValue::String(format!("{year}-01-01")).as_f64()?;
        let next =
            DataValue::String(format!("{}-01-01", year.parse::<i32>().ok()? + 1)).as_f64()?;
        return Some((
            timestamp_day(start),
            timestamp_day(next) - 1,
            year.to_string(),
        ));
    }
    let values = range.as_array()?;
    let start = parse_time_value(values.first()?)?;
    let end = parse_time_value(values.get(1)?)?;
    Some((timestamp_day(start), timestamp_day(end), String::new()))
}

fn parse_time_value(value: &serde_json::Value) -> Option<f64> {
    value.as_f64().or_else(|| {
        value
            .as_str()
            .and_then(|value| DataValue::String(value.to_string()).as_f64())
    })
}

fn timestamp_day(timestamp: f64) -> i64 {
    (timestamp / 86_400_000.0).floor() as i64
}

fn weekday(day: i64) -> usize {
    (day + 4).rem_euclid(7) as usize
}

fn calendar_cell(day: i64, start_day: i64, start_weekday: usize, orient: &str) -> (usize, usize) {
    let slot = start_weekday + (day - start_day) as usize;
    let week = slot / 7;
    let weekday = slot % 7;
    if orient == "vertical" {
        (weekday, week)
    } else {
        (week, weekday)
    }
}

fn nested_value<'a>(
    value: Option<&'a serde_json::Map<String, serde_json::Value>>,
    path: &[&str],
) -> Option<&'a serde_json::Value> {
    let first = path.first()?;
    path[1..]
        .iter()
        .try_fold(value?.get(*first)?, |value, key| {
            value.as_object().and_then(|value| value.get(*key))
        })
}

fn nested_color(
    value: Option<&serde_json::Map<String, serde_json::Value>>,
    path: &[&str],
) -> Option<u32> {
    nested_value(value, path).and_then(crate::parser::parse_color)
}

fn nested_number(
    value: Option<&serde_json::Map<String, serde_json::Value>>,
    path: &[&str],
    default: f64,
) -> f64 {
    nested_value(value, path)
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(default)
}

#[cfg(test)]
mod calendar_tests {
    use super::*;

    #[test]
    fn calendar_cells_advance_by_week() {
        let start = timestamp_day(
            DataValue::String(String::from("2026-01-01"))
                .as_f64()
                .unwrap(),
        );
        assert_eq!(weekday(start), 4);
        assert_eq!(calendar_cell(start, start, 4, "horizontal"), (0, 4));
        assert_eq!(calendar_cell(start + 3, start, 4, "horizontal"), (1, 0));
    }
}
