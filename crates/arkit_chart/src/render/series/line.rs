use ohos_drawing_binding::{Canvas, Path, Rect};

use super::super::prelude::*;
use super::super::symbol::{draw_symbol, resolve_symbol};
use crate::render::label_layout::draw_rotated_text;
use crate::render::surface::stroke_path_style;

type ScreenPoint<'a> = (usize, &'a DataPoint, f32, f32);

pub(super) fn render(series: &BasicSeries, context: &mut CartesianRenderContext<'_>) {
    let series_index = context.series_index;
    let plot = context.plot;
    let layout = context.layout;
    let palette = context.palette;

    let points: Vec<ScreenPoint<'_>> = series
        .data
        .iter()
        .enumerate()
        .filter_map(|(index, point)| {
            let paired = point.values.len() > 1;
            let x_value = if !layout.x.is_category() && paired {
                Some(point.number_opt(0)?)
            } else {
                None
            };
            if layout.x.is_category() && !layout.x.contains(x_value, index) {
                return None;
            }
            let raw_y = if paired {
                point.number_opt(1)?
            } else {
                point.number_opt(0)?
            };
            let y_value = context
                .stack
                .and_then(|stack| stack.get(index))
                .map(|(_, end)| *end)
                .unwrap_or(raw_y);
            let x = layout.x.position_unclamped(plot, x_value, index, false);
            let y = layout
                .y
                .position_unclamped(plot, Some(y_value), index, true);
            Some((index, point, x, y))
        })
        .collect();
    let segments = split_segments(&points, series.options.connect_nulls);

    if let Some(canvas) = context.canvas {
        if series.options.clip {
            begin_clip(canvas, plot);
        }

        draw_lines_and_areas(canvas, series, context, &segments);
        draw_symbols(canvas, series, palette, series_index, plot, &points);

        if series.options.clip {
            canvas.restore();
        }
        draw_labels(canvas, series, &points);
        draw_end_label(canvas, series, &points);
    }

    let hits = &mut *context.hits;
    for (index, point, x, y) in points {
        if !inside_plot(plot, x, y) {
            continue;
        }
        let symbol = resolve_symbol(series, point, None);
        let center = symbol.center(x, y);
        hits.push(point_hit(
            "line",
            series_index,
            index,
            series.name.clone(),
            point,
            center,
            symbol.hit_radius().max(8.0),
        ));
    }
}

fn split_segments<'a>(
    points: &[ScreenPoint<'a>],
    connect_nulls: bool,
) -> Vec<Vec<ScreenPoint<'a>>> {
    let mut segments: Vec<Vec<ScreenPoint<'a>>> = Vec::new();
    for point in points {
        let previous = segments
            .last()
            .and_then(|segment| segment.last())
            .map(|point| point.0);
        if starts_new_segment(previous, point.0, connect_nulls) || segments.is_empty() {
            segments.push(Vec::new());
        }
        segments.last_mut().unwrap().push(*point);
    }
    segments
}

