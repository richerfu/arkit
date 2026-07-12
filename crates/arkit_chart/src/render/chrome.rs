//! Chart chrome atoms: title, legend, visual map, and tooltip.

use ohos_drawing_binding::Canvas;
use std::collections::BTreeSet;

use super::geometry::color;
use super::hit::{HitRegion, HitShape};
use super::style::{gradient_color, legend_color};
use super::surface::{
    draw_text, fill_circle, fill_rect, fill_rounded_rect, stroke_arc_with_cap, stroke_circle,
    stroke_line, stroke_rect, stroke_rounded_rect,
};
use super::symbol::{draw_symbol, SymbolSpec};
use super::viewport::{slider_plot, ZoomWindow};
use crate::model::{BrushArea, ChartEvent, ChartOption, Series, Timeline, Title};
use crate::parser::parse_color;

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
    let entries: Vec<(usize, &str, String, &str)> = option
        .series
        .iter()
        .enumerate()
        .filter_map(|(index, series)| {
            let name = series.name()?;
            if !legend.data.is_empty() && !legend.data.iter().any(|entry| entry == name) {
                return None;
            }
            let label = legend
                .formatter
                .as_deref()
                .map(|formatter| formatter.replace("{name}", name))
                .unwrap_or_else(|| name.to_string());
            let icon = legend
                .data_icons
                .get(name)
                .map(String::as_str)
                .unwrap_or(&legend.icon);
            Some((index, name, label, icon))
        })
        .collect();
    let text_widths: Vec<f32> = entries
        .iter()
        .map(|(_, _, label, _)| label.chars().count() as f32 * legend.text_style.font_size * 0.56)
        .collect();
    let entry_widths = text_widths
        .iter()
        .map(|text_width| legend.item_width + 5.0 + text_width)
        .collect::<Vec<_>>();
    let entry_height = legend.item_height.max(legend.text_style.font_size);
    let gap = legend.item_gap.max(0.0);
    let mut positions = Vec::with_capacity(entries.len());
    let (content_width, content_height) = if legend.orient == "vertical" {
        let mut y = 0.0;
        let mut content_width = 0.0_f32;
        for entry_width in &entry_widths {
            positions.push((0.0, y));
            content_width = content_width.max(*entry_width);
            y += entry_height + gap;
        }
        (
            content_width,
            (y - gap).max(if entries.is_empty() {
                0.0
            } else {
                entry_height
            }),
        )
    } else {
        // Plain ECharts legends use box layout and wrap within the viewport.
        let max_row_width = (width - 10.0).max(1.0);
        let mut x = 0.0;
        let mut y = 0.0;
        let mut widest = 0.0_f32;
        for entry_width in &entry_widths {
            let next_x = if x > 0.0 { x + gap } else { x };
            if x > 0.0 && next_x + entry_width > max_row_width {
                widest = widest.max(x);
                x = 0.0;
                y += entry_height + gap;
            } else {
                x = next_x;
            }
            positions.push((x, y));
            x += entry_width;
        }
        widest = widest.max(x);
        (
            widest,
            if entries.is_empty() {
                0.0
            } else {
                y + entry_height
            },
        )
    };
    let origin_x = horizontal_position(&legend.left, width, content_width, 5.0);
    let origin_y = vertical_position(&legend.top, height, content_height, 5.0);
    let align_right = legend.align == "right"
        || (legend.align == "auto"
            && legend.orient == "vertical"
            && legend.left.as_str() == Some("right"));
    for ((((series_index, name, label, icon), text_width), entry_width), (x, y)) in entries
        .into_iter()
        .zip(text_widths)
        .zip(entry_widths)
        .zip(positions)
    {
        let x = origin_x + x;
        let y = origin_y + y;
        let hidden = hidden_series.contains(&series_index);
        let item_color = if hidden {
            legend.inactive_color
        } else {
            legend_color(&option.series[series_index], palette, series_index)
        };
        if let Some(canvas) = canvas {
            let icon_x = if align_right {
                x + text_width + 5.0 + legend.item_width / 2.0
            } else {
                x + legend.item_width / 2.0
            };
            let icon_y = y + entry_height / 2.0;
            let icon = if icon == "inherit" { "roundRect" } else { icon };
            if icon == "line" {
                stroke_line(
                    canvas,
                    icon_x - legend.item_width / 2.0,
                    icon_y,
                    icon_x + legend.item_width / 2.0,
                    icon_y,
                    item_color,
                    2.0,
                );
            } else {
                draw_symbol(
                    canvas,
                    &SymbolSpec {
                        name: icon,
                        size: [legend.item_width, legend.item_height],
                        rotate: 0.0,
                        offset: [0.0, 0.0],
                    },
                    icon_x,
                    icon_y,
                    item_color,
                    None,
                );
            }
            draw_text(
                canvas,
                &label,
                if align_right {
                    x
                } else {
                    x + legend.item_width + 5.0
                },
                y + (entry_height + legend.text_style.font_size) / 2.0,
                legend.text_style.font_size as f64,
                if hidden {
                    legend.inactive_color
                } else {
                    legend
                        .text_style
                        .color
                        .unwrap_or(option.visual_style.text_color)
                },
                legend.text_style.font_weight,
            );
        }
        if legend.selected_mode != "false" {
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
        }
    }
}

