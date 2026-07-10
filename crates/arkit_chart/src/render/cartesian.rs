//! Cartesian coordinate-system composition.

use ohos_drawing_binding::Canvas;

use super::geometry::Plot;
use super::hit::HitRegion;
use super::scale::{CartesianLayout, ScaleTick};
use super::series;
use super::surface::{draw_text, fill_rect, stroke_line};
use super::viewport::ZoomWindow;
use crate::model::{ChartEvent, ChartOption};

pub(super) struct CartesianChartRenderContext<'a> {
    pub(super) option: &'a ChartOption,
    pub(super) series_indices: &'a [usize],
    pub(super) plot: &'a Plot,
    pub(super) axis_indices: (usize, usize),
    pub(super) palette: &'a [u32],
    pub(super) canvas: Option<&'a Canvas>,
    pub(super) hits: &'a mut Vec<HitRegion>,
    pub(super) zoom_windows: &'a [ZoomWindow],
    pub(super) selected: Option<&'a ChartEvent>,
}

pub(super) fn render(context: CartesianChartRenderContext<'_>) {
    let CartesianChartRenderContext {
        option,
        series_indices,
        plot,
        axis_indices,
        palette,
        canvas,
        hits,
        zoom_windows,
        selected,
    } = context;
    let layout = CartesianLayout::collect(
        option,
        series_indices,
        axis_indices.0,
        axis_indices.1,
        zoom_windows,
    );
    if let Some(canvas) = canvas {
        draw_axes(canvas, option, plot, &layout);
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

fn draw_axes(canvas: &Canvas, option: &ChartOption, plot: &Plot, layout: &CartesianLayout) {
    let x_ticks = sampled_ticks(layout.x.ticks(), 7);
    let y_ticks = sampled_ticks(layout.y.ticks(), 6);

    if layout.y.draws_split_line() {
        for tick in &y_ticks {
            let y = layout.y.position(plot, Some(tick.value), tick.index, true);
            stroke_line(
                canvas,
                plot.x,
                y,
                plot.x + plot.width,
                y,
                option.visual_style.split_line_color,
                0.7,
            );
        }
    }
    if layout.x.draws_split_line() {
        for tick in &x_ticks {
            let x = layout.x.position(plot, Some(tick.value), tick.index, false);
            stroke_line(
                canvas,
                x,
                plot.y,
                x,
                plot.y + plot.height,
                option.visual_style.split_line_color,
                0.7,
            );
        }
    }

    if layout.y.is_visible() {
        stroke_line(
            canvas,
            plot.x,
            plot.y,
            plot.x,
            plot.y + plot.height,
            option.visual_style.axis_color,
            1.0,
        );
    }
    if layout.x.is_visible() {
        stroke_line(
            canvas,
            plot.x,
            plot.y + plot.height,
            plot.x + plot.width,
            plot.y + plot.height,
            option.visual_style.axis_color,
            1.0,
        );
    }

    if layout.x.draws_labels() {
        for tick in &x_ticks {
            let x = layout.x.position(plot, Some(tick.value), tick.index, false);
            draw_text(
                canvas,
                &tick.label,
                x - estimate_label_width(&tick.label, 10.0) / 2.0,
                plot.y + plot.height + 17.0,
                10.0,
                option.visual_style.text_color,
                400,
            );
        }
    }
    if layout.y.draws_labels() {
        for tick in &y_ticks {
            let y = layout.y.position(plot, Some(tick.value), tick.index, true);
            draw_text(
                canvas,
                &tick.label,
                plot.x - estimate_label_width(&tick.label, 10.0) - 7.0,
                y + 4.0,
                10.0,
                option.visual_style.text_color,
                400,
            );
        }
    }

    if let Some(name) = layout.x.name() {
        draw_text(
            canvas,
            name,
            plot.x + plot.width + 5.0,
            plot.y + plot.height + 4.0,
            10.0,
            option.visual_style.text_color,
            400,
        );
    }
    if let Some(name) = layout.y.name() {
        draw_text(
            canvas,
            name,
            plot.x + 4.0,
            plot.y - 7.0,
            10.0,
            option.visual_style.text_color,
            400,
        );
    }
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
}
