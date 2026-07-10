//! Chart chrome atoms: title, legend, visual map, and tooltip.

use ohos_drawing_binding::Canvas;
use std::collections::BTreeSet;

use super::geometry::color;
use super::hit::{HitRegion, HitShape};
use super::style::gradient_color;
use super::surface::{draw_text, fill_circle, fill_rect, stroke_circle, stroke_rect};
use super::viewport::{slider_plot, ZoomWindow};
use crate::model::{ChartEvent, ChartOption, Title};

pub(super) fn draw_title(
    canvas: &Canvas,
    option: &ChartOption,
    title: &Title,
    width: f32,
    height: f32,
) {
    let estimated_width = title.text.chars().count() as f32 * title.text_style.font_size * 0.56;
    let x = horizontal_position(&title.left, width, estimated_width, 5.0);
    let y = vertical_position(&title.top, height, title.text_style.font_size, 5.0);
    draw_text(
        canvas,
        &title.text,
        x,
        y + title.text_style.font_size,
        title.text_style.font_size as f64,
        title
            .text_style
            .color
            .unwrap_or(option.visual_style.text_color),
        title.text_style.font_weight,
    );
    if let Some(subtext) = &title.subtext {
        draw_text(
            canvas,
            subtext,
            x,
            y + title.text_style.font_size + title.subtext_style.font_size + 4.0,
            title.subtext_style.font_size as f64,
            title
                .subtext_style
                .color
                .unwrap_or(option.visual_style.text_color),
            title.subtext_style.font_weight,
        );
    }
}

pub(super) fn draw_legend(
    canvas: Option<&Canvas>,
    option: &ChartOption,
    width: f32,
    height: f32,
    palette: &[u32],
    hidden_series: &BTreeSet<usize>,
    hits: &mut Vec<HitRegion>,
) {
    let Some(legend) = option.legend.as_ref().filter(|legend| legend.show) else {
        return;
    };
    let entries: Vec<(usize, &str)> = option
        .series
        .iter()
        .enumerate()
        .filter_map(|(index, series)| {
            let name = series.name()?;
            (legend.data.is_empty() || legend.data.iter().any(|entry| entry == name))
                .then_some((index, name))
        })
        .collect();
    let widths: Vec<f32> = entries
        .iter()
        .map(|(_, name)| {
            legend.item_width
                + 7.0
                + name.chars().count() as f32 * legend.text_style.font_size * 0.56
                + 12.0
        })
        .collect();
    let content_width = if legend.orient == "vertical" {
        widths.iter().copied().reduce(f32::max).unwrap_or(0.0)
    } else {
        widths.iter().sum()
    };
    let content_height = if legend.orient == "vertical" {
        entries.len() as f32 * (legend.item_height.max(legend.text_style.font_size) + 8.0)
    } else {
        legend.item_height.max(legend.text_style.font_size)
    };
    let mut x = horizontal_position(&legend.left, width, content_width, 5.0);
    let mut y = vertical_position(&legend.top, height, content_height, 5.0);
    for ((series_index, name), entry_width) in entries.into_iter().zip(widths) {
        let entry_height = legend.item_height.max(legend.text_style.font_size) + 6.0;
        if let Some(canvas) = canvas {
            fill_rect(
                canvas,
                x,
                y + (legend.text_style.font_size - legend.item_height) / 2.0,
                legend.item_width,
                legend.item_height,
                if hidden_series.contains(&series_index) {
                    0xFFB8B8B8
                } else {
                    color(palette, series_index)
                },
            );
            draw_text(
                canvas,
                name,
                x + legend.item_width + 7.0,
                y + legend.text_style.font_size,
                legend.text_style.font_size as f64,
                if hidden_series.contains(&series_index) {
                    0xFFB8B8B8
                } else {
                    legend
                        .text_style
                        .color
                        .unwrap_or(option.visual_style.text_color)
                },
                legend.text_style.font_weight,
            );
        }
        hits.push(HitRegion {
            shape: HitShape::Rect {
                x,
                y,
                width: entry_width,
                height: entry_height,
            },
            event: ChartEvent {
                series_index,
                data_index: 0,
                series_name: Some(name.to_string()),
                name: Some(name.to_string()),
                value: Vec::new(),
                x,
                y,
                component_type: String::from("legend"),
            },
        });
        if legend.orient == "vertical" {
            y += legend.item_height.max(legend.text_style.font_size) + 8.0;
        } else {
            x += entry_width;
        }
    }
}

pub(super) fn draw_visual_map(canvas: &Canvas, option: &ChartOption, width: f32, height: f32) {
    let Some(visual_map) = option
        .visual_map
        .as_ref()
        .filter(|visual_map| visual_map.show)
    else {
        return;
    };
    let bar_width = 12.0;
    let bar_height = 90.0_f32.min(height * 0.35);
    let x = width - bar_width - 12.0;
    let y = (height - bar_height) / 2.0;
    let steps = bar_height.max(1.0) as usize;
    for step in 0..steps {
        let normalized = 1.0 - step as f64 / steps.max(1) as f64;
        fill_rect(
            canvas,
            x,
            y + step as f32,
            bar_width,
            1.5,
            gradient_color(&visual_map.colors, normalized),
        );
    }
    draw_text(
        canvas,
        &format_value(visual_map.max),
        x - 4.0,
        y - 4.0,
        9.0,
        option.visual_style.text_color,
        400,
    );
    draw_text(
        canvas,
        &format_value(visual_map.min),
        x - 4.0,
        y + bar_height + 12.0,
        9.0,
        option.visual_style.text_color,
        400,
    );
}

