//! Cartesian coordinate-system composition.

use ohos_drawing_binding::Canvas;

use super::geometry::Plot;
use super::hit::HitRegion;
use super::scale::{CartesianLayout, Scale, ScaleTick};
use super::series;
use super::style::with_opacity;
use super::surface::{draw_rotated_text, draw_text, fill_rect, stroke_line};
use super::viewport::ZoomWindow;
use crate::model::{AxisLabelStyle, ChartEvent, ChartOption, LineStyle};

pub(super) struct CartesianChartRenderContext<'a> {
    pub(super) option: &'a ChartOption,
    pub(super) domain_option: &'a ChartOption,
    pub(super) series_indices: &'a [usize],
    pub(super) plot: &'a Plot,
    pub(super) axis_indices: (usize, usize),
    pub(super) palette: &'a [u32],
    pub(super) canvas: Option<&'a Canvas>,
    pub(super) hits: &'a mut Vec<HitRegion>,
    pub(super) zoom_windows: &'a [ZoomWindow],
    pub(super) selected: Option<&'a ChartEvent>,
    pub(super) draw_x_axis: bool,
    pub(super) draw_y_axis: bool,
}

pub(super) fn render(context: CartesianChartRenderContext<'_>) {
    let CartesianChartRenderContext {
        option,
        domain_option,
        series_indices,
        plot,
        axis_indices,
        palette,
        canvas,
        hits,
        zoom_windows,
        selected,
        draw_x_axis,
        draw_y_axis,
    } = context;
    let layout = CartesianLayout::collect(
        domain_option,
        series_indices,
        axis_indices.0,
        axis_indices.1,
        zoom_windows,
    );
    if let Some(canvas) = canvas {
        draw_axes(canvas, option, plot, &layout, draw_x_axis, draw_y_axis);
    }
    series::render_cartesian_set(option, series_indices, plot, &layout, palette, canvas, hits);
    if let (Some(canvas), Some(selected)) = (canvas, selected) {
        if series_indices.contains(&selected.series_index) {
            draw_axis_pointer(canvas, option, plot, &layout, selected);
        }
    }
}

fn draw_axis_pointer(
    canvas: &Canvas,
    option: &ChartOption,
    plot: &Plot,
    layout: &CartesianLayout,
    selected: &ChartEvent,
) {
    let pointer = &option.tooltip.axis_pointer;
    if option.tooltip.trigger != "axis" || !pointer.show {
        return;
    }
    let x = selected.x.clamp(plot.x, plot.x + plot.width);
    let y = selected.y.clamp(plot.y, plot.y + plot.height);
    let color = pointer.line_style.color.unwrap_or(0xFF777777);
    if pointer.kind == "shadow" {
        let width = layout
            .x
            .band_width(plot, false, layout.x.count().max(1))
            .max(2.0);
        fill_rect(
            canvas,
            x - width / 2.0,
            plot.y,
            width,
            plot.height,
            (0x22 << 24) | (color & 0x00FFFFFF),
        );
    } else {
        stroke_line(
            canvas,
            x,
            plot.y,
            x,
            plot.y + plot.height,
            color,
            pointer.line_style.width.max(0.5),
        );
        if pointer.kind == "cross" {
            stroke_line(
                canvas,
                plot.x,
                y,
                plot.x + plot.width,
                y,
                color,
                pointer.line_style.width.max(0.5),
            );
        }
    }
    if pointer.label.show {
        let label = layout
            .x
            .ticks()
            .into_iter()
            .find(|tick| tick.index == selected.data_index)
            .map(|tick| tick.label)
            .unwrap_or_else(|| selected.data_index.to_string());
        let label_width = estimate_label_width(&label, pointer.label.font_size) + 8.0;
        fill_rect(
            canvas,
            (x - label_width / 2.0).clamp(plot.x, plot.x + plot.width - label_width),
            plot.y + plot.height + 2.0,
            label_width,
            pointer.label.font_size + 6.0,
            color,
        );
        draw_text(
            canvas,
            &label,
            (x - label_width / 2.0 + 4.0).clamp(plot.x + 4.0, plot.x + plot.width - label_width),
            plot.y + plot.height + pointer.label.font_size + 4.0,
            pointer.label.font_size as f64,
            pointer.label.color.unwrap_or(0xFFFFFFFF),
            pointer.label.font_weight,
        );
    }
}

