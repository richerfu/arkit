use super::super::layout::{component_canvas, fixed_width};
use super::shared::DemoContext;
use arkit::prelude::*;
use arkit_shadcn as shadcn;

fn repo_row(name: &str) -> Element {
    arkit::row_component()
        .percent_width(1.0)
        .style(ArkUINodeAttributeType::Padding, vec![8.0, 16.0, 8.0, 16.0])
        .style(ArkUINodeAttributeType::BorderWidth, vec![1.0, 1.0, 1.0, 1.0])
        .style(
            ArkUINodeAttributeType::BorderColor,
            vec![shadcn::theme::color::BORDER],
        )
        .style(
            ArkUINodeAttributeType::BorderRadius,
            vec![
                shadcn::theme::radius::MD,
                shadcn::theme::radius::MD,
                shadcn::theme::radius::MD,
                shadcn::theme::radius::MD,
            ],
        )
        .children(vec![shadcn::text_sm(name)])
        .into()
}

pub(crate) fn render(ctx: DemoContext) -> Element {
    component_canvas(
        fixed_width(
            shadcn::collapsible(
                "@peduarte starred 3 repositories",
                ctx.toggle_state,
                vec![
                    repo_row("@radix-ui/primitives"),
                    repo_row("@radix-ui/react"),
                    repo_row("@stitches/core"),
                ],
            ),
            350.0,
        ),
        true,
        24.0,
    )
}