pub(super) fn draw_data_zoom(
    canvas: Option<&Canvas>,
    option: &ChartOption,
    windows: &[ZoomWindow],
    width: f32,
    height: f32,
    hits: &mut Vec<HitRegion>,
) {
    for (index, data_zoom) in option.data_zoom.iter().enumerate() {
        if data_zoom.kind != "slider" || !data_zoom.show {
            continue;
        }
        let Some(track) = slider_plot(option, index, width, height) else {
            continue;
        };
        let window = windows
            .get(index)
            .copied()
            .unwrap_or_else(|| ZoomWindow::new(data_zoom.start, data_zoom.end));
        let vertical = data_zoom.orient == "vertical";
        let (start, end) = if vertical {
            (
                track.y + track.height * window.start as f32 / 100.0,
                track.y + track.height * window.end as f32 / 100.0,
            )
        } else {
            (
                track.x + track.width * window.start as f32 / 100.0,
                track.x + track.width * window.end as f32 / 100.0,
            )
        };
        if let Some(canvas) = canvas {
            fill_rect(
                canvas,
                track.x,
                track.y,
                track.width,
                track.height,
                0xFFE5E7EB,
            );
            if vertical {
                fill_rect(
                    canvas,
                    track.x,
                    start,
                    track.width,
                    (end - start).max(1.0),
                    0x665470C6,
                );
                for y in [start, end] {
                    fill_circle(canvas, track.x + track.width / 2.0, y, 6.0, 0xFF5470C6);
                    stroke_circle(canvas, track.x + track.width / 2.0, y, 6.0, 0xFFFFFFFF, 1.0);
                }
            } else {
                fill_rect(
                    canvas,
                    start,
                    track.y,
                    (end - start).max(1.0),
                    track.height,
                    0x665470C6,
                );
                for x in [start, end] {
                    fill_circle(canvas, x, track.y + track.height / 2.0, 6.0, 0xFF5470C6);
                    stroke_circle(
                        canvas,
                        x,
                        track.y + track.height / 2.0,
                        6.0,
                        0xFFFFFFFF,
                        1.0,
                    );
                }
            }
        }
        let event = |data_index: usize, x: f32, y: f32| ChartEvent {
            series_index: index,
            data_index,
            series_name: None,
            name: Some(String::from("dataZoom")),
            value: vec![window.start, window.end],
            x,
            y,
            component_type: String::from("dataZoom"),
        };
        if vertical {
            hits.push(HitRegion {
                shape: HitShape::Rect {
                    x: track.x - 6.0,
                    y: start - 10.0,
                    width: track.width + 12.0,
                    height: 20.0,
                },
                event: event(0, track.x + track.width / 2.0, start),
            });
            hits.push(HitRegion {
                shape: HitShape::Rect {
                    x: track.x - 6.0,
                    y: end - 10.0,
                    width: track.width + 12.0,
                    height: 20.0,
                },
                event: event(1, track.x + track.width / 2.0, end),
            });
            hits.push(HitRegion {
                shape: HitShape::Rect {
                    x: track.x,
                    y: start + 10.0,
                    width: track.width,
                    height: (end - start - 20.0).max(1.0),
                },
                event: event(2, track.x + track.width / 2.0, (start + end) / 2.0),
            });
        } else {
            hits.push(HitRegion {
                shape: HitShape::Rect {
                    x: start - 10.0,
                    y: track.y - 6.0,
                    width: 20.0,
                    height: track.height + 12.0,
                },
                event: event(0, start, track.y + track.height / 2.0),
            });
            hits.push(HitRegion {
                shape: HitShape::Rect {
                    x: end - 10.0,
                    y: track.y - 6.0,
                    width: 20.0,
                    height: track.height + 12.0,
                },
                event: event(1, end, track.y + track.height / 2.0),
            });
            hits.push(HitRegion {
                shape: HitShape::Rect {
                    x: start + 10.0,
                    y: track.y,
                    width: (end - start - 20.0).max(1.0),
                    height: track.height,
                },
                event: event(2, (start + end) / 2.0, track.y + track.height / 2.0),
            });
        }
    }
}