fn draw_axes(
    canvas: &Canvas,
    option: &ChartOption,
    plot: &Plot,
    layout: &CartesianLayout,
    draw_x_axis: bool,
    draw_y_axis: bool,
) {
    let x_scale_ticks = layout.x.ticks();
    let y_scale_ticks = layout.y.ticks();
    let x_ticks = visible_ticks(x_scale_ticks.clone(), layout.x.axis_label_style(), 7);
    let y_ticks = visible_ticks(y_scale_ticks.clone(), layout.y.axis_label_style(), 6);

    if draw_y_axis && layout.y.draws_split_line() {
        let (color, width) = resolved_line_style(
            layout.y.split_line_style(),
            option.visual_style.split_line_color,
        );
        for tick in &y_scale_ticks {
            let y = layout.y.position(plot, Some(tick.value), tick.index, true);
            stroke_line(canvas, plot.x, y, plot.x + plot.width, y, color, width);
        }
    }
    if draw_x_axis && layout.x.draws_split_line() {
        let (color, width) = resolved_line_style(
            layout.x.split_line_style(),
            option.visual_style.split_line_color,
        );
        for tick in &x_scale_ticks {
            let x = layout.x.position(plot, Some(tick.value), tick.index, false);
            stroke_line(canvas, x, plot.y, x, plot.y + plot.height, color, width);
        }
    }

    let x_axis_y = x_axis_coordinate(plot, layout);
    let y_axis_x = y_axis_coordinate(plot, layout);
    if draw_y_axis && layout.y.is_visible() {
        let (color, width) = resolved_line_style(
            &layout.y.axis_line().line_style,
            option.visual_style.axis_color,
        );
        stroke_line(
            canvas,
            y_axis_x,
            plot.y,
            y_axis_x,
            plot.y + plot.height,
            color,
            width,
        );
    }
    if draw_x_axis && layout.x.is_visible() {
        let (color, width) = resolved_line_style(
            &layout.x.axis_line().line_style,
            option.visual_style.axis_color,
        );
        stroke_line(
            canvas,
            plot.x,
            x_axis_y,
            plot.x + plot.width,
            x_axis_y,
            color,
            width,
        );
    }

    if draw_x_axis {
        draw_axis_ticks(canvas, option, plot, &layout.x, false, x_axis_y);
    }
    if draw_y_axis {
        draw_axis_ticks(canvas, option, plot, &layout.y, true, y_axis_x);
    }

    if draw_x_axis && layout.x.draws_labels() {
        let style = layout.x.axis_label_style();
        let bottom = layout.x.axis_position() != "top";
        for tick in &x_ticks {
            let x = layout.x.position(plot, Some(tick.value), tick.index, false);
            let label = format_axis_label(style, &tick.label);
            let width = estimate_label_width(&label, style.font_size);
            let baseline = if bottom {
                x_axis_y + style.margin + style.font_size
            } else {
                x_axis_y - style.margin
            };
            draw_rotated_text(
                canvas,
                &label,
                x - width / 2.0,
                baseline,
                x,
                baseline,
                style.rotate,
                style.font_size as f64,
                style.color.unwrap_or(option.visual_style.text_color),
                style.font_weight,
            );
        }
    }
    if draw_y_axis && layout.y.draws_labels() {
        let style = layout.y.axis_label_style();
        let right = layout.y.axis_position() == "right";
        for tick in &y_ticks {
            let y = layout.y.position(plot, Some(tick.value), tick.index, true);
            let label = format_axis_label(style, &tick.label);
            let width = estimate_label_width(&label, style.font_size);
            let x = if right {
                y_axis_x + style.margin
            } else {
                y_axis_x - style.margin - width
            };
            let baseline = y + style.font_size * 0.35;
            draw_rotated_text(
                canvas,
                &label,
                x,
                baseline,
                if right { x } else { x + width },
                baseline,
                style.rotate,
                style.font_size as f64,
                style.color.unwrap_or(option.visual_style.text_color),
                style.font_weight,
            );
        }
    }

    if let Some(name) = draw_x_axis.then(|| layout.x.name()).flatten() {
        let top = layout.x.axis_position() == "top";
        draw_text(
            canvas,
            name,
            plot.x + plot.width + 5.0,
            if top { x_axis_y - 4.0 } else { x_axis_y + 12.0 },
            10.0,
            option.visual_style.text_color,
            400,
        );
    }
    if let Some(name) = draw_y_axis.then(|| layout.y.name()).flatten() {
        let right = layout.y.axis_position() == "right";
        draw_text(
            canvas,
            name,
            if right {
                y_axis_x + 4.0
            } else {
                y_axis_x - estimate_label_width(name, 10.0)
            },
            plot.y - 7.0,
            10.0,
            option.visual_style.text_color,
            400,
        );
    }
}

