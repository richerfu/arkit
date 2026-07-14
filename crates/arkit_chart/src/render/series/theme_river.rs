use std::collections::BTreeMap;

use super::super::prelude::*;

pub(super) fn render(series: &BasicSeries, context: &mut FreeRenderContext<'_>) {
    let series_index = context.series_index;
    let plot = context.plot;
    let palette = context.palette;
    let canvas = context.canvas;
    let hits = &mut *context.hits;
    let mut times = Vec::new();
    let mut groups: BTreeMap<String, BTreeMap<i64, f64>> = BTreeMap::new();
    for point in &series.data {
        if point.values.len() < 3 {
            continue;
        }
        let (Some(time), Some(value)) = (point.number_opt(0), point.number_opt(1)) else {
            continue;
        };
        let time = time as i64;
        let value = value.max(0.0);
        let name = match &point.values[2] {
            DataValue::String(value) => value.clone(),
            DataValue::Number(value) => value.to_string(),
            DataValue::Null => continue,
        };
        times.push(time);
        groups.entry(name).or_default().insert(time, value);
    }
    times.sort_unstable();
    times.dedup();
    let (Some(min_time), Some(max_time)) = (times.first().copied(), times.last().copied()) else {
        return;
    };
    let totals: BTreeMap<i64, f64> = times
        .iter()
        .map(|time| {
            (
                *time,
                groups
                    .values()
                    .map(|values| values.get(time).copied().unwrap_or_default())
                    .sum(),
            )
        })
        .collect();
    let max_total = totals
        .values()
        .copied()
        .reduce(f64::max)
        .unwrap_or(1.0)
        .max(1.0);
    let x_at = |time: i64| {
        plot.x + (time - min_time) as f32 / (max_time - min_time).max(1) as f32 * plot.width
    };
    let mut lower: BTreeMap<i64, f64> = totals
        .iter()
        .map(|(time, total)| (*time, -*total / 2.0))
        .collect();

    for (group_index, (name, values)) in groups.iter().enumerate() {
        let mut top_points = Vec::new();
        let mut bottom_points = Vec::new();
        for time in &times {
            let bottom = lower.get(time).copied().unwrap_or_default();
            let top = bottom + values.get(time).copied().unwrap_or_default();
            let y = |value: f64| {
                plot.y + plot.height / 2.0 - value as f32 / max_total as f32 * plot.height * 0.85
            };
            bottom_points.push((x_at(*time), y(bottom)));
            top_points.push((x_at(*time), y(top)));
            lower.insert(*time, top);
        }
        let mut path = Path::new();
        for (index, (x, y)) in top_points.iter().enumerate() {
            if index == 0 {
                path.move_to(*x, *y);
            } else {
                path.line_to(*x, *y);
            }
        }
        for (x, y) in bottom_points.iter().rev() {
            path.line_to(*x, *y);
        }
        path.close();
        if let Some(canvas) = canvas {
            fill_path(
                canvas,
                &path,
                with_opacity(color(palette, group_index), 0.72),
            );
            if let Some((x, y)) = top_points.first() {
                set_next_data_index(group_index);
                draw_text(canvas, name, *x + 4.0, *y, 10.0, 0xFF333333, 400);
            }
        }
        if let (Some(top), Some(bottom)) = (top_points.first(), bottom_points.last()) {
            hits.push(HitRegion {
                shape: HitShape::Rect {
                    x: plot.x,
                    y: top.1.min(bottom.1),
                    width: plot.width,
                    height: (top.1 - bottom.1).abs().max(8.0),
                },
                event: ChartEvent {
                    series_index,
                    data_index: group_index,
                    series_name: series.name.clone(),
                    name: Some(name.clone()),
                    value: vec![values.values().sum()],
                    x: plot.x + plot.width / 2.0,
                    y: plot.y + plot.height / 2.0,
                    component_type: String::from("themeRiver"),
                },
            });
        }
    }
}
