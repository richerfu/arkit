use super::super::prelude::*;

pub(super) fn render(series: &BasicSeries, context: &mut CartesianRenderContext<'_>) {
    let series_index = context.series_index;
    let plot = context.plot;
    let layout = context.layout;
    let bar_layout = context
        .bar_layout
        .expect("bar renderer requires bar layout");
    let horizontal = layout.y.is_category() && !layout.x.is_category();
    let palette = context.palette;
    let canvas = context.canvas;
    let hits = &mut *context.hits;

    for (index, point) in series.data.iter().enumerate() {
        let paired = point.values.len() > 1;
        let value = if paired && !horizontal {
            point.number_opt(1)
        } else {
            point.number_opt(0)
        };
        let Some(value) = value else { continue };
        let category_value = if paired {
            point.number_opt(usize::from(horizontal))
        } else {
            None
        };
        let category_scale = if horizontal { &layout.y } else { &layout.x };
        if !category_scale.contains(category_value, index) {
            continue;
        }
        let (base_value, end_value) = context
            .stack
            .and_then(|stack| stack.get(index))
            .copied()
            .unwrap_or((0.0, value));
        let value_scale = if horizontal { &layout.x } else { &layout.y };
        if !value_scale.contains(Some(end_value), index)
            && !value_scale.contains(Some(base_value), index)
        {
            continue;
        }

        let category_center = category_scale.position(plot, category_value, index, horizontal);
        let base = value_scale.position(plot, Some(base_value), index, !horizontal);
        let end = value_scale.position(plot, Some(end_value), index, !horizontal);
        let end = enforce_minimum_extent(
            value_scale,
            plot,
            base,
            end,
            base_value,
            end_value,
            index,
            !horizontal,
            series.options.bar_min_height,
        );
        let bounds = if horizontal {
            (
                base.min(end),
                category_center + bar_layout.offset,
                (end - base).abs().max(1.0),
                bar_layout.width,
            )
        } else {
            (
                category_center + bar_layout.offset,
                base.min(end),
                bar_layout.width,
                (end - base).abs().max(1.0),
            )
        };

        if let Some(canvas) = canvas {
            if series.options.show_background {
                draw_background(canvas, series, plot, bounds, horizontal);
            }
            let style = effective_item_style(series, Some(point));
            fill_rounded_rect(
                canvas,
                bounds.0,
                bounds.1,
                bounds.2,
                bounds.3,
                style.border_radius,
                item_color(series, Some(point), palette, series_index),
            );
            if style.border_width > 0.0 {
                if let Some(border_color) = style.border_color {
                    stroke_rounded_rect(
                        canvas,
                        bounds,
                        style.border_radius,
                        with_opacity(border_color, style.opacity),
                        style.border_width,
                    );
                }
            }
            draw_label(
                canvas,
                series,
                point,
                index,
                bounds,
                horizontal,
                end_value >= base_value,
            );
        }
        hits.push(rect_hit(
            "bar",
            series_index,
            index,
            series.name.clone(),
            point,
            bounds,
        ));
    }
}

pub(super) fn render_polar(series: &BasicSeries, context: &mut FreeRenderContext<'_>) {
    let series_index = context.series_index;
    let polar_index = series
        .options
        .extra
        .get("polarIndex")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0) as usize;
    let config =
        super::line::PolarConfig::from_option(context.option, polar_index, context.plot, series);
    let band = config.angle_band();
    let gap = series
        .options
        .bar_category_gap
        .resolve(band.abs())
        .clamp(0.0, band.abs() * 0.95);
    let width = series
        .options
        .bar_width
        .map(|value| value.resolve(band.abs()))
        .unwrap_or(band.abs() - gap)
        .clamp(0.5_f32.to_radians(), band.abs());
    let sweep = width.copysign(band);
    let palette = context.palette;

    if let Some(canvas) = context.canvas {
        config.draw_axes(canvas, context.option.visual_style.text_color);
    }
    for (index, point) in series.data.iter().enumerate() {
        let paired = point.values.len() > 1;
        let Some(value) = point.number_opt(0) else {
            continue;
        };
        let angle_value = paired.then(|| point.number_opt(1)).flatten();
        let angle = config.angle_for(angle_value, index);
        let base_radius = config.zero_radius();
        let end_radius = config.radius_for(value, index);
        let inner = base_radius.min(end_radius);
        let outer = base_radius.max(end_radius).max(inner + 0.5);
        let start = angle - sweep / 2.0;
        let style = effective_item_style(series, Some(point));
        let fill = item_color(series, Some(point), palette, series_index);
        if let Some(canvas) = context.canvas {
            if series.options.show_background {
                let background = &series.options.background_style;
                fill_ring_sector(
                    canvas,
                    config.center,
                    (config.inner_radius, config.outer_radius),
                    start,
                    sweep,
                    with_opacity(background.color.unwrap_or(0x33B4B4B4), background.opacity),
                );
            }
            fill_ring_sector(canvas, config.center, (inner, outer), start, sweep, fill);
            if style.border_width > 0.0 {
                if let Some(border_color) = style.border_color {
                    stroke_ring_sector(
                        canvas,
                        config.center,
                        (inner, outer),
                        start,
                        sweep,
                        with_opacity(border_color, style.opacity),
                        style.border_width,
                    );
                }
            }
            let label = effective_label(series, point);
            if label.show {
                let label_radius = if matches!(label.position.as_str(), "inside" | "middle") {
                    (inner + outer) / 2.0
                } else {
                    outer + label.distance
                };
                let position = config.project(label_radius, angle);
                let text = format_label(&label, series, point, index);
                set_next_data_index(index);
                draw_text(
                    canvas,
                    &text,
                    position.0 - text.chars().count() as f32 * label.font_size * 0.28,
                    position.1 + label.font_size * 0.35,
                    label.font_size as f64,
                    label.color.unwrap_or(0xFF333333),
                    label.font_weight,
                );
            }
        }
        let middle = config.project((inner + outer) / 2.0, angle);
        context.hits.push(HitRegion {
            shape: HitShape::Sector {
                cx: config.center.0,
                cy: config.center.1,
                inner,
                outer,
                start: normalize_angle(start),
                sweep,
            },
            event: chart_event(
                "bar",
                series_index,
                index,
                series.name.clone(),
                point,
                middle.0,
                middle.1,
            ),
        });
    }
}