pub(super) fn draw_toolbox(
    canvas: Option<&Canvas>,
    option: &ChartOption,
    width: f32,
    height: f32,
    hits: &mut Vec<HitRegion>,
) {
    let Some(toolbox) = option
        .extra
        .get("toolbox")
        .and_then(serde_json::Value::as_object)
    else {
        return;
    };
    if !toolbox
        .get("show")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(true)
    {
        return;
    }
    let Some(features) = toolbox
        .get("feature")
        .and_then(serde_json::Value::as_object)
    else {
        return;
    };
    let base_style = toolbox_icon_style(toolbox.get("iconStyle"), ToolboxIconStyle::default());
    let mut supported = Vec::new();
    for (feature_name, feature) in features {
        if matches!(feature, serde_json::Value::Bool(false))
            || feature
                .get("show")
                .and_then(serde_json::Value::as_bool)
                .is_some_and(|show| !show)
        {
            continue;
        }
        let style = toolbox_icon_style(feature.get("iconStyle"), base_style);
        match feature_name.as_str() {
            "restore" => supported.push(ToolboxAction {
                name: String::from("restore"),
                icon: ToolboxIcon::Restore,
                style,
                active: false,
            }),
            "brush" => {
                let types = feature
                    .get("type")
                    .and_then(serde_json::Value::as_array)
                    .map(|values| {
                        values
                            .iter()
                            .filter_map(serde_json::Value::as_str)
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_else(|| vec!["rect", "clear"]);
                if types
                    .iter()
                    .any(|value| matches!(*value, "rect" | "lineX" | "lineY"))
                {
                    supported.push(ToolboxAction {
                        name: String::from("brush-rect"),
                        icon: ToolboxIcon::BrushRect,
                        style,
                        active: option.brush.as_ref().is_some_and(|brush| brush.active),
                    });
                }
                if types.contains(&"clear") {
                    supported.push(ToolboxAction {
                        name: String::from("brush-clear"),
                        icon: ToolboxIcon::BrushClear,
                        style,
                        active: false,
                    });
                }
            }
            "dataZoom" => {
                let types = toolbox_feature_types(feature, &["zoom", "back"]);
                if types.contains(&"zoom") {
                    supported.push(ToolboxAction {
                        name: String::from("data-zoom"),
                        icon: ToolboxIcon::DataZoom,
                        style,
                        active: feature
                            .get("__active")
                            .and_then(serde_json::Value::as_bool)
                            .unwrap_or(false),
                    });
                }
                if types.contains(&"back") {
                    supported.push(ToolboxAction {
                        name: String::from("data-zoom-back"),
                        icon: ToolboxIcon::DataZoomBack,
                        style,
                        active: feature
                            .get("__canBack")
                            .and_then(serde_json::Value::as_bool)
                            .unwrap_or(false),
                    });
                }
            }
            "magicType" => {
                for kind in toolbox_feature_types(feature, &[]) {
                    let (name, icon) = match kind {
                        "line" => ("magic-line", ToolboxIcon::MagicLine),
                        "bar" => ("magic-bar", ToolboxIcon::MagicBar),
                        "stack" => ("magic-stack", ToolboxIcon::MagicStack),
                        _ => continue,
                    };
                    supported.push(ToolboxAction {
                        name: String::from(name),
                        icon,
                        style,
                        active: magic_type_active(option, kind),
                    });
                }
            }
            "dataView" => supported.push(ToolboxAction {
                name: String::from("data-view"),
                icon: ToolboxIcon::DataView,
                style,
                active: feature
                    .get("__visible")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(false),
            }),
            "saveAsImage" => supported.push(ToolboxAction {
                name: String::from("save-as-image"),
                icon: ToolboxIcon::SaveAsImage,
                style,
                active: false,
            }),
            _ => {}
        }
    }
    if supported.is_empty() {
        return;
    }

    let item_size = toolbox_number(toolbox.get("itemSize"), 15.0).clamp(8.0, 64.0);
    let item_gap = toolbox_number(toolbox.get("itemGap"), 8.0).max(0.0);
    let padding = toolbox_padding(toolbox.get("padding"));
    let vertical = toolbox
        .get("orient")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|value| value == "vertical");
    let count = supported.len() as f32;
    let main_size = count * item_size + (count - 1.0).max(0.0) * item_gap;
    let (content_width, content_height) = if vertical {
        (item_size, main_size)
    } else {
        (main_size, item_size)
    };
    let outer_width = content_width + padding[1] + padding[3];
    let outer_height = content_height + padding[0] + padding[2];
    let outer_x = toolbox_axis_position(
        toolbox.get("left"),
        toolbox.get("right"),
        width,
        outer_width,
        false,
    );
    let outer_y = toolbox_axis_position(
        toolbox.get("top"),
        toolbox.get("bottom"),
        height,
        outer_height,
        true,
    );
    if let Some(canvas) = canvas {
        let background = toolbox
            .get("backgroundColor")
            .and_then(parse_color)
            .unwrap_or(0x00000000);
        let border_color = toolbox
            .get("borderColor")
            .and_then(parse_color)
            .unwrap_or(0xFFCCCCCC);
        let border_width = toolbox_number(toolbox.get("borderWidth"), 0.0).max(0.0);
        let radii = toolbox_radii(toolbox.get("borderRadius"));
        if background >> 24 != 0 {
            fill_rounded_rect(
                canvas,
                outer_x,
                outer_y,
                outer_width,
                outer_height,
                radii,
                background,
            );
        }
        if border_width > 0.0 && border_color >> 24 != 0 {
            stroke_rounded_rect(
                canvas,
                (outer_x, outer_y, outer_width, outer_height),
                radii,
                border_color,
                border_width,
            );
        }
    }

    let mut x = outer_x + padding[3];
    let mut y = outer_y + padding[0];
    for (index, action) in supported.into_iter().enumerate() {
        if let Some(canvas) = canvas {
            if action.active {
                fill_rounded_rect(
                    canvas,
                    x - 2.0,
                    y - 2.0,
                    item_size + 4.0,
                    item_size + 4.0,
                    [3.0; 4],
                    0x195470C6,
                );
            }
            draw_toolbox_icon(canvas, action.icon, x, y, item_size, action.style);
        }
        hits.push(HitRegion {
            shape: HitShape::Rect {
                x,
                y,
                width: item_size,
                height: item_size,
            },
            event: ChartEvent {
                series_index: 0,
                data_index: index,
                series_name: None,
                name: Some(action.name),
                value: Vec::new(),
                x: x + item_size / 2.0,
                y: y + item_size / 2.0,
                component_type: String::from("toolbox"),
            },
        });
        if vertical {
            y += item_size + item_gap;
        } else {
            x += item_size + item_gap;
        }
    }
}

pub(super) fn draw_data_view(
    canvas: Option<&Canvas>,
    option: &ChartOption,
    width: f32,
    height: f32,
    hits: &mut Vec<HitRegion>,
) {
    let Some(feature) = option
        .extra
        .get("toolbox")
        .and_then(serde_json::Value::as_object)
        .and_then(|toolbox| toolbox.get("feature"))
        .and_then(serde_json::Value::as_object)
        .and_then(|features| features.get("dataView"))
        .and_then(serde_json::Value::as_object)
        .filter(|feature| {
            feature
                .get("__visible")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false)
        })
    else {
        return;
    };
    let margin = 10.0;
    let panel_x = margin;
    let panel_y = margin;
    let panel_width = (width - margin * 2.0).max(1.0);
    let panel_height = (height - margin * 2.0).max(1.0);
    if let Some(canvas) = canvas {
        fill_rect(canvas, 0.0, 0.0, width, height, 0x990F172A);
        fill_rounded_rect(
            canvas,
            panel_x,
            panel_y,
            panel_width,
            panel_height,
            [7.0; 4],
            0xFFF8FAFC,
        );
        stroke_rounded_rect(
            canvas,
            (panel_x, panel_y, panel_width, panel_height),
            [7.0; 4],
            0xFFCBD5E1,
            1.0,
        );
        draw_text(
            canvas,
            "Data View",
            panel_x + 14.0,
            panel_y + 26.0,
            15.0,
            0xFF0F172A,
            600,
        );
        let close_x = panel_x + panel_width - 24.0;
        let close_y = panel_y + 10.0;
        stroke_line(
            canvas,
            close_x,
            close_y,
            close_x + 12.0,
            close_y + 12.0,
            0xFF475569,
            1.6,
        );
        stroke_line(
            canvas,
            close_x + 12.0,
            close_y,
            close_x,
            close_y + 12.0,
            0xFF475569,
            1.6,
        );
        stroke_line(
            canvas,
            panel_x + 12.0,
            panel_y + 36.0,
            panel_x + panel_width - 12.0,
            panel_y + 36.0,
            0xFFE2E8F0,
            1.0,
        );
        let lines = data_view_lines(option);
        let mut y = panel_y + 55.0;
        let max_y = panel_y + panel_height - 18.0;
        for line in lines {
            if y > max_y {
                draw_text(canvas, "…", panel_x + 14.0, max_y, 12.0, 0xFF64748B, 400);
                break;
            }
            let header = !line.starts_with("  ");
            draw_text(
                canvas,
                line.trim_end(),
                panel_x + if header { 14.0 } else { 24.0 },
                y,
                if header { 12.0 } else { 10.5 },
                if header { 0xFF1E293B } else { 0xFF475569 },
                if header { 600 } else { 400 },
            );
            y += if header { 18.0 } else { 15.0 };
        }
        if !feature
            .get("readOnly")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false)
        {
            draw_text(
                canvas,
                "Editing is delegated to the ArkUI host",
                panel_x + 14.0,
                panel_y + panel_height - 8.0,
                9.0,
                0xFF94A3B8,
                400,
            );
        }
    }
    hits.push(HitRegion {
        shape: HitShape::Rect {
            x: 0.0,
            y: 0.0,
            width,
            height,
        },
        event: ChartEvent {
            series_index: 0,
            data_index: 0,
            series_name: None,
            name: Some(String::from("data-view-overlay")),
            value: Vec::new(),
            x: width / 2.0,
            y: height / 2.0,
            component_type: String::from("toolbox"),
        },
    });
    hits.push(HitRegion {
        shape: HitShape::Rect {
            x: panel_x + panel_width - 34.0,
            y: panel_y + 4.0,
            width: 28.0,
            height: 28.0,
        },
        event: ChartEvent {
            series_index: 0,
            data_index: 1,
            series_name: None,
            name: Some(String::from("data-view-close")),
            value: Vec::new(),
            x: panel_x + panel_width - 20.0,
            y: panel_y + 18.0,
            component_type: String::from("toolbox"),
        },
    });
}

