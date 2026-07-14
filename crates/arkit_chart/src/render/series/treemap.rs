use super::super::compat;
use super::super::geometry::Plot;
use super::super::layout::squarify;
use super::super::prelude::*;

pub(super) fn render(series: &BasicSeries, context: &mut FreeRenderContext<'_>) {
    let items = series
        .data
        .iter()
        .cloned()
        .map(TreemapItem::from_point)
        .collect::<Vec<_>>();
    let visible_min = compat::number(&series.options.extra, "visibleMin", 10.0).max(0.0) as f32;
    let children_visible_min = compat::number(
        &series.options.extra,
        "childrenVisibleMin",
        visible_min as f64,
    )
    .max(0.0) as f32;
    let leaf_depth = series
        .options
        .extra
        .get("leafDepth")
        .and_then(serde_json::Value::as_u64)
        .map(|value| value as usize);
    let mut state = TreemapRenderState {
        series,
        series_index: context.series_index,
        palette: context.palette,
        canvas: context.canvas,
        hits: context.hits,
        visible_min,
        children_visible_min,
        leaf_depth,
        data_index: 0,
    };
    state.render_items(&items, context.plot, 0);
}

#[derive(Debug, Clone)]
struct TreemapItem {
    point: DataPoint,
    children: Vec<TreemapItem>,
}

impl TreemapItem {
    fn from_point(point: DataPoint) -> Self {
        let children = point
            .extra
            .get("children")
            .and_then(serde_json::Value::as_array)
            .map(|children| {
                children
                    .iter()
                    .map(crate::parser::parse_data_point)
                    .map(Self::from_point)
                    .collect()
            })
            .unwrap_or_default();
        Self { point, children }
    }

    fn weight(&self) -> f64 {
        self.point
            .number_opt(0)
            .unwrap_or_else(|| self.children.iter().map(Self::weight).sum())
            .max(0.0)
    }
}

struct TreemapRenderState<'a> {
    series: &'a BasicSeries,
    series_index: usize,
    palette: &'a [u32],
    canvas: Option<&'a ohos_drawing_binding::Canvas>,
    hits: &'a mut Vec<HitRegion>,
    visible_min: f32,
    children_visible_min: f32,
    leaf_depth: Option<usize>,
    data_index: usize,
}

impl TreemapRenderState<'_> {
    fn render_items(&mut self, items: &[TreemapItem], plot: Plot, depth: usize) {
        let weights = items.iter().map(TreemapItem::weight).collect::<Vec<_>>();
        let areas = squarify(&weights, plot);
        for (item, area) in items.iter().zip(areas) {
            let data_index = self.data_index;
            self.data_index += 1;
            if area.width * area.height < self.visible_min {
                continue;
            }
            let style = effective_item_style(self.series, Some(&item.point));
            let gap = style.border_width.max(1.0);
            let x = area.x + gap / 2.0;
            let y = area.y + gap / 2.0;
            let width = (area.width - gap).max(1.0);
            let height = (area.height - gap).max(1.0);
            let fill = style
                .color
                .map(|color| with_opacity(color, style.opacity))
                .unwrap_or_else(|| {
                    with_opacity(
                        color(self.palette, data_index + depth),
                        (0.92 - depth as f32 * 0.08).max(0.55),
                    )
                });
            if let Some(canvas) = self.canvas {
                fill_rounded_rect(canvas, x, y, width, height, style.border_radius, fill);
                if let Some(border_color) = style.border_color.filter(|_| style.border_width > 0.0)
                {
                    stroke_rounded_rect(
                        canvas,
                        (x, y, width, height),
                        style.border_radius,
                        border_color,
                        style.border_width,
                    );
                }
                self.draw_label(canvas, item, data_index, (x, y, width, height), depth);
            }
            let may_render_children = !item.children.is_empty()
                && self.leaf_depth.is_none_or(|leaf_depth| depth < leaf_depth)
                && width * height >= self.children_visible_min;
            if may_render_children {
                let header = if label_show(self.series, &item.point) {
                    effective_label(self.series, &item.point).font_size + 8.0
                } else {
                    3.0
                };
                let child_plot = Plot {
                    x: x + 3.0,
                    y: y + header,
                    width: (width - 6.0).max(1.0),
                    height: (height - header - 3.0).max(1.0),
                };
                self.render_items(&item.children, child_plot, depth + 1);
            }
            self.hits.push(rect_hit(
                "treemap",
                self.series_index,
                data_index,
                self.series.name.clone(),
                &item.point,
                (x, y, width, height),
            ));
        }
    }

    fn draw_label(
        &self,
        canvas: &ohos_drawing_binding::Canvas,
        item: &TreemapItem,
        data_index: usize,
        rect: (f32, f32, f32, f32),
        depth: usize,
    ) {
        let label = effective_label(self.series, &item.point);
        if !label.show || rect.2 < 28.0 || rect.3 < label.font_size + 8.0 {
            return;
        }
        let text = format_label(&label, self.series, &item.point, data_index);
        set_next_data_index(data_index);
        draw_text(
            canvas,
            &text,
            rect.0 + 5.0,
            rect.1 + label.font_size + 5.0,
            label.font_size as f64,
            label
                .color
                .unwrap_or(if depth == 0 { 0xFFFFFFFF } else { 0xFF1F2937 }),
            label.font_weight.max(500),
        );
    }
}

fn label_show(series: &BasicSeries, point: &DataPoint) -> bool {
    effective_label(series, point).show
}
