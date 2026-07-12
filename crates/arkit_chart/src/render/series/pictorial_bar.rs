use ohos_drawing_binding::Rect;

use super::super::compat;
use super::super::prelude::*;
use super::super::symbol::{draw_symbol, resolve_symbol, SymbolSpec};
use super::BarLayout;

pub(super) fn render(series: &BasicSeries, context: &mut CartesianRenderContext<'_>) {
    let series_index = context.series_index;
    let plot = context.plot;
    let layout = context.layout;
    let horizontal = layout.y.is_category() && !layout.x.is_category();
    let category_scale = if horizontal { &layout.y } else { &layout.x };
    let value_scale = if horizontal { &layout.x } else { &layout.y };
    let bar_layout = context.bar_layout.unwrap_or(BarLayout {
        offset: -category_scale.band_width(plot, horizontal, series.data.len().max(1)) * 0.3,
        width: category_scale.band_width(plot, horizontal, series.data.len().max(1)) * 0.6,
    });
    let repeat = series
        .options
        .extra
        .get("symbolRepeat")
        .is_some_and(|value| value == true || value.as_str() == Some("fixed"));
    let repeat_direction =
        compat::string(&series.options.extra, "symbolRepeatDirection").unwrap_or("end");
    let symbol_clip = compat::boolean(&series.options.extra, "symbolClip", false);
    let symbol_position = compat::string(&series.options.extra, "symbolPosition").unwrap_or("end");
    let symbol_margin = resolve_margin(series.options.extra.get("symbolMargin"), bar_layout.width);

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
        if !category_scale.contains(category_value, index) {
            continue;
        }
        let category = category_scale.position(plot, category_value, index, horizontal);
        let baseline = value_scale.zero_position(plot, !horizontal);
        let end = value_scale.position(plot, Some(value), index, !horizontal);
        let bounding_value = series
            .options
            .extra
            .get("symbolBoundingData")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(value);
        let bounding_end = value_scale.position(plot, Some(bounding_value), index, !horizontal);
        let value_extent = (end - baseline).abs().max(0.5);
        let bounding_extent = (bounding_end - baseline).abs().max(value_extent).max(1.0);
        let positive_direction = (end - baseline).signum();
        let bounds = if horizontal {
            (
                baseline.min(end),
                category + bar_layout.offset,
                value_extent,
                bar_layout.width,
            )
        } else {
            (
                category + bar_layout.offset,
                baseline.min(end),
                bar_layout.width,
                value_extent,
            )
        };
        let fill = item_color(series, Some(point), context.palette, series_index);
        let border = border(series, Some(point));
        let base_spec = resolve_symbol(series, point, None);
        let explicit_size = series.options.symbol_size_dimensions.is_some()
            || series.options.symbol_size != 7.0
            || point.extra.contains_key("symbolSize");
        let cross_size = if horizontal {
            base_spec.size[1]
        } else {
            base_spec.size[0]
        };
        let main_size = if horizontal {
            base_spec.size[0]
        } else {
            base_spec.size[1]
        };
        let cross_size = if explicit_size {
            cross_size.min(bar_layout.width.max(1.0))
        } else {
            bar_layout.width.max(1.0)
        };
        let main_size = if explicit_size {
            main_size.max(1.0)
        } else if repeat {
            cross_size
        } else {
            bounding_extent
        };

        if let Some(canvas) = context.canvas {
            if symbol_clip {
                begin_clip(canvas, bounds);
            }
            if repeat {
                let step = (main_size + symbol_margin).max(1.0);
                let count = (bounding_extent / step).ceil().max(1.0) as usize;
                for symbol_index in 0..count {
                    let distance = main_size / 2.0 + symbol_index as f32 * step;
                    let distance = if repeat_direction == "start" {
                        bounding_extent - distance
                    } else {
                        distance
                    };
                    draw_at(
                        canvas,
                        &base_spec,
                        horizontal,
                        category + bar_layout.offset + bar_layout.width / 2.0,
                        baseline + positive_direction * distance,
                        cross_size,
                        main_size,
                        fill,
                        border,
                    );
                }
            } else {
                let distance = match symbol_position {
                    "start" => main_size / 2.0,
                    "center" => bounding_extent / 2.0,
                    _ => bounding_extent - main_size / 2.0,
                };
                draw_at(
                    canvas,
                    &base_spec,
                    horizontal,
                    category + bar_layout.offset + bar_layout.width / 2.0,
                    baseline + positive_direction * distance,
                    cross_size,
                    main_size,
                    fill,
                    border,
                );
            }
            if symbol_clip {
                canvas.restore();
            }
            let label = effective_label(series, point);
            if label.show {
                let text = format_label(&label, series, point, index);
                let (x, y) = if horizontal {
                    (
                        end + positive_direction * label.distance,
                        category + label.font_size * 0.35,
                    )
                } else {
                    (category, end + positive_direction * label.distance)
                };
                set_next_data_index(index);
                draw_text(
                    canvas,
                    &text,
                    x,
                    y,
                    label.font_size as f64,
                    label.color.unwrap_or(0xFF333333),
                    label.font_weight,
                );
            }
        }
        context.hits.push(rect_hit(
            "pictorialBar",
            series_index,
            index,
            series.name.clone(),
            point,
            bounds,
        ));
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_at(
    canvas: &ohos_drawing_binding::Canvas,
    base: &SymbolSpec<'_>,
    horizontal: bool,
    cross: f32,
    main: f32,
    cross_size: f32,
    main_size: f32,
    color: u32,
    border: Option<(u32, f32)>,
) {
    let mut spec = *base;
    spec.size = if horizontal {
        [main_size, cross_size]
    } else {
        [cross_size, main_size]
    };
    let (x, y) = if horizontal {
        (main, cross)
    } else {
        (cross, main)
    };
    draw_symbol(canvas, &spec, x, y, color, border);
}

fn resolve_margin(value: Option<&serde_json::Value>, base: f32) -> f32 {
    value
        .and_then(|value| {
            value.as_f64().map(|value| value as f32).or_else(|| {
                value
                    .as_str()?
                    .strip_suffix('%')?
                    .parse::<f32>()
                    .ok()
                    .map(|value| base * value / 100.0)
            })
        })
        .unwrap_or(0.0)
}

fn begin_clip(canvas: &ohos_drawing_binding::Canvas, bounds: (f32, f32, f32, f32)) {
    canvas.save();
    let rect = Rect::new(bounds.0, bounds.1, bounds.0 + bounds.2, bounds.1 + bounds.3);
    unsafe {
        ohos_native_drawing_sys::OH_Drawing_CanvasClipRect(
            canvas.as_ptr(),
            rect.as_ptr(),
            ohos_native_drawing_sys::OH_Drawing_CanvasClipOp_INTERSECT,
            true,
        );
    }
}