fn data_view_lines(option: &ChartOption) -> Vec<String> {
    let mut lines = Vec::new();
    for (series_index, series) in option.series.iter().enumerate() {
        let name = series
            .name()
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| format!("series {series_index}"));
        lines.push(name);
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
                for (index, point) in series.data.iter().take(12).enumerate() {
                    let label = point
                        .name
                        .clone()
                        .or_else(|| {
                            option
                                .x_axis
                                .first()
                                .and_then(|axis| axis.data.get(index).cloned())
                        })
                        .unwrap_or_else(|| index.to_string());
                    let values = point
                        .values
                        .iter()
                        .map(data_view_value)
                        .collect::<Vec<_>>()
                        .join(", ");
                    lines.push(format!("  {label}: {values}"));
                }
                if series.data.len() > 12 {
                    lines.push(format!("  … {} more", series.data.len() - 12));
                }
            }
            Series::Custom(series) => {
                for (index, point) in series.data.iter().take(12).enumerate() {
                    let values = point
                        .values
                        .iter()
                        .map(data_view_value)
                        .collect::<Vec<_>>()
                        .join(", ");
                    lines.push(format!("  {index}: {values}"));
                }
            }
            Series::Tree(series) | Series::Graph(series) => {
                lines.push(format!("  {} nodes", series.nodes.len()));
            }
            Series::Sankey(series) => lines.push(format!("  {} nodes", series.nodes.len())),
            Series::Map(series) => lines.push(format!("  {} regions", series.features.len())),
            Series::Lines(series) => lines.push(format!("  {} paths", series.data.len())),
            Series::Sunburst(series) => lines.push(format!("  {} root nodes", series.data.len())),
        }
    }
    if lines.is_empty() {
        lines.push(String::from("No series data"));
    }
    lines
}

