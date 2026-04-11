use super::shared::{button_carousel, no_padding_center_canvas, DemoContext};
use crate::prelude::*;
use arkit_shadcn as shadcn;

pub(crate) fn render(ctx: DemoContext) -> Element {
    let mut children = vec![button_carousel(ctx.page).into()];
    if let Some(feedback) = ctx.button_preview_feedback {
        children.push(
            arkit::row_component()
                .style(ArkUINodeAttributeType::Margin, vec![16.0, 0.0, 0.0, 0.0])
                .children(vec![shadcn::text_with_variant(feedback, shadcn::TextVariant::Muted)])
                .into(),
        );
    }

    no_padding_center_canvas(
        arkit::column_component()
            .percent_width(1.0)
            .percent_height(1.0)
            .align_items_center()
            .style(ArkUINodeAttributeType::ColumnJustifyContent, 2_i32)
            .children(children)
            .into(),
    )
}
