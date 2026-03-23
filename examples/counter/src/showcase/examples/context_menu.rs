use super::super::layout::fixed_width;
use super::shared::{top_center_canvas, DemoContext};
use arkit::prelude::*;
use arkit_shadcn as shadcn;

pub(crate) fn render(ctx: DemoContext) -> Element {
    let toggle = ctx.toggle_state.clone();
    top_center_canvas(
        fixed_width(
            shadcn::context_menu(
                arkit::column_component()
                    .width(300.0)
                    .height(150.0)
                    .align_items_center()
                    .style(ArkUINodeAttributeType::ColumnJustifyContent, 2_i32)
                    .style(
                        ArkUINodeAttributeType::BorderWidth,
                        vec![1.0, 1.0, 1.0, 1.0],
                    )
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
                    .style(ArkUINodeAttributeType::BorderStyle, 1_i32)
                    .on_click(move || toggle.update(|open| *open = !*open))
                    .children(vec![shadcn::text_sm("Long press here")])
                    .into(),
                vec![
                    shadcn::dropdown_item("Back"),
                    shadcn::disabled_button("Forward", shadcn::ButtonVariant::Ghost)
                        .height(36.0)
                        .style(
                            ArkUINodeAttributeType::RowJustifyContent,
                            super::super::layout::FLEX_ALIGN_START,
                        )
                        .style(
                            ArkUINodeAttributeType::Padding,
                            vec![
                                8.0,
                                shadcn::theme::spacing::SM,
                                8.0,
                                shadcn::theme::spacing::SM,
                            ],
                        )
                        .style(
                            ArkUINodeAttributeType::BorderRadius,
                            vec![
                                shadcn::theme::radius::SM,
                                shadcn::theme::radius::SM,
                                shadcn::theme::radius::SM,
                                shadcn::theme::radius::SM,
                            ],
                        )
                        .style(ArkUINodeAttributeType::FontWeight, 3_i32)
                        .style(
                            ArkUINodeAttributeType::FontColor,
                            shadcn::theme::color::POPOVER_FOREGROUND,
                        )
                        .into(),
                    shadcn::dropdown_item("Reload"),
                    shadcn::dropdown_item("More Tools"),
                    shadcn::separator(),
                    shadcn::dropdown_item("Show Bookmarks"),
                    shadcn::dropdown_item("Show Full URLs"),
                    shadcn::separator(),
                    arkit::row_component()
                        .style(ArkUINodeAttributeType::Padding, vec![8.0, 8.0, 8.0, 32.0])
                        .children(vec![shadcn::text_sm_medium("People")])
                        .into(),
                    shadcn::dropdown_item("Pedro Duarte"),
                    shadcn::dropdown_item("Colm Tuite"),
                ],
                ctx.toggle_state,
            ),
            384.0,
        ),
        [24.0, 24.0, 24.0, 24.0],
        true,
    )
}
