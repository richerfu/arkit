use super::super::compat;
use super::super::prelude::*;

pub(super) fn render(series: &SunburstSeries, context: &mut FreeRenderContext<'_>) {
    let plot = context.plot;
    let center = compat::pair(&series.options.extra, "center");
    let cx = compat::position(
        center.map(|pair| pair[0]),
        plot.x,
        plot.width,
        plot.x + plot.width / 2.0,
    );
    let cy = compat::position(
        center.map(|pair| pair[1]),
        plot.y,
        plot.height,
        plot.y + plot.height / 2.0,
    );
    let radius_base = plot.width.min(plot.height) / 2.0;
    let radii = series
        .options
        .extra
        .get("radius")
        .and_then(serde_json::Value::as_array);
    let inner = radii
        .and_then(|values| values.first())
        .map(|value| compat::length(Some(value), radius_base, 0.0))
        .unwrap_or(0.0);
    let outer = radii
        .and_then(|values| values.get(1))
        .map(|value| compat::length(Some(value), radius_base, radius_base * 0.9))
        .unwrap_or(radius_base * 0.9);
    let depth = max_depth(&series.data).max(1);
    let ring_width = (outer - inner).max(1.0) / depth as f32;
    let start = -(compat::number(&series.options.extra, "startAngle", 90.0) as f32).to_radians();
    let mut state = SunburstRenderState {
        series,
        series_index: context.series_index,
        center: (cx, cy),
        inner,
        ring_width,
        palette: context.palette,
        canvas: context.canvas,
        hits: context.hits,
        data_index: 0,
    };
    state.render_level(&series.data, 0, start, TAU);
}

struct SunburstRenderState<'a> {
    series: &'a SunburstSeries,
    series_index: usize,
    center: (f32, f32),
    inner: f32,
    ring_width: f32,
    palette: &'a [u32],
    canvas: Option<&'a ohos_drawing_binding::Canvas>,
    hits: &'a mut Vec<HitRegion>,
    data_index: usize,
}

impl SunburstRenderState<'_> {
    fn render_level(&mut self, nodes: &[SunburstNode], depth: usize, start: f32, sweep: f32) {
        let total = nodes
            .iter()
            .map(|node| node.value.max(0.0))
            .sum::<f64>()
            .max(1.0);
        let mut cursor = start;
        for node in nodes {
            let node_sweep = sweep * (node.value.max(0.0) / total) as f32;
            let data_index = self.data_index;
            self.data_index += 1;
            let inner = self.inner + self.ring_width * depth as f32;
            let outer = inner + self.ring_width;
            let fill = node
                .item_style
                .color
                .map(|color| with_opacity(color, node.item_style.opacity))
                .unwrap_or_else(|| color(self.palette, data_index));
            if let Some(canvas) = self.canvas {
                fill_ring_sector(
                    canvas,
                    self.center,
                    (inner, outer),
                    cursor,
                    node_sweep,
                    fill,
                );
                if self.series.options.label.show && node_sweep * outer > 18.0 {
                    let middle = cursor + node_sweep / 2.0;
                    let radius = (inner + outer) / 2.0;
                    set_next_data_index(data_index);
                    draw_text(
                        canvas,
                        &node.name,
                        self.center.0 + middle.cos() * radius
                            - node.name.chars().count() as f32 * 2.5,
                        self.center.1 + middle.sin() * radius + 4.0,
                        self.series.options.label.font_size.min(11.0) as f64,
                        self.series.options.label.color.unwrap_or(0xFFFFFFFF),
                        self.series.options.label.font_weight,
                    );
                }
            }
            self.hits.push(HitRegion {
                shape: HitShape::Sector {
                    cx: self.center.0,
                    cy: self.center.1,
                    inner,
                    outer,
                    start: normalize_angle(cursor),
                    sweep: node_sweep,
                },
                event: ChartEvent {
                    series_index: self.series_index,
                    data_index,
                    series_name: self.series.name.clone(),
                    name: Some(node.name.clone()),
                    value: vec![node.value],
                    x: self.center.0,
                    y: self.center.1,
                    component_type: String::from("sunburst"),
                },
            });
            if !node.children.is_empty() {
                self.render_level(&node.children, depth + 1, cursor, node_sweep);
            }
            cursor += node_sweep;
        }
    }
}

fn max_depth(nodes: &[SunburstNode]) -> usize {
    nodes
        .iter()
        .map(|node| 1 + max_depth(&node.children))
        .max()
        .unwrap_or(0)
}
