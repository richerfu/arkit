use super::super::prelude::*;

pub(super) fn render(series: &CustomSeries, context: &mut FreeRenderContext<'_>) {
    let plot = context.plot;
    let palette = context.palette;
    let canvas = context.canvas;

    if let Some(canvas) = canvas {
        (series.renderer)(CustomRenderContext {
            canvas,
            width: plot.width,
            height: plot.height,
            palette,
        });
    }
}
