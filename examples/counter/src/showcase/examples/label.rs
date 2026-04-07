use super::super::layout::component_canvas;
use super::shared::DemoContext;
use arkit::prelude::*;
use arkit_shadcn as shadcn;

pub(crate) fn render(ctx: DemoContext) -> Element {
    let on_row_toggle = ctx.on_toggle_state.clone();
    let on_checkbox_toggle = ctx.on_toggle_state.clone();
    let checked = ctx.toggle_state;
    component_canvas(
        arkit::row_component()
            .align_items_center()
            .on_click(move || on_row_toggle(!checked))
            .children(vec![
                shadcn::checkbox("", checked, move |value| on_checkbox_toggle(value)),
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