fn data_view_value(value: &crate::model::DataValue) -> String {
    match value {
        crate::model::DataValue::Number(value) => format_value(*value),
        crate::model::DataValue::String(value) => value.clone(),
        crate::model::DataValue::Null => String::from("-"),
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ToolboxIcon {
    Restore,
    BrushRect,
    BrushClear,
    DataZoom,
    DataZoomBack,
    MagicLine,
    MagicBar,
    MagicStack,
    DataView,
    SaveAsImage,
}

struct ToolboxAction {
    name: String,
    icon: ToolboxIcon,
    style: ToolboxIconStyle,
    active: bool,
}

#[derive(Clone, Copy)]
struct ToolboxIconStyle {
    color: u32,
    width: f32,
    opacity: f32,
}

impl Default for ToolboxIconStyle {
    fn default() -> Self {
        Self {
            color: 0xFF5470C6,
            width: 1.5,
            opacity: 1.0,
        }
    }
}

fn toolbox_icon_style(
    value: Option<&serde_json::Value>,
    fallback: ToolboxIconStyle,
) -> ToolboxIconStyle {
    let Some(style) = value.and_then(serde_json::Value::as_object) else {
        return fallback;
    };
    ToolboxIconStyle {
        color: style
            .get("borderColor")
            .or_else(|| style.get("color"))
            .and_then(parse_color)
            .unwrap_or(fallback.color),
        width: toolbox_number(style.get("borderWidth"), fallback.width).max(0.5),
        opacity: toolbox_number(style.get("opacity"), fallback.opacity).clamp(0.0, 1.0),
    }
}

fn draw_toolbox_icon(
    canvas: &Canvas,
    icon: ToolboxIcon,
    x: f32,
    y: f32,
    size: f32,
    style: ToolboxIconStyle,
) {
    let color = super::style::with_opacity(style.color, style.opacity);
    let line_width = style.width.min(size * 0.18);
    let inset = size * 0.16;
    let left = x + inset;
    let top = y + inset;
    let right = x + size - inset;
    let bottom = y + size - inset;
    match icon {
        ToolboxIcon::Restore => {
            let center = (x + size / 2.0, y + size / 2.0);
            let radius = size * 0.31;
            let start = 0.28;
            let sweep = 4.85;
            stroke_arc_with_cap(
                canvas, center, radius, start, sweep, color, line_width, true,
            );
            let end = start + sweep;
            let tip = (center.0 + end.cos() * radius, center.1 + end.sin() * radius);
            let tangent = (-end.sin(), end.cos());
            let normal = (tangent.1, -tangent.0);
            let arrow = size * 0.22;
            for direction in [-1.0_f32, 1.0] {
                stroke_line(
                    canvas,
                    tip.0,
                    tip.1,
                    tip.0 - tangent.0 * arrow + normal.0 * arrow * 0.55 * direction,
                    tip.1 - tangent.1 * arrow + normal.1 * arrow * 0.55 * direction,
                    color,
                    line_width,
                );
            }
        }
        ToolboxIcon::BrushRect => {
            let arm = size * 0.27;
            for (corner_x, corner_y, dx, dy) in [
                (left, top, 1.0, 1.0),
                (right, top, -1.0, 1.0),
                (left, bottom, 1.0, -1.0),
                (right, bottom, -1.0, -1.0),
            ] {
                stroke_line(
                    canvas,
                    corner_x,
                    corner_y,
                    corner_x + dx * arm,
                    corner_y,
                    color,
                    line_width,
                );
                stroke_line(
                    canvas,
                    corner_x,
                    corner_y,
                    corner_x,
                    corner_y + dy * arm,
                    color,
                    line_width,
                );
            }
        }
        ToolboxIcon::BrushClear => {
            stroke_line(canvas, left, top, right, bottom, color, line_width);
            stroke_line(canvas, right, top, left, bottom, color, line_width);
        }
        ToolboxIcon::DataZoom => {
            let radius = size * 0.25;
            let center = (x + size * 0.44, y + size * 0.44);
            stroke_circle(canvas, center.0, center.1, radius, color, line_width);
            stroke_line(
                canvas,
                center.0 + radius * 0.72,
                center.1 + radius * 0.72,
                right,
                bottom,
                color,
                line_width,
            );
            stroke_line(
                canvas,
                center.0 - radius * 0.5,
                center.1,
                center.0 + radius * 0.5,
                center.1,
                color,
                line_width,
            );
            stroke_line(
                canvas,
                center.0,
                center.1 - radius * 0.5,
                center.0,
                center.1 + radius * 0.5,
                color,
                line_width,
            );
        }
        ToolboxIcon::DataZoomBack => {
            stroke_rect(
                canvas,
                x + size * 0.3,
                y + size * 0.28,
                size * 0.5,
                size * 0.5,
                color,
                line_width,
            );
            stroke_line(
                canvas,
                left,
                y + size * 0.42,
                right,
                y + size * 0.42,
                color,
                line_width,
            );
            stroke_line(
                canvas,
                left,
                y + size * 0.42,
                x + size * 0.34,
                top,
                color,
                line_width,
            );
            stroke_line(
                canvas,
                left,
                y + size * 0.42,
                x + size * 0.34,
                y + size * 0.58,
                color,
                line_width,
            );
        }
        ToolboxIcon::MagicLine => {
            let points = [
                (left, y + size * 0.68),
                (x + size * 0.38, y + size * 0.42),
                (x + size * 0.57, y + size * 0.58),
                (right, top),
            ];
            for pair in points.windows(2) {
                stroke_line(
                    canvas, pair[0].0, pair[0].1, pair[1].0, pair[1].1, color, line_width,
                );
            }
            stroke_line(canvas, left, bottom, right, bottom, color, line_width);
        }
        ToolboxIcon::MagicBar => {
            let bar_width = size * 0.14;
            for (bar_x, bar_top) in [
                (left, y + size * 0.52),
                (x + size * 0.43, y + size * 0.34),
                (x + size * 0.68, top),
            ] {
                stroke_rect(
                    canvas,
                    bar_x,
                    bar_top,
                    bar_width,
                    bottom - bar_top,
                    color,
                    line_width,
                );
            }
            stroke_line(canvas, left, bottom, right, bottom, color, line_width);
        }
        ToolboxIcon::MagicStack => {
            for offset in [size * -0.11, 0.0, size * 0.11] {
                let cy = y + size * 0.5 + offset;
                stroke_line(
                    canvas,
                    left,
                    cy,
                    x + size * 0.5,
                    cy + size * 0.18,
                    color,
                    line_width,
                );
                stroke_line(
                    canvas,
                    x + size * 0.5,
                    cy + size * 0.18,
                    right,
                    cy,
                    color,
                    line_width,
                );
            }
        }
        ToolboxIcon::DataView => {
            stroke_rect(
                canvas,
                left,
                top,
                right - left,
                bottom - top,
                color,
                line_width,
            );
            for row in [0.36_f32, 0.52, 0.68] {
                stroke_line(
                    canvas,
                    x + size * 0.28,
                    y + size * row,
                    x + size * 0.72,
                    y + size * row,
                    color,
                    line_width,
                );
            }
        }
        ToolboxIcon::SaveAsImage => {
            stroke_line(
                canvas,
                left,
                y + size * 0.62,
                left,
                bottom,
                color,
                line_width,
            );
            stroke_line(canvas, left, bottom, right, bottom, color, line_width);
            stroke_line(
                canvas,
                right,
                bottom,
                right,
                y + size * 0.62,
                color,
                line_width,
            );
            stroke_line(
                canvas,
                x + size * 0.5,
                top,
                x + size * 0.5,
                y + size * 0.64,
                color,
                line_width,
            );
            stroke_line(
                canvas,
                x + size * 0.32,
                y + size * 0.48,
                x + size * 0.5,
                y + size * 0.66,
                color,
                line_width,
            );
            stroke_line(
                canvas,
                x + size * 0.68,
                y + size * 0.48,
                x + size * 0.5,
                y + size * 0.66,
                color,
                line_width,
            );
        }
    }
}

fn toolbox_feature_types<'a>(
    feature: &'a serde_json::Value,
    defaults: &'a [&'a str],
) -> Vec<&'a str> {
    feature
        .get("type")
        .and_then(serde_json::Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(serde_json::Value::as_str)
                .collect()
        })
        .unwrap_or_else(|| defaults.to_vec())
}