fn draw_lines_and_areas(
    canvas: &Canvas,
    series: &BasicSeries,
    context: &CartesianRenderContext<'_>,
    segments: &[Vec<ScreenPoint<'_>>],
) {
    for segment in segments {
        let raw_points: Vec<(f32, f32)> = segment.iter().map(|(_, _, x, y)| (*x, *y)).collect();
        let sampled = sample_polyline(
            &raw_points,
            &series.options.sampling,
            context.plot.width,
            &context.layout.y,
            context.plot,
        );
        let curve = line_polyline(
            &sampled,
            series.options.step.as_deref(),
            series.options.smooth,
            series.options.smooth_monotone.as_deref(),
        );
        let baseline_points: Vec<(f32, f32)> = segment
            .iter()
            .map(|(index, _, x, _)| {
                let baseline = context
                    .stack
                    .and_then(|stack| stack.get(*index))
                    .map(|(base, _)| {
                        context
                            .layout
                            .y
                            .position_unclamped(context.plot, Some(*base), *index, true)
                    })
                    .unwrap_or_else(|| area_origin_position(series, context, *index));
                (*x, baseline)
            })
            .collect();
        let baseline_curve = line_polyline(
            &baseline_points,
            series.options.step.as_deref(),
            series.options.smooth,
            series.options.smooth_monotone.as_deref(),
        );

        if let (Some(fill), Some(first)) = (
            area_color(series, context.palette, context.series_index),
            curve.first(),
        ) {
            let mut area = Path::new();
            let first_baseline = baseline_curve
                .first()
                .map(|point| point.1)
                .unwrap_or_else(|| context.layout.y.zero_position(context.plot, true));
            area.move_to(first.0, first_baseline);
            for (x, y) in &curve {
                area.line_to(*x, *y);
            }
            for (x, y) in baseline_curve.iter().rev() {
                area.line_to(*x, *y);
            }
            area.close();
            fill_path(canvas, &area, fill);
        }

        if curve.len() >= 2 && series.options.line_style.width > 0.0 {
            let mut path = Path::new();
            for (index, (x, y)) in curve.iter().enumerate() {
                if index == 0 {
                    path.move_to(*x, *y);
                } else {
                    path.line_to(*x, *y);
                }
            }
            stroke_path_style(
                canvas,
                &path,
                line_color(series, context.palette, context.series_index),
                series.options.line_style.width,
                &series.options.line_style.kind,
            );
        }
    }
}

fn area_origin_position(
    series: &BasicSeries,
    context: &CartesianRenderContext<'_>,
    index: usize,
) -> f32 {
    if let Some(value) = series.options.area_origin.as_f64() {
        return context
            .layout
            .y
            .position_unclamped(context.plot, Some(value), index, true);
    }
    match series.options.area_origin.as_str().unwrap_or("auto") {
        "start" => context.layout.y.extent_position(context.plot, true, true),
        "end" => context.layout.y.extent_position(context.plot, false, true),
        _ => context.layout.y.zero_position(context.plot, true),
    }
}

fn draw_symbols(
    canvas: &Canvas,
    series: &BasicSeries,
    palette: &[u32],
    series_index: usize,
    plot: &crate::render::geometry::Plot,
    points: &[ScreenPoint<'_>],
) {
    let stride = symbol_stride(series, points, plot.width);
    for (index, point, x, y) in points {
        if series.options.show_symbol && *index % stride == 0 {
            let symbol = resolve_symbol(series, point, None);
            draw_symbol(
                canvas,
                &symbol,
                *x,
                *y,
                item_color(series, Some(point), palette, series_index),
                border(series, Some(point)),
            );
        }
    }
}

fn draw_labels(canvas: &Canvas, series: &BasicSeries, points: &[ScreenPoint<'_>]) {
    for (index, point, x, y) in points {
        let label = effective_label(series, point);
        if label.show {
            set_next_data_index(*index);
            draw_point_label(
                canvas,
                &format_label(&label, series, point, *index),
                &label,
                *x,
                *y,
            );
        }
    }
}

fn draw_end_label(canvas: &Canvas, series: &BasicSeries, points: &[ScreenPoint<'_>]) {
    let label = &series.options.end_label;
    let Some((index, point, x, y)) = points.last() else {
        return;
    };
    if !label.show {
        return;
    }
    set_next_data_index(*index);
    draw_point_label(
        canvas,
        &format_label(label, series, point, *index),
        label,
        *x,
        *y,
    );
}

fn begin_clip(canvas: &Canvas, plot: &crate::render::geometry::Plot) {
    canvas.save();
    let rect = Rect::new(plot.x, plot.y, plot.x + plot.width, plot.y + plot.height);
    // SAFETY: canvas and rect are live for the synchronous clip call; the
    // matching end-clip path restores the saved canvas state.
    unsafe {
        ohos_native_drawing_sys::OH_Drawing_CanvasClipRect(
            canvas.as_ptr(),
            rect.as_ptr(),
            ohos_native_drawing_sys::OH_Drawing_CanvasClipOp_INTERSECT,
            true,
        );
    }
}

fn line_polyline(
    points: &[(f32, f32)],
    step: Option<&str>,
    smooth: f32,
    smooth_monotone: Option<&str>,
) -> Vec<(f32, f32)> {
    if let Some(step) = step {
        step_polyline(points, step)
    } else {
        smooth_monotone_polyline(points, smooth, smooth_monotone)
    }
}

fn smooth_monotone_polyline(
    points: &[(f32, f32)],
    smooth: f32,
    monotone: Option<&str>,
) -> Vec<(f32, f32)> {
    let mut output = smooth_polyline(points, smooth);
    let Some(monotone) = monotone else {
        return output;
    };
    if points.len() < 3 || smooth <= 0.0 {
        return output;
    }
    let samples = (4.0 + smooth.clamp(0.0, 1.0) * 8.0).round() as usize;
    for (segment, pair) in points.windows(2).enumerate() {
        let start = segment * samples + 1;
        let end = (start + samples).min(output.len());
        for point in &mut output[start..end] {
            if monotone == "x" {
                point.1 = point
                    .1
                    .clamp(pair[0].1.min(pair[1].1), pair[0].1.max(pair[1].1));
            } else if monotone == "y" {
                point.0 = point
                    .0
                    .clamp(pair[0].0.min(pair[1].0), pair[0].0.max(pair[1].0));
            }
        }
    }
    output
}

fn step_polyline(points: &[(f32, f32)], step: &str) -> Vec<(f32, f32)> {
    let Some(first) = points.first().copied() else {
        return Vec::new();
    };
    let mut output = vec![first];
    for pair in points.windows(2) {
        let (left, right) = (pair[0], pair[1]);
        match step {
            "end" => output.push((right.0, left.1)),
            "middle" => {
                let middle = (left.0 + right.0) / 2.0;
                output.push((middle, left.1));
                output.push((middle, right.1));
            }
            _ => output.push((left.0, right.1)),
        }
        output.push(right);
    }
    output
}

fn sample_polyline(
    points: &[(f32, f32)],
    mode: &str,
    width: f32,
    y_scale: &crate::render::scale::Scale,
    plot: &crate::render::geometry::Plot,
) -> Vec<(f32, f32)> {
    let target = width.max(2.0).round() as usize;
    if mode == "none" || points.len() <= target * 2 {
        return points.to_vec();
    }
    if mode == "lttb" {
        return lttb(points, target);
    }
    let bucket_size = (points.len() as f32 / target as f32).ceil() as usize;
    let mut output = Vec::with_capacity(target + 2);
    for bucket in points.chunks(bucket_size) {
        let selected = match mode {
            "min" => bucket
                .iter()
                .min_by(|left, right| {
                    y_scale
                        .value_at_position(plot, left.1, true)
                        .total_cmp(&y_scale.value_at_position(plot, right.1, true))
                })
                .copied(),
            "max" => bucket
                .iter()
                .max_by(|left, right| {
                    y_scale
                        .value_at_position(plot, left.1, true)
                        .total_cmp(&y_scale.value_at_position(plot, right.1, true))
                })
                .copied(),
            "average" | "sum" => {
                let x = bucket.iter().map(|point| point.0).sum::<f32>() / bucket.len() as f32;
                let value = if mode == "sum" {
                    bucket
                        .iter()
                        .map(|point| y_scale.value_at_position(plot, point.1, true))
                        .sum::<f64>()
                } else {
                    bucket
                        .iter()
                        .map(|point| y_scale.value_at_position(plot, point.1, true))
                        .sum::<f64>()
                        / bucket.len() as f64
                };
                let y = y_scale.position_unclamped(plot, Some(value), 0, true);
                Some((x, y))
            }
            _ => bucket.first().copied(),
        };
        if let Some(point) = selected {
            output.push(point);
        }
    }
    output
}

fn lttb(points: &[(f32, f32)], threshold: usize) -> Vec<(f32, f32)> {
    if threshold >= points.len() || threshold < 3 {
        return points.to_vec();
    }
    let every = (points.len() - 2) as f32 / (threshold - 2) as f32;
    let mut sampled = Vec::with_capacity(threshold);
    let mut selected = 0usize;
    sampled.push(points[selected]);
    for bucket in 0..threshold - 2 {
        let avg_start = ((bucket + 1) as f32 * every).floor() as usize + 1;
        let avg_end = (((bucket + 2) as f32 * every).floor() as usize + 1).min(points.len());
        let avg_slice =
            &points[avg_start.min(points.len() - 1)..avg_end.max(avg_start + 1).min(points.len())];
        let average = (
            avg_slice.iter().map(|point| point.0).sum::<f32>() / avg_slice.len() as f32,
            avg_slice.iter().map(|point| point.1).sum::<f32>() / avg_slice.len() as f32,
        );
        let range_start = (bucket as f32 * every).floor() as usize + 1;
        let range_end = (((bucket + 1) as f32 * every).floor() as usize + 1).min(points.len() - 1);
        let anchor = points[selected];
        selected = (range_start..range_end.max(range_start + 1))
            .max_by(|left, right| {
                triangle_area(anchor, points[*left], average).total_cmp(&triangle_area(
                    anchor,
                    points[*right],
                    average,
                ))
            })
            .unwrap_or(range_start)
            .min(points.len() - 2);
        sampled.push(points[selected]);
    }
    sampled.push(*points.last().unwrap());
    sampled
}

fn triangle_area(a: (f32, f32), b: (f32, f32), c: (f32, f32)) -> f32 {
    ((a.0 - c.0) * (b.1 - a.1) - (a.0 - b.0) * (c.1 - a.1)).abs() * 0.5
}

fn symbol_stride(series: &BasicSeries, points: &[ScreenPoint<'_>], width: f32) -> usize {
    if series.options.show_all_symbol == Some(true) || points.len() < 2 {
        return 1;
    }
    let max_size = series
        .options
        .symbol_size_dimensions
        .map(|size| size[0].max(size[1]))
        .unwrap_or(series.options.symbol_size);
    let available = (width / max_size.max(6.0)).floor() as usize;
    points.len().div_ceil(available.max(1)).max(1)
}

fn draw_point_label(canvas: &Canvas, text: &str, label: &LabelStyle, x: f32, y: f32) {
    let width = text.chars().count() as f32 * label.font_size * 0.55;
    let distance = label.distance;
    let (mut x, mut y) = match label.position.as_str() {
        "bottom" => (x - width / 2.0, y + label.font_size + distance),
        "left" => (x - width - distance, y + label.font_size * 0.35),
        "right" => (x + distance, y + label.font_size * 0.35),
        "inside" => (x - width / 2.0, y + label.font_size * 0.35),
        "insideBottom" | "insideBottomLeft" | "insideBottomRight" => {
            (x - width / 2.0, y + label.font_size + distance)
        }
        "insideLeft" => (x + distance, y + label.font_size * 0.35),
        "insideRight" => (x - width - distance, y + label.font_size * 0.35),
        _ => (x - width / 2.0, y - distance),
    };
    x += label.offset[0];
    y += label.offset[1];
    draw_rotated_text(
        canvas,
        text,
        x,
        y,
        x,
        y,
        label.rotate,
        label.font_size as f64,
        label.color.unwrap_or(0xFF333333),
        label.font_weight,
    );
}

pub(super) fn render_polar(series: &BasicSeries, context: &mut FreeRenderContext<'_>) {
    let polar_index = series
        .options
        .extra
        .get("polarIndex")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0) as usize;
    let config = PolarConfig::from_option(context.option, polar_index, context.plot, series);
    let points = series
        .data
        .iter()
        .enumerate()
        .filter_map(|(index, point)| {
            let radius_value = point.number_opt(0)?;
            let angle_value = (point.values.len() > 1)
                .then(|| point.number_opt(1))
                .flatten();
            let radius = config.radius_for(radius_value, index);
            let angle = config.angle_for(angle_value, index);
            let position = config.project(radius, angle);
            Some((index, point, position.0, position.1))
        })
        .collect::<Vec<_>>();
    let segments = split_segments(&points, series.options.connect_nulls);

    if let Some(canvas) = context.canvas {
        if first_polar_line(context.option, context.series_index, polar_index) {
            config.draw_axes(canvas, context.option.visual_style.text_color);
        }
        if series.options.clip {
            begin_polar_clip(canvas, &config);
        }
        for segment in &segments {
            draw_polar_segment(
                canvas,
                series,
                context.palette,
                context.series_index,
                &config,
                segment,
            );
        }
        draw_symbols(
            canvas,
            series,
            context.palette,
            context.series_index,
            &context.plot,
            &points,
        );
        if series.options.clip {
            canvas.restore();
        }
        draw_labels(canvas, series, &points);
        draw_end_label(canvas, series, &points);
    }

    for (index, point, x, y) in points {
        let symbol = resolve_symbol(series, point, None);
        context.hits.push(point_hit(
            "line",
            context.series_index,
            index,
            series.name.clone(),
            point,
            symbol.center(x, y),
            symbol.hit_radius().max(8.0),
        ));
    }
}

fn first_polar_line(option: &ChartOption, series_index: usize, polar_index: usize) -> bool {
    !option.series[..series_index].iter().any(|series| {
        let Series::Line(series) = series else {
            return false;
        };
        series
            .options
            .extra
            .get("coordinateSystem")
            .and_then(serde_json::Value::as_str)
            == Some("polar")
            && series
                .options
                .extra
                .get("polarIndex")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0) as usize
                == polar_index
    })
}

fn begin_polar_clip(canvas: &Canvas, config: &PolarConfig) {
    canvas.save();
    let mut clip = Path::new();
    clip.add_circle(
        config.center.0,
        config.center.1,
        config.outer_radius,
        ohos_native_drawing_sys::OH_Drawing_PathDirection_PATH_DIRECTION_CW,
    );
    // SAFETY: canvas and clip path are live for this synchronous call; the
    // saved canvas state is restored by the matching polar clip teardown.
    unsafe {
        ohos_native_drawing_sys::OH_Drawing_CanvasClipPath(
            canvas.as_ptr(),
            clip.as_ptr(),
            ohos_native_drawing_sys::OH_Drawing_CanvasClipOp_INTERSECT,
            true,
        );
    }
}

fn draw_polar_segment(
    canvas: &Canvas,
    series: &BasicSeries,
    palette: &[u32],
    series_index: usize,
    config: &PolarConfig,
    segment: &[ScreenPoint<'_>],
) {
    let screen = segment
        .iter()
        .map(|(_, _, x, y)| (*x, *y))
        .collect::<Vec<_>>();
    let curve = smooth_monotone_polyline(
        &screen,
        series.options.smooth,
        series.options.smooth_monotone.as_deref(),
    );
    if let (Some(fill), Some(first)) = (area_color(series, palette, series_index), curve.first()) {
        let mut area = Path::new();
        let first_angle = config.screen_angle(*first);
        let first_baseline = config.project(config.zero_radius(), first_angle);
        area.move_to(first_baseline.0, first_baseline.1);
        for (x, y) in &curve {
            area.line_to(*x, *y);
        }
        for point in segment.iter().rev() {
            let angle = config.screen_angle((point.2, point.3));
            let baseline = config.project(config.zero_radius(), angle);
            area.line_to(baseline.0, baseline.1);
        }
        area.close();
        fill_path(canvas, &area, fill);
    }
    if curve.len() >= 2 && series.options.line_style.width > 0.0 {
        let mut path = Path::new();
        for (index, point) in curve.iter().enumerate() {
            if index == 0 {
                path.move_to(point.0, point.1);
            } else {
                path.line_to(point.0, point.1);
            }
        }
        stroke_path_style(
            canvas,
            &path,
            line_color(series, palette, series_index),
            series.options.line_style.width,
            &series.options.line_style.kind,
        );
    }
}

#[derive(Debug, Clone)]
pub(super) struct PolarConfig {
    pub(super) center: (f32, f32),
    pub(super) inner_radius: f32,
    pub(super) outer_radius: f32,
    start_angle: f32,
    clockwise: bool,
    angle: PolarScale,
    radius: PolarScale,
}

impl PolarConfig {
    pub(super) fn from_option(
        option: &ChartOption,
        polar_index: usize,
        plot: crate::render::geometry::Plot,
        series: &BasicSeries,
    ) -> Self {
        let polar = option_component(&option.extra, "polar", polar_index);
        let center_values = polar
            .and_then(|value| value.get("center"))
            .and_then(serde_json::Value::as_array);
        let center = (
            resolve_polar_length(
                center_values.and_then(|values| values.first()),
                plot.width,
                plot.x,
                0.5,
            ),
            resolve_polar_length(
                center_values.and_then(|values| values.get(1)),
                plot.height,
                plot.y,
                0.5,
            ),
        );
        let max_radius = plot.width.min(plot.height) / 2.0;
        let radius_value = polar.and_then(|value| value.get("radius"));
        let (inner_radius, outer_radius) = match radius_value.and_then(serde_json::Value::as_array)
        {
            Some(values) => (
                resolve_polar_radius(values.first(), max_radius, 0.0),
                resolve_polar_radius(values.get(1), max_radius, 0.75),
            ),
            None => (0.0, resolve_polar_radius(radius_value, max_radius, 0.75)),
        };
        let angle_axis = matching_polar_axis(&option.extra, "angleAxis", polar_index);
        let radius_axis = matching_polar_axis(&option.extra, "radiusAxis", polar_index);
        let radius_values = series
            .data
            .iter()
            .filter_map(|point| point.number_opt(0))
            .collect::<Vec<_>>();
        let angle_values = series
            .data
            .iter()
            .enumerate()
            .filter_map(|(index, point)| {
                if point.values.len() > 1 {
                    point.number_opt(1)
                } else {
                    Some(index as f64)
                }
            })
            .collect::<Vec<_>>();
        Self {
            center,
            inner_radius: inner_radius.min(outer_radius),
            outer_radius: outer_radius.max(inner_radius),
            start_angle: angle_axis
                .and_then(|axis| axis.get("startAngle"))
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(90.0) as f32,
            clockwise: angle_axis
                .and_then(|axis| axis.get("clockwise"))
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(true),
            angle: PolarScale::new(angle_axis, &angle_values, series.data.len(), false),
            radius: PolarScale::new(radius_axis, &radius_values, series.data.len(), true),
        }
    }

    pub(super) fn angle_for(&self, value: Option<f64>, index: usize) -> f32 {
        let normalized = self.angle.normalize(value, index);
        self.angle_from_normalized(normalized)
    }

    fn axis_angle(&self, index: usize) -> f32 {
        let normalized = if self.angle.labels.is_empty() {
            index as f32 / self.angle.tick_count() as f32
        } else {
            self.angle.normalize(None, index)
        };
        self.angle_from_normalized(normalized)
    }

    fn angle_from_normalized(&self, normalized: f32) -> f32 {
        let direction = if self.clockwise { 1.0 } else { -1.0 };
        -self.start_angle.to_radians() + direction * normalized * TAU
    }

    pub(super) fn radius_for(&self, value: f64, index: usize) -> f32 {
        self.inner_radius
            + (self.outer_radius - self.inner_radius) * self.radius.normalize(Some(value), index)
    }

    pub(super) fn zero_radius(&self) -> f32 {
        self.inner_radius
            + (self.outer_radius - self.inner_radius) * self.radius.normalize(Some(0.0), 0)
    }

    pub(super) fn project(&self, radius: f32, angle: f32) -> (f32, f32) {
        (
            self.center.0 + angle.cos() * radius,
            self.center.1 + angle.sin() * radius,
        )
    }

    fn screen_angle(&self, point: (f32, f32)) -> f32 {
        (point.1 - self.center.1).atan2(point.0 - self.center.0)
    }

    pub(super) fn draw_axes(&self, canvas: &Canvas, text_color: u32) {
        if self.radius.split_line {
            for split in 1..=self.radius.split_number {
                let radius = self.inner_radius
                    + (self.outer_radius - self.inner_radius) * split as f32
                        / self.radius.split_number as f32;
                stroke_circle(
                    canvas,
                    self.center.0,
                    self.center.1,
                    radius,
                    0xFFE0E6F1,
                    1.0,
                );
            }
        }
        let angle_count = self.angle.tick_count();
        if self.angle.split_line {
            for index in 0..angle_count {
                let angle = self.axis_angle(index);
                let inner = self.project(self.inner_radius, angle);
                let outer = self.project(self.outer_radius, angle);
                stroke_line(canvas, inner.0, inner.1, outer.0, outer.1, 0xFFE0E6F1, 1.0);
            }
        }
        if self.angle.show_labels {
            for index in 0..angle_count {
                let angle = self.axis_angle(index);
                let position = self.project(self.outer_radius + 12.0, angle);
                let label = self.angle.label(index);
                draw_text(
                    canvas,
                    &label,
                    position.0 - label.chars().count() as f32 * 3.0,
                    position.1 + 4.0,
                    11.0,
                    text_color,
                    400,
                );
            }
        }
        if self.radius.show_labels {
            for split in 1..=self.radius.split_number {
                let normalized = split as f32 / self.radius.split_number as f32;
                let radius =
                    self.inner_radius + (self.outer_radius - self.inner_radius) * normalized;
                let position = self.project(radius, -self.start_angle.to_radians());
                draw_text(
                    canvas,
                    &self.radius.value_label(normalized),
                    position.0 + 3.0,
                    position.1,
                    10.0,
                    text_color,
                    400,
                );
            }
        }
    }

    pub(super) fn angle_band(&self) -> f32 {
        TAU / self.angle.tick_count().max(1) as f32 * if self.clockwise { 1.0 } else { -1.0 }
    }
}

#[derive(Debug, Clone)]
struct PolarScale {
    labels: Vec<String>,
    min: f64,
    max: f64,
    inverse: bool,
    split_number: usize,
    split_line: bool,
    show_labels: bool,
}

impl PolarScale {
    fn new(
        axis: Option<&serde_json::Map<String, serde_json::Value>>,
        values: &[f64],
        inferred_count: usize,
        include_zero: bool,
    ) -> Self {
        let labels: Vec<String> = axis
            .and_then(|axis| axis.get("data"))
            .and_then(serde_json::Value::as_array)
            .map(|values| {
                values
                    .iter()
                    .map(|value| {
                        value
                            .as_str()
                            .map(ToOwned::to_owned)
                            .unwrap_or_else(|| value.to_string())
                    })
                    .collect()
            })
            .unwrap_or_default();
        let is_category = axis
            .and_then(|axis| axis.get("type"))
            .and_then(serde_json::Value::as_str)
            == Some("category")
            || !labels.is_empty();
        let mut min = values.iter().copied().reduce(f64::min).unwrap_or(0.0);
        let mut max = values.iter().copied().reduce(f64::max).unwrap_or(1.0);
        if include_zero {
            min = min.min(0.0);
            max = max.max(0.0);
        }
        min = axis
            .and_then(|axis| axis.get("min"))
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(min);
        max = axis
            .and_then(|axis| axis.get("max"))
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(max);
        if (max - min).abs() < f64::EPSILON {
            max = min + 1.0;
        }
        Self {
            labels: if is_category {
                if labels.is_empty() {
                    (0..inferred_count).map(|index| index.to_string()).collect()
                } else {
                    labels
                }
            } else {
                Vec::new()
            },
            min,
            max,
            inverse: axis
                .and_then(|axis| axis.get("inverse"))
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
            split_number: axis
                .and_then(|axis| axis.get("splitNumber"))
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(5) as usize,
            split_line: axis
                .and_then(|axis| axis.get("splitLine"))
                .and_then(serde_json::Value::as_object)
                .and_then(|line| line.get("show"))
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(true),
            show_labels: axis
                .and_then(|axis| axis.get("axisLabel"))
                .and_then(serde_json::Value::as_object)
                .and_then(|label| label.get("show"))
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(true),
        }
    }

    fn normalize(&self, value: Option<f64>, index: usize) -> f32 {
        let normalized = if self.labels.is_empty() {
            (value.unwrap_or(index as f64) - self.min) / (self.max - self.min).max(1e-12)
        } else {
            let count = self.labels.len().max(1);
            (value.map(|value| value as usize).unwrap_or(index) % count) as f64 / count as f64
        }
        .clamp(0.0, 1.0) as f32;
        if self.inverse {
            1.0 - normalized
        } else {
            normalized
        }
    }

    fn tick_count(&self) -> usize {
        if self.labels.is_empty() {
            self.split_number.max(1)
        } else {
            self.labels.len().max(1)
        }
    }

    fn label(&self, index: usize) -> String {
        self.labels
            .get(index)
            .cloned()
            .unwrap_or_else(|| self.value_label(index as f32 / self.tick_count() as f32))
    }

    fn value_label(&self, normalized: f32) -> String {
        let value = self.min + normalized as f64 * (self.max - self.min);
        if (value - value.round()).abs() < 1e-5 {
            format!("{value:.0}")
        } else {
            format!("{value:.2}")
        }
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

fn matching_polar_axis<'a>(
    extra: &'a std::collections::BTreeMap<String, serde_json::Value>,
    key: &str,
    polar_index: usize,
) -> Option<&'a serde_json::Map<String, serde_json::Value>> {
    match extra.get(key)? {
        serde_json::Value::Array(values) => values.iter().find_map(|value| {
            let axis = value.as_object()?;
            (axis
                .get("polarIndex")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0) as usize
                == polar_index)
                .then_some(axis)
        }),
        value if polar_index == 0 => value.as_object(),
        _ => None,
    }
}

fn resolve_polar_length(
    value: Option<&serde_json::Value>,
    total: f32,
    origin: f32,
    fallback: f32,
) -> f32 {
    origin
        + value
            .and_then(|value| {
                value.as_f64().map(|value| value as f32).or_else(|| {
                    value
                        .as_str()?
                        .strip_suffix('%')?
                        .parse::<f32>()
                        .ok()
                        .map(|value| total * value / 100.0)
                })
            })
            .unwrap_or(total * fallback)
}

fn resolve_polar_radius(value: Option<&serde_json::Value>, max_radius: f32, fallback: f32) -> f32 {
    value
        .and_then(|value| {
            value.as_f64().map(|value| value as f32).or_else(|| {
                value
                    .as_str()?
                    .strip_suffix('%')?
                    .parse::<f32>()
                    .ok()
                    .map(|value| max_radius * value / 100.0)
            })
        })
        .unwrap_or(max_radius * fallback)
        .max(0.0)
}

fn inside_plot(plot: &crate::render::geometry::Plot, x: f32, y: f32) -> bool {
    x >= plot.x && x <= plot.x + plot.width && y >= plot.y && y <= plot.y + plot.height
}

fn starts_new_segment(previous: Option<usize>, current: usize, connect_nulls: bool) -> bool {
    !connect_nulls && previous.is_some_and(|previous| previous + 1 != current)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_index_breaks_line_unless_connect_nulls_is_enabled() {
        assert!(starts_new_segment(Some(0), 2, false));
        assert!(!starts_new_segment(Some(0), 2, true));
        assert!(!starts_new_segment(Some(1), 2, false));
    }

    #[test]
    fn step_modes_match_echarts_turn_order() {
        let points = [(0.0, 10.0), (20.0, 30.0)];
        assert_eq!(
            step_polyline(&points, "start"),
            [(0.0, 10.0), (0.0, 30.0), (20.0, 30.0)]
        );
        assert_eq!(
            step_polyline(&points, "end"),
            [(0.0, 10.0), (20.0, 10.0), (20.0, 30.0)]
        );
        assert_eq!(
            step_polyline(&points, "middle"),
            [(0.0, 10.0), (10.0, 10.0), (10.0, 30.0), (20.0, 30.0)]
        );
    }

    #[test]
    fn lttb_preserves_endpoints_and_target_size() {
        let points = (0..100)
            .map(|x| (x as f32, ((x * 17) % 31) as f32))
            .collect::<Vec<_>>();
        let sampled = lttb(&points, 20);
        assert_eq!(sampled.len(), 20);
        assert_eq!(sampled.first(), points.first());
        assert_eq!(sampled.last(), points.last());
    }

    #[test]
    fn polar_category_starts_at_top_and_advances_clockwise() {
        let option = ChartOption::from_json_str(
            r#"{
                "polar":{"center":["50%","50%"],"radius":"80%"},
                "angleAxis":{"type":"category","data":["A","B","C","D"]},
                "radiusAxis":{"min":0,"max":10},
                "series":[{"type":"line","coordinateSystem":"polar","data":[5,5,5,5]}]
            }"#,
        )
        .unwrap();
        let Series::Line(series) = &option.series[0] else {
            panic!("expected line");
        };
        let config = PolarConfig::from_option(
            &option,
            0,
            crate::render::geometry::Plot {
                x: 0.0,
                y: 0.0,
                width: 200.0,
                height: 200.0,
            },
            series,
        );
        let first = config.project(config.radius_for(5.0, 0), config.angle_for(None, 0));
        let second = config.project(config.radius_for(5.0, 1), config.angle_for(None, 1));
        assert!((first.0 - 100.0).abs() < 1e-4);
        assert!(first.1 < 100.0);
        assert!(second.0 > 100.0);
        assert!((second.1 - 100.0).abs() < 1e-4);
    }
}
