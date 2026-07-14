use ohos_drawing_binding::Path;
use serde_json::Value;

use super::super::prelude::*;
use crate::render::geometry::Plot;

#[derive(Debug, Clone, Copy)]
pub(super) struct GeoTransform {
    bounds: (f64, f64, f64, f64),
    scale_x: f32,
    scale_y: f32,
    offset_x: f32,
    offset_y: f32,
}

impl GeoTransform {
    pub(super) fn project(self, point: (f64, f64)) -> (f32, f32) {
        (
            self.offset_x + (point.0 - self.bounds.0) as f32 * self.scale_x,
            self.offset_y + (self.bounds.3 - point.1) as f32 * self.scale_y,
        )
    }
}

fn option_component<'a>(
    extra: &'a std::collections::BTreeMap<String, Value>,
    key: &str,
    index: usize,
) -> Option<&'a serde_json::Map<String, Value>> {
    match extra.get(key)? {
        Value::Array(values) => values.get(index)?.as_object(),
        value if index == 0 => value.as_object(),
        _ => None,
    }
}

pub(super) fn transform_from_geo_component(
    option: &ChartOption,
    plot: Plot,
    geo_index: usize,
) -> Option<GeoTransform> {
    let geo = option_component(&option.extra, "geo", geo_index)?;
    let options = crate::parser::parse_map_options(geo);
    let features = geo
        .get("geoJson")
        .or_else(|| geo.get("geoJSON"))
        .or_else(|| geo.get("features"))
        .and_then(crate::parser::parse_geo_features)
        .or_else(|| {
            geo.get("map")
                .and_then(Value::as_str)
                .and_then(crate::registry::registered_map)
        })?;
    let bounds = options
        .bounding_coords
        .map(normalize_bounding_coords)
        .or_else(|| map_bounds(&features))?;
    Some(geo_transform(bounds, map_layout(plot, &options), &options))
}

pub(super) fn draw_geo_component(
    option: &ChartOption,
    plot: Plot,
    geo_index: usize,
    canvas: Option<&ohos_drawing_binding::Canvas>,
) {
    let Some(canvas) = canvas else {
        return;
    };
    let Some(geo) = option_component(&option.extra, "geo", geo_index) else {
        return;
    };
    if !geo.get("show").and_then(Value::as_bool).unwrap_or(true) {
        return;
    }
    let Some(transform) = transform_from_geo_component(option, plot, geo_index) else {
        return;
    };
    let features = geo
        .get("geoJson")
        .or_else(|| geo.get("geoJSON"))
        .or_else(|| geo.get("features"))
        .and_then(crate::parser::parse_geo_features)
        .or_else(|| {
            geo.get("map")
                .and_then(Value::as_str)
                .and_then(crate::registry::registered_map)
        })
        .unwrap_or_default();
    for feature in &features {
        let (path, _) = feature_path(feature, transform);
        fill_path(canvas, &path, 0xFFF1F5F9);
        stroke_path(canvas, &path, 0xFFCBD5E1, 1.0);
    }
}