fn magic_type_active(option: &ChartOption, kind: &str) -> bool {
    let eligible = option
        .series
        .iter()
        .filter(|series| matches!(series, Series::Line(_) | Series::Bar(_)))
        .collect::<Vec<_>>();
    !eligible.is_empty()
        && match kind {
            "line" => eligible
                .iter()
                .all(|series| matches!(series, Series::Line(_))),
            "bar" => eligible
                .iter()
                .all(|series| matches!(series, Series::Bar(_))),
            "stack" => eligible.iter().all(|series| match series {
                Series::Line(series) | Series::Bar(series) => {
                    series.options.stack.as_deref() == Some("__ec_magicType_stack__")
                }
                _ => false,
            }),
            _ => false,
        }
}

fn toolbox_axis_position(
    leading: Option<&serde_json::Value>,
    trailing: Option<&serde_json::Value>,
    total: f32,
    content: f32,
    vertical: bool,
) -> f32 {
    if let Some(value) = leading.filter(|value| !value.is_null() && value.as_str() != Some("auto"))
    {
        return if vertical {
            vertical_position(value, total, content, 5.0)
        } else {
            horizontal_position(value, total, content, 5.0)
        }
        .clamp(0.0, (total - content).max(0.0));
    }
    if let Some(value) = trailing.filter(|value| !value.is_null() && value.as_str() != Some("auto"))
    {
        return (total - content - edge_value(value, total, 5.0))
            .clamp(0.0, (total - content).max(0.0));
    }
    if vertical {
        5.0
    } else {
        (total - content - 5.0).max(0.0)
    }
}

