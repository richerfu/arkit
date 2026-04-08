use super::super::layout::fixed_width;
use super::shared::{top_center_canvas, DemoContext};
use crate::prelude::*;
use crate::Message;
use arkit_shadcn as shadcn;

pub(crate) fn render(ctx: DemoContext) -> Element {
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
                    .on_long_press_message(Message::SetContextMenuOpen(true))
                    .children(vec![shadcn::text_sm("Long press here")])
                    .into(),
                vec![
                    shadcn::context_menu_item_inset_with_shortcut_action(
                        "Back",
                        "CMD+[",
                        Message::Back,
                    ),
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
                        ctx.context_submenu_open,
                        Message::SetContextSubmenuOpen,
                    ),
                    shadcn::context_menu_separator(),
                    shadcn::context_menu_checkbox_item(
                        "Show Bookmarks",
                        ctx.context_bookmarks,
                        Message::SetContextBookmarks,
                    ),
                    shadcn::context_menu_checkbox_item(
                        "Show Full URLs",
                        ctx.context_full_urls,
                        Message::SetContextFullUrls,
                    ),
                    shadcn::context_menu_separator(),
                    shadcn::context_menu_label_inset("People"),
                    shadcn::context_menu_radio_item(
                        "Pedro Duarte",
                        "pedro",
                        ctx.context_person.clone(),
                        Message::SetContextPerson,
                    ),
                    shadcn::context_menu_radio_item(
                        "Colm Tuite",
                        "colm",
                        ctx.context_person,
                        Message::SetContextPerson,
                    ),
                ],
                ctx.context_menu_open,
                Message::SetContextMenuOpen,
            ),
            300.0,
        ),
        [24.0, 24.0, 24.0, 24.0],
        true,
    )
}