pub(super) fn draw_tooltip(
    canvas: &Canvas,
    option: &ChartOption,
    event: &ChartEvent,
    hidden_series: &BTreeSet<usize>,
    width: f32,
    height: f32,
) {
    if option.tooltip.trigger == "axis" && is_series_event(&event.component_type) {
        draw_axis_tooltip(canvas, option, event, hidden_series, width, height);
        return;
    }
    let name = event
        .name
        .as_deref()
        .or(event.series_name.as_deref())
        .unwrap_or("value");
    let values = event
        .value
        .iter()
        .map(|value| format_value(*value))
        .collect::<Vec<_>>()
        .join(", ");
    let label = option
        .tooltip
        .formatter
        .as_deref()
        .unwrap_or("{b}: {c}")
        .replace("{a}", event.series_name.as_deref().unwrap_or_default())
        .replace("{b}", name)
        .replace("{c}", &values);
    let padding = option.tooltip.padding.max(0.0);
    let w = (label.chars().count() as f32 * 6.5 + padding * 2.0).clamp(72.0, 240.0);
    let h = 18.0 + padding * 2.0;
    let x = event.x.min(width - w - 8.0).max(8.0);
    let y = (event.y - h - 10.0).min(height - h - 8.0).max(8.0);
    fill_rect(canvas, x, y, w, h, option.tooltip.background_color);
    if option.tooltip.border_color >> 24 != 0 {
        stroke_rect(canvas, x, y, w, h, option.tooltip.border_color, 1.0);
    }
    draw_text(
        canvas,
        &label,
        x + padding,
        y + padding + 13.0,
        11.0,
        option.tooltip.text_color,
        500,
    );
}

fn draw_axis_tooltip(
    canvas: &Canvas,
    option: &ChartOption,
    event: &ChartEvent,
    hidden_series: &BTreeSet<usize>,
    width: f32,
    height: f32,
) {
    let selected_axes = option
        .series
        .get(event.series_index)
        .map(super::series::cartesian_axis_indices)
        .unwrap_or((0, 0));
    let axis_label = option
        .x_axis
        .get(selected_axes.0)
        .and_then(|axis| axis.data.get(event.data_index))
        .cloned()
        .unwrap_or_else(|| event.data_index.to_string());
    let entries: Vec<(usize, String)> = option
        .series
        .iter()
        .enumerate()
        .filter_map(|(series_index, series)| {
            if hidden_series.contains(&series_index)
                || !super::series::is_cartesian(series)
                || super::series::cartesian_axis_indices(series) != selected_axes
            {
                return None;
            }
            let point = super::series::data(series).get(event.data_index)?;
            let name = series.name().unwrap_or("series");
            let values = point
                .values
                .iter()
                .filter_map(crate::model::DataValue::as_f64)
                .map(format_value)
                .collect::<Vec<_>>()
                .join(", ");
            Some((series_index, format!("{name}: {values}")))
        })
        .collect();
    let longest = std::iter::once(axis_label.chars().count())
        .chain(entries.iter().map(|(_, label)| label.chars().count()))
        .max()
        .unwrap_or(8);
    let padding = option.tooltip.padding.max(0.0);
    let tooltip_width = (longest as f32 * 6.5 + padding * 2.0 + 10.0).clamp(90.0, 260.0);
    let tooltip_height = padding * 2.0 + 18.0 + entries.len() as f32 * 16.0;
    let x = event.x.min(width - tooltip_width - 8.0).max(8.0);
    let y = (event.y - tooltip_height - 10.0)
        .min(height - tooltip_height - 8.0)
        .max(8.0);
    fill_rect(
        canvas,
        x,
        y,
        tooltip_width,
        tooltip_height,
        option.tooltip.background_color,
    );
    draw_text(
        canvas,
        &axis_label,
        x + padding,
        y + padding + 12.0,
        11.0,
        option.tooltip.text_color,
        600,
    );
    for (row, (series_index, label)) in entries.iter().enumerate() {
        let baseline = y + padding + 29.0 + row as f32 * 16.0;
        fill_circle(
            canvas,
            x + padding + 4.0,
            baseline - 4.0,
            3.0,
            color(&option.visual_style.palette, *series_index),
        );
        draw_text(
            canvas,
            label,
            x + padding + 11.0,
            baseline,
            10.0,
            option.tooltip.text_color,
            400,
        );
    }
}

fn is_series_event(component_type: &str) -> bool {
    matches!(
        component_type,
        "line"
            | "bar"
            | "scatter"
            | "effectScatter"
            | "heatmap"
            | "candlestick"
            | "boxplot"
            | "pictorialBar"
    )
}

fn horizontal_position(value: &serde_json::Value, total: f32, content: f32, default: f32) -> f32 {
    match value.as_str() {
        Some("center") => (total - content) / 2.0,
        Some("right") => total - content - 5.0,
        Some("left") => 5.0,
        _ => super::compat::length(Some(value), total, default),
    }
}

fn vertical_position(value: &serde_json::Value, total: f32, content: f32, default: f32) -> f32 {
    match value.as_str() {
        Some("middle") | Some("center") => (total - content) / 2.0,
        Some("bottom") => total - content - 5.0,
        Some("top") => 5.0,
        _ => super::compat::length(Some(value), total, default),
    }
}

fn format_value(value: f64) -> String {
    if (value - value.round()).abs() < 1e-8 {
        format!("{value:.0}")
    } else {
        format!("{value:.2}")
    }
}