fn toolbox_number(value: Option<&serde_json::Value>, fallback: f32) -> f32 {
    value
        .and_then(serde_json::Value::as_f64)
        .map(|value| value as f32)
        .unwrap_or(fallback)
}

fn toolbox_padding(value: Option<&serde_json::Value>) -> [f32; 4] {
    let Some(value) = value else {
        return [5.0; 4];
    };
    if let Some(padding) = value.as_f64() {
        return [(padding as f32).max(0.0); 4];
    }
    let Some(values) = value.as_array() else {
        return [5.0; 4];
    };
    let numbers = values
        .iter()
        .filter_map(serde_json::Value::as_f64)
        .map(|value| (value as f32).max(0.0))
        .collect::<Vec<_>>();
    match numbers.as_slice() {
        [all] => [*all; 4],
        [vertical, horizontal] => [*vertical, *horizontal, *vertical, *horizontal],
        [top, horizontal, bottom] => [*top, *horizontal, *bottom, *horizontal],
        [top, right, bottom, left, ..] => [*top, *right, *bottom, *left],
        _ => [5.0; 4],
    }
}

fn toolbox_radii(value: Option<&serde_json::Value>) -> [f32; 4] {
    let Some(value) = value else {
        return [0.0; 4];
    };
    if let Some(radius) = value.as_f64() {
        return [(radius as f32).max(0.0); 4];
    }
    let Some(values) = value.as_array() else {
        return [0.0; 4];
    };
    let numbers = values
        .iter()
        .filter_map(serde_json::Value::as_f64)
        .map(|value| (value as f32).max(0.0))
        .collect::<Vec<_>>();
    match numbers.as_slice() {
        [all] => [*all; 4],
        [top_left, top_right] => [*top_left, *top_right, *top_left, *top_right],
        [top_left, top_right, bottom_right] => [*top_left, *top_right, *bottom_right, *top_right],
        [top_left, top_right, bottom_right, bottom_left, ..] => {
            [*top_left, *top_right, *bottom_right, *bottom_left]
        }
        _ => [0.0; 4],
    }
}

pub(super) fn draw_timeline(
    canvas: Option<&Canvas>,
    option: &ChartOption,
    width: f32,
    height: f32,
    hits: &mut Vec<HitRegion>,
) {
    let Some(timeline) = option.timeline.as_ref().filter(|timeline| timeline.show) else {
        return;
    };
    let count = option.timeline_options.len().max(timeline.data.len());
    if count == 0 {
        return;
    }
    if timeline.orient == "vertical" {
        draw_vertical_timeline(canvas, timeline, count, width, height, hits);
    } else {
        draw_horizontal_timeline(canvas, timeline, count, width, height, hits);
    }
}