pub(super) fn render(series: &MapSeries, context: &mut FreeRenderContext<'_>) {
    let series_index = context.series_index;
    let bounds = series
        .map_options
        .bounding_coords
        .map(normalize_bounding_coords)
        .or_else(|| map_bounds(&series.features))
        .unwrap_or((0.0, 0.0, 1.0, 1.0));
    let layout = map_layout(context.plot, &series.map_options);
    let transform = geo_transform(bounds, layout, &series.map_options);
    let values = series
        .features
        .iter()
        .filter_map(|feature| feature.value)
        .collect::<Vec<_>>();
    let data_min = values.iter().copied().reduce(f64::min).unwrap_or(0.0);
    let data_max = values.iter().copied().reduce(f64::max).unwrap_or(1.0);

    for (index, feature) in series.features.iter().enumerate() {
        let persistently_selected = context.selected_items.contains(&(series_index, index));
        let emphasized = context.selected.is_some_and(|event| {
            event.component_type == "map"
                && event.series_index == series_index
                && event.data_index == index
        });
        let state = if persistently_selected {
            MapState::Select
        } else if emphasized {
            MapState::Emphasis
        } else {
            MapState::Normal
        };
        let style = feature_style(series, feature, state);
        let fill = map_fill_color(
            feature,
            context.option.visual_map_for_series(series_index),
            context.palette,
            data_min,
            data_max,
            &style,
            state,
        );
        let label = feature_label(series, feature, state);

        let (path, hit_polygons) = feature_path(feature, transform);
        if let Some(canvas) = context.canvas {
            fill_path(canvas, &path, with_opacity(fill, style.opacity));
            if let Some(border_color) = style.border_color {
                if style.border_width > 0.0 {
                    stroke_path(
                        canvas,
                        &path,
                        with_opacity(border_color, style.opacity),
                        style.border_width,
                    );
                }
            }
        }

        let geo_center = feature.center.unwrap_or_else(|| feature_center(feature));
        let center = transform.project(geo_center);
        if let Some(canvas) = context.canvas {
            if label.show {
                let text = format_map_label(&label, series, feature);
                let width = text.chars().count() as f32 * label.font_size * 0.55;
                set_next_data_index(index);
                draw_text(
                    canvas,
                    &text,
                    center.0 - width / 2.0 + label.offset[0],
                    center.1 + label.font_size * 0.35 + label.offset[1],
                    label.font_size as f64,
                    label.color.unwrap_or(0xFF333333),
                    label.font_weight,
                );
            }
        }
        if !hit_polygons.is_empty() {
            let point = DataPoint::named(feature.name.clone(), feature.value);
            context.hits.push(polygon_hit(
                "map",
                series_index,
                index,
                series.name.clone(),
                &point,
                center,
                hit_polygons,
            ));
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MapState {
    Normal,
    Emphasis,
    Select,
}

fn feature_style(series: &MapSeries, feature: &MapFeature, state: MapState) -> ItemStyle {
    let mut style = merge_item_style(&series.options.item_style, &feature.item_style);
    let (series_state, feature_state) = match state {
        MapState::Normal => return style,
        MapState::Emphasis => (
            &series.map_options.emphasis_item_style,
            &feature.emphasis_item_style,
        ),
        MapState::Select => (
            &series.map_options.select_item_style,
            &feature.select_item_style,
        ),
    };
    style = merge_item_style(&style, series_state);
    merge_item_style(&style, feature_state)
}

fn feature_label(series: &MapSeries, feature: &MapFeature, state: MapState) -> LabelStyle {
    let mut label = merge_label_style(&series.options.label, &feature.label);
    let (series_state, feature_state) = match state {
        MapState::Normal => return label,
        MapState::Emphasis => (&series.map_options.emphasis_label, &feature.emphasis_label),
        MapState::Select => (&series.map_options.select_label, &feature.select_label),
    };
    label = merge_label_style(&label, series_state);
    merge_label_style(&label, feature_state)
}

fn map_fill_color(
    feature: &MapFeature,
    visual_map: Option<&VisualMap>,
    palette: &[u32],
    data_min: f64,
    data_max: f64,
    style: &ItemStyle,
    state: MapState,
) -> u32 {
    if state != MapState::Normal {
        if let Some(color) = style.color {
            return color;
        }
    }
    if let (Some(value), Some(visual_map)) = (feature.value, visual_map) {
        return visual_map_color(visual_map, value);
    }
    if let Some(color) = style.color {
        return color;
    }
    if let Some(value) = feature.value {
        let normalized = (value - data_min) / (data_max - data_min).max(1e-12);
        return gradient_color(palette, normalized);
    }
    0xFFEEEEEE
}

fn feature_path(feature: &MapFeature, transform: GeoTransform) -> (Path, Vec<HitPolygon>) {
    let mut path = Path::new();
    path.set_fill_type(ohos_native_drawing_sys::OH_Drawing_PathFillType_PATH_FILL_TYPE_EVEN_ODD);
    let mut hit_polygons = Vec::with_capacity(feature.polygons.len());
    for polygon in &feature.polygons {
        let exterior = project_ring(&mut path, &polygon.exterior, transform);
        let holes = polygon
            .holes
            .iter()
            .map(|ring| project_ring(&mut path, ring, transform))
            .filter(|ring| ring.len() >= 3)
            .collect::<Vec<_>>();
        if exterior.len() >= 3 {
            hit_polygons.push(HitPolygon { exterior, holes });
        }
    }
    (path, hit_polygons)
}

fn project_ring(path: &mut Path, ring: &[(f64, f64)], transform: GeoTransform) -> Vec<(f32, f32)> {
    let projected = ring
        .iter()
        .map(|point| transform.project(*point))
        .collect::<Vec<_>>();
    for (index, point) in projected.iter().enumerate() {
        if index == 0 {
            path.move_to(point.0, point.1);
        } else {
            path.line_to(point.0, point.1);
        }
    }
    if projected.len() >= 3 {
        path.close();
    }
    projected
}

fn map_layout(view: Plot, options: &MapOptions) -> Plot {
    if let (Some(center), Some(size)) = (&options.layout_center, &options.layout_size) {
        let cx = resolve_position(&center[0], view.width, view.x, "center");
        let cy = resolve_position(&center[1], view.height, view.y, "center");
        let size = resolve_length(size, view.width.min(view.height))
            .unwrap_or(view.width.min(view.height));
        return Plot {
            x: cx - size / 2.0,
            y: cy - size / 2.0,
            width: size,
            height: size,
        };
    }

    let left = explicit_length(&options.left, view.width);
    let right = options
        .right
        .as_ref()
        .and_then(|value| explicit_length(value, view.width));
    let top = explicit_length(&options.top, view.height);
    let bottom = options
        .bottom
        .as_ref()
        .and_then(|value| explicit_length(value, view.height));
    let width = options
        .width
        .as_ref()
        .and_then(|value| resolve_length(value, view.width))
        .or_else(|| Some(view.width - left? - right?))
        .unwrap_or(view.width)
        .max(1.0);
    let height = options
        .height
        .as_ref()
        .and_then(|value| resolve_length(value, view.height))
        .or_else(|| Some(view.height - top? - bottom?))
        .unwrap_or(view.height)
        .max(1.0);
    let x = left
        .map(|value| view.x + value)
        .or_else(|| right.map(|value| view.x + view.width - value - width))
        .unwrap_or_else(|| resolve_box_start(&options.left, view.x, view.width, width));
    let y = top
        .map(|value| view.y + value)
        .or_else(|| bottom.map(|value| view.y + view.height - value - height))
        .unwrap_or_else(|| resolve_box_start(&options.top, view.y, view.height, height));
    Plot {
        x,
        y,
        width,
        height,
    }
}

fn geo_transform(bounds: (f64, f64, f64, f64), layout: Plot, options: &MapOptions) -> GeoTransform {
    let geo_width = (bounds.2 - bounds.0).max(1e-9) as f32;
    let geo_height = (bounds.3 - bounds.1).max(1e-9) as f32;
    let aspect = options.aspect_scale.max(1e-6);
    let mut zoom = options.zoom.max(1e-6);
    if let Some((min, max)) = options.scale_limit {
        zoom = zoom.clamp(min.max(1e-6), max.max(min).max(1e-6));
    }
    let base_scale = (layout.width / (geo_width * aspect)).min(layout.height / geo_height) * zoom;
    let scale_x = base_scale * aspect;
    let scale_y = base_scale;
    let center = options
        .center
        .unwrap_or(((bounds.0 + bounds.2) / 2.0, (bounds.1 + bounds.3) / 2.0));
    let offset_x = layout.x + layout.width / 2.0 - (center.0 - bounds.0) as f32 * scale_x
        + options.pan_offset[0];
    let offset_y = layout.y + layout.height / 2.0 - (bounds.3 - center.1) as f32 * scale_y
        + options.pan_offset[1];
    GeoTransform {
        bounds,
        scale_x,
        scale_y,
        offset_x,
        offset_y,
    }
}

fn explicit_length(value: &Value, total: f32) -> Option<f32> {
    match value {
        Value::Number(_) => resolve_length(value, total),
        Value::String(value) if value.ends_with('%') => {
            resolve_length(&Value::String(value.clone()), total)
        }
        _ => None,
    }
}

fn resolve_length(value: &Value, total: f32) -> Option<f32> {
    value.as_f64().map(|value| value as f32).or_else(|| {
        value
            .as_str()?
            .strip_suffix('%')?
            .parse::<f32>()
            .ok()
            .map(|value| total * value / 100.0)
    })
}

fn resolve_position(value: &Value, total: f32, origin: f32, default: &str) -> f32 {
    if let Some(value) = resolve_length(value, total) {
        return origin + value;
    }
    match value.as_str().unwrap_or(default) {
        "left" | "top" => origin,
        "right" | "bottom" => origin + total,
        _ => origin + total / 2.0,
    }
}

fn resolve_box_start(value: &Value, origin: f32, total: f32, size: f32) -> f32 {
    match value.as_str().unwrap_or("center") {
        "left" | "top" => origin,
        "right" | "bottom" => origin + total - size,
        _ => origin + (total - size) / 2.0,
    }
}

fn normalize_bounding_coords(coords: [(f64, f64); 2]) -> (f64, f64, f64, f64) {
    (
        coords[0].0.min(coords[1].0),
        coords[0].1.min(coords[1].1),
        coords[0].0.max(coords[1].0),
        coords[0].1.max(coords[1].1),
    )
}

fn feature_center(feature: &MapFeature) -> (f64, f64) {
    feature
        .polygons
        .iter()
        .filter_map(|polygon| ring_centroid(&polygon.exterior))
        .max_by(|left, right| left.2.abs().total_cmp(&right.2.abs()))
        .map(|(x, y, _)| (x, y))
        .unwrap_or((0.0, 0.0))
}

fn ring_centroid(ring: &[(f64, f64)]) -> Option<(f64, f64, f64)> {
    if ring.len() < 3 {
        return None;
    }
    let mut twice_area = 0.0;
    let mut x = 0.0;
    let mut y = 0.0;
    for index in 0..ring.len() {
        let current = ring[index];
        let next = ring[(index + 1) % ring.len()];
        let cross = current.0 * next.1 - next.0 * current.1;
        twice_area += cross;
        x += (current.0 + next.0) * cross;
        y += (current.1 + next.1) * cross;
    }
    if twice_area.abs() < 1e-12 {
        let count = ring.len() as f64;
        return Some((
            ring.iter().map(|point| point.0).sum::<f64>() / count,
            ring.iter().map(|point| point.1).sum::<f64>() / count,
            0.0,
        ));
    }
    Some((
        x / (3.0 * twice_area),
        y / (3.0 * twice_area),
        twice_area / 2.0,
    ))
}

fn format_map_label(label: &LabelStyle, series: &MapSeries, feature: &MapFeature) -> String {
    let value = feature
        .value
        .map(|value| {
            if (value - value.round()).abs() < 1e-8 {
                format!("{value:.0}")
            } else {
                value.to_string()
            }
        })
        .unwrap_or_else(|| String::from("-"));
    label
        .formatter
        .as_deref()
        .unwrap_or("{b}")
        .replace("{a}", series.name.as_deref().unwrap_or_default())
        .replace("{b}", &feature.name)
        .replace("{c}", &value)
}

pub(crate) fn map_bounds(features: &[MapFeature]) -> Option<(f64, f64, f64, f64)> {
    let mut min_x = f64::MAX;
    let mut min_y = f64::MAX;
    let mut max_x = f64::MIN;
    let mut max_y = f64::MIN;
    let mut any = false;
    for feature in features {
        for polygon in &feature.polygons {
            for (x, y) in &polygon.exterior {
                any = true;
                min_x = min_x.min(*x);
                min_y = min_y.min(*y);
                max_x = max_x.max(*x);
                max_y = max_y.max(*y);
            }
        }
    }
    any.then_some((min_x, min_y, max_x, max_y))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_center_and_size_override_box_layout() {
        let options = MapOptions {
            layout_center: Some([
                Value::String(String::from("25%")),
                Value::String(String::from("75%")),
            ]),
            layout_size: Some(Value::String(String::from("50%"))),
            ..MapOptions::default()
        };
        let layout = map_layout(
            Plot {
                x: 0.0,
                y: 0.0,
                width: 400.0,
                height: 200.0,
            },
            &options,
        );
        assert_eq!(
            (layout.x, layout.y, layout.width, layout.height),
            (50.0, 100.0, 100.0, 100.0)
        );
    }

    #[test]
    fn centroid_uses_largest_polygon() {
        let feature = MapFeature::new(
            "region",
            vec![
                MapPolygon::new([(0.0, 0.0), (2.0, 0.0), (2.0, 2.0), (0.0, 2.0)]),
                MapPolygon::new([(10.0, 10.0), (11.0, 10.0), (11.0, 11.0)]),
            ],
        );
        let center = feature_center(&feature);
        assert!((center.0 - 1.0).abs() < 1e-6);
        assert!((center.1 - 1.0).abs() < 1e-6);
    }

    #[test]
    fn state_style_merges_only_explicit_fields() {
        let normal_label = LabelStyle {
            show: true,
            formatter: Some(String::from("{b} {c}")),
            ..LabelStyle::default()
        };
        let mut select_label = LabelStyle {
            color: Some(0xFF111827),
            ..LabelStyle::default()
        };
        select_label.specified.insert(String::from("color"));
        let merged = merge_label_style(&normal_label, &select_label);
        assert!(merged.show);
        assert_eq!(merged.formatter.as_deref(), Some("{b} {c}"));
        assert_eq!(merged.color, Some(0xFF111827));

        let normal_item = ItemStyle {
            border_width: 2.0,
            ..ItemStyle::default()
        };
        let mut override_item = ItemStyle::default();
        override_item.specified.insert(String::from("borderWidth"));
        assert_eq!(
            merge_item_style(&normal_item, &override_item).border_width,
            0.0
        );
    }
}
