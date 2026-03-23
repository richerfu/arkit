use super::super::layout::component_canvas;
use super::shared::DemoContext;
use arkit::prelude::*;
use arkit_shadcn as shadcn;

pub(crate) fn render(ctx: DemoContext) -> Element {
    let toggle = ctx.toggle_state.clone();
    component_canvas(
        arkit::row_component()
            .align_items_center()
            .on_click(move || toggle.update(|checked| *checked = !*checked))
            .children(vec![
                shadcn::checkbox("", ctx.toggle_state.clone()),
                arkit::row_component()
                    .style(
                        ArkUINodeAttributeType::Margin,
                        vec![0.0, 0.0, 0.0, shadcn::theme::spacing::SM],
                    )
                    .children(vec![shadcn::label("Accept terms and conditions").into()])
                    .into(),
            ])
            .into(),
        true,
        24.0,
    )
}