fn draw_horizontal_timeline(
    canvas: Option<&Canvas>,
    timeline: &Timeline,
    count: usize,
    width: f32,
    height: f32,
    hits: &mut Vec<HitRegion>,
) {
    let left = edge_value(&timeline.left, width, 40.0);
    let right = edge_value(&timeline.right, width, 40.0);
    let y = if !timeline.top.is_null() {
        edge_value(&timeline.top, height, height - 30.0)
    } else {
        height - edge_value(&timeline.bottom, height, 8.0) - 20.0
    };
    let controls = 58.0;
    let start = (left + controls).min(width - right);
    let end = (width - right).max(start);
    if let Some(canvas) = canvas {
        stroke_line(
            canvas,
            start,
            y,
            end,
            y,
            timeline.line_style.color.unwrap_or(0xFFCBD5E1),
            timeline.line_style.width,
        );
        let current_x = timeline_position(timeline, timeline.current_index, count, start, end);
        stroke_line(
            canvas,
            start,
            y,
            current_x,
            y,
            0xFF5470C6,
            timeline.line_style.width,
        );
    }
    for index in 0..count {
        let x = timeline_position(timeline, index, count, start, end);
        let current = index == timeline.current_index;
        let radius = if current { 6.0 } else { 4.0 };
        if let Some(canvas) = canvas {
            let style = if current {
                &timeline.checkpoint_style
            } else {
                &timeline.item_style
            };
            fill_circle(canvas, x, y, radius, style.color.unwrap_or(0xFFFFFFFF));
            stroke_circle(
                canvas,
                x,
                y,
                radius,
                style.border_color.unwrap_or(0xFF94A3B8),
                style.border_width.max(1.0),
            );
            if timeline.label.show {
                let label = timeline_label(timeline, index);
                let text_width = label.chars().count() as f32 * timeline.label.font_size * 0.56;
                draw_text(
                    canvas,
                    &label,
                    x - text_width / 2.0,
                    y + radius + timeline.label.distance + timeline.label.font_size,
                    timeline.label.font_size as f64,
                    timeline.label.color.unwrap_or(0xFF475569),
                    timeline.label.font_weight,
                );
            }
        }
        hits.push(timeline_hit(
            index,
            timeline_label(timeline, index),
            x,
            y,
            radius + 7.0,
        ));
    }
    draw_timeline_controls(canvas, timeline, left, y, count, hits);
}

fn draw_vertical_timeline(
    canvas: Option<&Canvas>,
    timeline: &Timeline,
    count: usize,
    width: f32,
    height: f32,
    hits: &mut Vec<HitRegion>,
) {
    let x = if !timeline.left.is_null() {
        edge_value(&timeline.left, width, 28.0)
    } else {
        width - edge_value(&timeline.right, width, 20.0)
    };
    let top = edge_value(&timeline.top, height, 30.0) + 48.0;
    let bottom = height - edge_value(&timeline.bottom, height, 30.0);
    if let Some(canvas) = canvas {
        stroke_line(
            canvas,
            x,
            top,
            x,
            bottom,
            timeline.line_style.color.unwrap_or(0xFFCBD5E1),
            timeline.line_style.width,
        );
    }
    for index in 0..count {
        let y = timeline_position(timeline, index, count, top, bottom);
        let current = index == timeline.current_index;
        let radius = if current { 6.0 } else { 4.0 };
        if let Some(canvas) = canvas {
            let style = if current {
                &timeline.checkpoint_style
            } else {
                &timeline.item_style
            };
            fill_circle(canvas, x, y, radius, style.color.unwrap_or(0xFFFFFFFF));
            stroke_circle(
                canvas,
                x,
                y,
                radius,
                style.border_color.unwrap_or(0xFF94A3B8),
                style.border_width.max(1.0),
            );
            if timeline.label.show {
                draw_text(
                    canvas,
                    &timeline_label(timeline, index),
                    x + radius + timeline.label.distance,
                    y + timeline.label.font_size / 2.0,
                    timeline.label.font_size as f64,
                    timeline.label.color.unwrap_or(0xFF475569),
                    timeline.label.font_weight,
                );
            }
        }
        hits.push(timeline_hit(
            index,
            timeline_label(timeline, index),
            x,
            y,
            radius + 7.0,
        ));
    }
    draw_timeline_controls(canvas, timeline, x - 27.0, top - 28.0, count, hits);
}

fn draw_timeline_controls(
    canvas: Option<&Canvas>,
    timeline: &Timeline,
    x: f32,
    y: f32,
    count: usize,
    hits: &mut Vec<HitRegion>,
) {
    let controls = [
        ("timeline-prev", "‹"),
        ("timeline-play", if timeline.auto_play { "Ⅱ" } else { "▶" }),
        ("timeline-next", "›"),
    ];
    for (index, (name, icon)) in controls.into_iter().enumerate() {
        let cx = x + 9.0 + index as f32 * 18.0;
        if let Some(canvas) = canvas {
            fill_circle(canvas, cx, y, 7.0, 0xFFF8FAFC);
            stroke_circle(
                canvas,
                cx,
                y,
                7.0,
                timeline.control_style.border_color.unwrap_or(0xFFCBD5E1),
                timeline.control_style.border_width.max(1.0),
            );
            draw_text(
                canvas,
                icon,
                cx - 3.5,
                y + 4.0,
                9.0,
                timeline.control_style.color.unwrap_or(0xFF475569),
                600,
            );
        }
        hits.push(timeline_hit(count + index, String::from(name), cx, y, 9.0));
    }
}

fn timeline_position(timeline: &Timeline, index: usize, count: usize, start: f32, end: f32) -> f32 {
    let index = if timeline.inverse {
        count - 1 - index
    } else {
        index
    };
    start + (end - start) * index as f32 / count.saturating_sub(1).max(1) as f32
}

fn timeline_label(timeline: &Timeline, index: usize) -> String {
    timeline
        .data
        .get(index)
        .cloned()
        .unwrap_or_else(|| index.to_string())
}