fn x_axis_coordinate(plot: &Plot, layout: &CartesianLayout) -> f32 {
    if layout.x.axis_line().on_zero
        && layout.x.offset() <= f32::EPSILON
        && !layout.y.is_category()
        && layout.y.contains(Some(0.0), 0)
    {
        return layout.y.position(plot, Some(0.0), 0, true);
    }
    if layout.x.axis_position() == "top" {
        plot.y - layout.x.offset()
    } else {
        plot.y + plot.height + layout.x.offset()
    }
}

fn y_axis_coordinate(plot: &Plot, layout: &CartesianLayout) -> f32 {
    if layout.y.axis_line().on_zero
        && layout.y.offset() <= f32::EPSILON
        && !layout.x.is_category()
        && layout.x.contains(Some(0.0), 0)
    {
        return layout.x.position(plot, Some(0.0), 0, false);
    }
    if layout.y.axis_position() == "right" {
        plot.x + plot.width + layout.y.offset()
    } else {
        plot.x - layout.y.offset()
    }
}

fn draw_axis_ticks(
    canvas: &Canvas,
    option: &ChartOption,
    plot: &Plot,
    scale: &Scale,
    vertical: bool,
    coordinate: f32,
) {
    if !scale.draws_ticks() {
        return;
    }
    let tick = scale.axis_tick();
    let outside_direction = if vertical {
        if scale.axis_position() == "right" {
            1.0
        } else {
            -1.0
        }
    } else if scale.axis_position() == "top" {
        -1.0
    } else {
        1.0
    };
    let direction = if tick.inside {
        -outside_direction
    } else {
        outside_direction
    };
    let (color, width) = resolved_line_style(&tick.line_style, option.visual_style.axis_color);
    for position in scale.tick_positions(plot, vertical) {
        if vertical {
            stroke_line(
                canvas,
                coordinate,
                position,
                coordinate + direction * tick.length,
                position,
                color,
                width,
            );
        } else {
            stroke_line(
                canvas,
                position,
                coordinate,
                position,
                coordinate + direction * tick.length,
                color,
                width,
            );
        }
    }
}

fn resolved_line_style(style: &LineStyle, fallback: u32) -> (u32, f32) {
    (
        with_opacity(style.color.unwrap_or(fallback), style.opacity),
        style.width.max(0.5),
    )
}

fn visible_ticks(
    ticks: Vec<ScaleTick>,
    style: &AxisLabelStyle,
    automatic_limit: usize,
) -> Vec<ScaleTick> {
    if let Some(interval) = style.interval {
        let step = interval.saturating_add(1);
        return ticks
            .into_iter()
            .enumerate()
            .filter_map(|(index, tick)| (index % step == 0).then_some(tick))
            .collect();
    }
    sampled_ticks(ticks, automatic_limit)
}

fn format_axis_label(style: &AxisLabelStyle, value: &str) -> String {
    style
        .formatter
        .as_deref()
        .unwrap_or("{value}")
        .replace("{value}", value)
}

fn sampled_ticks(ticks: Vec<ScaleTick>, limit: usize) -> Vec<ScaleTick> {
    if ticks.len() <= limit {
        return ticks;
    }
    let step = (ticks.len() - 1).div_ceil(limit.saturating_sub(1).max(1));
    let last = ticks.len() - 1;
    ticks
        .into_iter()
        .enumerate()
        .filter_map(|(index, tick)| ((index % step == 0) || index == last).then_some(tick))
        .collect()
}

fn estimate_label_width(text: &str, font_size: f32) -> f32 {
    text.chars().count() as f32 * font_size * 0.56
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn category_tick_sampling_keeps_both_ends() {
        let ticks = (0..20)
            .map(|index| ScaleTick {
                value: index as f64,
                index,
                label: index.to_string(),
            })
            .collect();
        let sampled = sampled_ticks(ticks, 6);
        assert_eq!(sampled.first().unwrap().index, 0);
        assert_eq!(sampled.last().unwrap().index, 19);
        assert!(sampled.len() <= 7);
    }

    #[test]
    fn explicit_axis_label_interval_zero_keeps_every_tick() {
        let ticks = (0..12)
            .map(|index| ScaleTick {
                value: index as f64,
                index,
                label: index.to_string(),
            })
            .collect();
        let style = AxisLabelStyle {
            interval: Some(0),
            ..AxisLabelStyle::default()
        };
        assert_eq!(visible_ticks(ticks, &style, 6).len(), 12);
    }

    #[test]
    fn axis_label_formatter_replaces_echarts_value_token() {
        let style = AxisLabelStyle {
            formatter: Some(String::from("{value} °C")),
            ..AxisLabelStyle::default()
        };
        assert_eq!(format_axis_label(&style, "25"), "25 °C");
    }
}
