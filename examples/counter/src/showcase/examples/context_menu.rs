use super::super::layout::fixed_width;
use super::shared::{top_center_canvas, DemoContext};
use arkit::prelude::*;
use arkit_shadcn as shadcn;

#[component]
pub(crate) fn render(ctx: DemoContext) -> Element {
    let on_toggle = ctx.on_toggle_state.clone();
    let on_menu_toggle = ctx.on_toggle_state.clone();

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
                    .style(ArkUINodeAttributeType::Clip, true)
                    .on_long_press(move || on_toggle(true))
                    .children(vec![shadcn::text_sm("Long press here")])
                    .into(),
                vec![
                    shadcn::context_menu_item_inset_with_shortcut_action("Back", "CMD+[", || {
                        let _ = back_route();
                    }),
                    shadcn::disabled_context_menu_item_inset_with_shortcut("Forward", "CMD+]"),
                    shadcn::context_menu_item_inset_with_shortcut("Reload", "CMD+R"),
                    shadcn::context_menu_submenu_inset(
                        "More Tools",
                        vec![
                            shadcn::context_menu_item("Save Page..."),
                            shadcn::context_menu_item("Create Shortcut..."),
                            shadcn::context_menu_item("Name Window..."),
                            shadcn::context_menu_separator(),
                            shadcn::context_menu_item("Developer Tools"),
                            shadcn::context_menu_separator(),
                            shadcn::context_menu_item_destructive("Delete"),
                        ],
                    ),
                    shadcn::context_menu_separator(),
                    shadcn::context_menu_checkbox_item(
                        "Show Bookmarks",
                        ctx.context_bookmarks,
                        {
                            let on_toggle = ctx.on_context_bookmarks.clone();
                            move |value| on_toggle(value)
                        },
                    ),
                    shadcn::context_menu_checkbox_item(
                        "Show Full URLs",
                        ctx.context_full_urls,
                        {
                            let on_toggle = ctx.on_context_full_urls.clone();
                            move |value| on_toggle(value)
                        },
                    ),
                    shadcn::context_menu_separator(),
                    shadcn::context_menu_label_inset("People"),
                    shadcn::context_menu_radio_item(
                        "Pedro Duarte",
                        "pedro",
                        ctx.context_person.clone(),
                        {
                            let on_select = ctx.on_context_person.clone();
                            move |value| on_select(value)
                        },
                    ),
                    shadcn::context_menu_radio_item(
                        "Colm Tuite",
                        "colm",
                        ctx.context_person,
                        {
                            let on_select = ctx.on_context_person.clone();
                            move |value| on_select(value)
                        },
                    ),
                ],
                ctx.toggle_state,
                move |value| on_menu_toggle(value),
            ),
            300.0,
        ),
        [24.0, 24.0, 24.0, 24.0],
        true,
    )
}