fn timeline_hit(index: usize, name: String, x: f32, y: f32, radius: f32) -> HitRegion {
    HitRegion {
        shape: HitShape::Point { x, y, radius },
        event: ChartEvent {
            series_index: 0,
            data_index: index,
            series_name: None,
            name: Some(name),
            value: vec![index as f64],
            x,
            y,
            component_type: String::from("timeline"),
        },
    }
}

fn edge_value(value: &serde_json::Value, total: f32, fallback: f32) -> f32 {
    value.as_f64().map(|value| value as f32).unwrap_or_else(|| {
        value
            .as_str()
            .and_then(|value| value.strip_suffix('%'))
            .and_then(|value| value.parse::<f32>().ok())
            .map(|value| total * value / 100.0)
            .unwrap_or(fallback)
    })
}

pub(super) fn draw_brush(
    canvas: Option<&Canvas>,
    option: &ChartOption,
    width: f32,
    height: f32,
    hits: &[HitRegion],
) {
    let Some(brush) = option.brush.as_ref() else {
        return;
    };
    let Some(canvas) = canvas else {
        return;
    };
    for area in &brush.areas {
        let (x, y, width, height) = brush_bounds(area, &brush.brush_type, width, height);
        fill_rect(
            canvas,
            x,
            y,
            width,
            height,
            brush.brush_style.color.unwrap_or(0x335470C6),
        );
        stroke_rect(
            canvas,
            x,
            y,
            width,
            height,
            brush.brush_style.border_color.unwrap_or(0xFF5470C6),
            brush.brush_style.border_width.max(1.0),
        );
        for hit in hits
            .iter()
            .filter(|hit| is_data_component(&hit.event.component_type))
        {
            if hit.event.x >= x
                && hit.event.x <= x + width
                && hit.event.y >= y
                && hit.event.y <= y + height
            {
                let color = brush.in_brush_color.unwrap_or(0xFF5470C6);
                stroke_circle(canvas, hit.event.x, hit.event.y, 6.0, color, 2.0);
            }
        }
    }
}

fn brush_bounds(
    area: &BrushArea,
    kind: &str,
    chart_width: f32,
    chart_height: f32,
) -> (f32, f32, f32, f32) {
    let mut x = area.start[0].min(area.end[0]);
    let mut y = area.start[1].min(area.end[1]);
    let mut width = (area.end[0] - area.start[0]).abs();
    let mut height = (area.end[1] - area.start[1]).abs();
    if kind == "lineX" {
        y = 0.0;
        height = chart_height;
    } else if kind == "lineY" {
        x = 0.0;
        width = chart_width;
    }
    (x, y, width.max(1.0), height.max(1.0))
}

fn is_data_component(component: &str) -> bool {
    !matches!(
        component,
        "legend" | "toolbox" | "dataZoom" | "timeline" | "brush"
    )
}

pub(super) fn draw_visual_map(canvas: &Canvas, option: &ChartOption, width: f32, height: f32) {
    let Some(visual_map) = option
        .visual_map
        .as_ref()
        .filter(|visual_map| visual_map.show)
    else {
        return;
    };
    if !visual_map.pieces.is_empty() {
        let row_height = 20.0;
        let content_height = visual_map.pieces.len() as f32 * row_height;
        let x = width - 92.0;
        let mut y = (height - content_height) / 2.0;
        for piece in &visual_map.pieces {
            let label =
                piece
                    .label
                    .clone()
                    .unwrap_or_else(|| match (piece.value, piece.min, piece.max) {
                        (Some(value), _, _) => format_value(value),
                        (_, Some(min), Some(max)) => {
                            format!("{} – {}", format_value(min), format_value(max))
                        }
                        (_, Some(min), None) => format!("≥ {}", format_value(min)),
                        (_, None, Some(max)) => format!("≤ {}", format_value(max)),
                        _ => String::from("other"),
                    });
            fill_rect(
                canvas,
                x,
                y + 3.0,
                14.0,
                14.0,
                piece.color.unwrap_or(option.visual_style.axis_color),
            );
            draw_text(
                canvas,
                &label,
                x + 20.0,
                y + 15.0,
                10.0,
                option.visual_style.text_color,
                400,
            );
            y += row_height;
        }
        return;
    }
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
    let mut label = option
        .tooltip
        .formatter
        .as_deref()
        .unwrap_or("{b}: {c}")
        .replace("{a}", event.series_name.as_deref().unwrap_or_default())
        .replace("{b}", name)
        .replace("{c}", &values);
    if let Some((percentage, precision)) = pie_percentage(option, event) {
        label = label.replace(
            "{d}",
            &format!("{percentage:.precision$}", precision = precision),
        );
    }
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

fn pie_percentage(option: &ChartOption, event: &ChartEvent) -> Option<(f64, usize)> {
    if event.component_type != "pie" {
        return None;
    }
    let Series::Pie(series) = option.series.get(event.series_index)? else {
        return None;
    };
    let value = series.data.get(event.data_index)?.number_opt(0)?.max(0.0);
    let total = series
        .data
        .iter()
        .filter_map(|point| point.number_opt(0))
        .map(|value| value.max(0.0))
        .sum::<f64>();
    let percentage = if total > f64::EPSILON {
        value / total * 100.0
    } else {
        100.0 / series.data.len().max(1) as f64
    };
    let precision = series
        .options
        .extra
        .get("percentPrecision")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(2)
        .min(20) as usize;
    Some((percentage, precision))
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