#[allow(clippy::too_many_arguments)]
fn enforce_minimum_extent(
    scale: &crate::render::scale::Scale,
    plot: &crate::render::geometry::Plot,
    base: f32,
    end: f32,
    base_value: f64,
    end_value: f64,
    index: usize,
    vertical: bool,
    minimum: f32,
) -> f32 {
    if minimum <= 0.0 || (end - base).abs() >= minimum {
        return end;
    }
    let logical_direction = if end_value < base_value { -1.0 } else { 1.0 };
    let probe =
        scale.position_unclamped(plot, Some(base_value + logical_direction), index, vertical);
    let direction = (probe - base).signum();
    base + if direction == 0.0 { 1.0 } else { direction } * minimum
}

fn draw_background(
    canvas: &ohos_drawing_binding::Canvas,
    series: &BasicSeries,
    plot: &crate::render::geometry::Plot,
    bounds: (f32, f32, f32, f32),
    horizontal: bool,
) {
    let style = &series.options.background_style;
    let background = if horizontal {
        (plot.x, bounds.1, plot.width, bounds.3)
    } else {
        (bounds.0, plot.y, bounds.2, plot.height)
    };
    let color = with_opacity(style.color.unwrap_or(0x33B4B4B4), style.opacity);
    fill_rounded_rect(
        canvas,
        background.0,
        background.1,
        background.2,
        background.3,
        style.border_radius,
        color,
    );
    if style.border_width > 0.0 {
        if let Some(border_color) = style.border_color {
            stroke_rounded_rect(
                canvas,
                background,
                style.border_radius,
                with_opacity(border_color, style.opacity),
                style.border_width,
            );
        }
    }
}

fn draw_label(
    canvas: &ohos_drawing_binding::Canvas,
    series: &BasicSeries,
    point: &DataPoint,
    index: usize,
    bounds: (f32, f32, f32, f32),
    horizontal: bool,
    positive: bool,
) {
    let label = effective_label(series, point);
    if !label.show {
        return;
    }
    set_next_data_index(index);
    let (x, y, width, height) = bounds;
    let distance = label.distance;
    let position = match label.position.as_str() {
        "outside" if horizontal && positive => "right",
        "outside" if horizontal => "left",
        "outside" if positive => "top",
        "outside" => "bottom",
        value => value,
    };
    let (text_x, text_y) = match position {
        "inside" => (x + width * 0.5, y + height * 0.5 + label.font_size * 0.5),
        "insideLeft" => (x + distance, y + height * 0.5 + label.font_size * 0.5),
        "insideRight" => (
            x + width - distance,
            y + height * 0.5 + label.font_size * 0.5,
        ),
        "insideTop" => (x + width * 0.5, y + distance + label.font_size),
        "insideBottom" => (x + width * 0.5, y + height - distance),
        "left" => (x - distance, y + height * 0.5 + label.font_size * 0.5),
        "right" => (
            x + width + distance,
            y + height * 0.5 + label.font_size * 0.5,
        ),
        "bottom" => (x + width * 0.5, y + height + distance + label.font_size),
        _ => (x + width * 0.5, y - distance),
    };
    draw_text(
        canvas,
        &format_label(&label, series, point, index),
        text_x + label.offset[0],
        text_y + label.offset[1],
        label.font_size as f64,
        label.color.unwrap_or(0xFF333333),
        label.font_weight,
    );
}
