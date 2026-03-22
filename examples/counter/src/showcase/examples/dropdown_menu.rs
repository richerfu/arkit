use super::super::layout::fixed_width;
use super::shared::{top_center_canvas, DemoContext};
use arkit::prelude::*;
use arkit_shadcn as shadcn;

pub(crate) fn render(ctx: DemoContext) -> Element {
    let toggle = ctx.toggle_state.clone();
    top_center_canvas(
        fixed_width(
            shadcn::dropdown_menu(
                shadcn::button("Open", shadcn::ButtonVariant::Outline)
                    .on_click(move || toggle.update(|open| *open = !*open))
                    .into(),
                vec![
                    arkit::column_component()
                        .style(
                            ArkUINodeAttributeType::Padding,
                            vec![8.0, 8.0, 8.0, 8.0],
                        )
                        .children(vec![
                            shadcn::text_sm_medium("My Account"),
                        ])
                        .into(),
                    shadcn::separator(),
                    shadcn::dropdown_item("Profile"),
                    shadcn::dropdown_item("Billing"),
                    shadcn::dropdown_item("Settings"),
                    shadcn::dropdown_item("Keyboard shortcuts"),
                    shadcn::separator(),
                    shadcn::dropdown_item("Team"),
                    shadcn::dropdown_item("Invite users"),
                    shadcn::dropdown_item("New Team"),
                    shadcn::separator(),
                    shadcn::dropdown_item("GitHub"),
                    shadcn::dropdown_item("Support"),
                    shadcn::disabled_button("API", shadcn::ButtonVariant::Ghost)
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
                    shadcn::separator(),
                    shadcn::dropdown_item_destructive("Log out"),
                ],
                ctx.toggle_state,
            ),
            384.0,
        ),
        [24.0, 24.0, 24.0, 24.0],
        true,
    )
}
