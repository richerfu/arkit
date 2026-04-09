use super::shared::{top_center_canvas, DemoContext};
use crate::prelude::*;
use crate::Message;
use arkit_shadcn as shadcn;

pub(crate) fn render(ctx: DemoContext) -> Element {
    top_center_canvas(
        shadcn::menubar_with_menus(
            vec![
                shadcn::menubar_menu(
                    "File",
                    vec![
                        shadcn::dropdown_item_with_shortcut("New Tab", "⌘T"),
                        shadcn::dropdown_item_with_shortcut("New Window", "⌘N"),
                        shadcn::disabled_dropdown_item("New Incognito Window"),
                        shadcn::dropdown_separator(),
                        shadcn::dropdown_submenu(
                            "Share",
                            vec![
                                shadcn::dropdown_item("Email link"),
                                shadcn::dropdown_item("Messages"),
                                shadcn::dropdown_item("Notes"),
                            ],
                        ),
                        shadcn::dropdown_separator(),
                        shadcn::dropdown_item_with_shortcut("Print...", "⌘P"),
                    ],
                ),
                shadcn::menubar_menu(
                    "Edit",
                    vec![
                        shadcn::dropdown_item_with_shortcut("Undo", "⌘Z"),
                        shadcn::dropdown_item_with_shortcut("Redo", "⇧⌘Z"),
                        shadcn::dropdown_separator(),
                        shadcn::dropdown_submenu(
                            "Find",
                            vec![
                                shadcn::dropdown_item("Search the web"),
                                shadcn::dropdown_separator(),
                                shadcn::dropdown_item("Find..."),
                                shadcn::dropdown_item("Find Next"),
                                shadcn::dropdown_item("Find Previous"),
                            ],
                        ),
                        shadcn::dropdown_separator(),
                        shadcn::dropdown_item("Cut"),
                        shadcn::dropdown_item("Copy"),
                        shadcn::dropdown_item("Paste"),
                    ],
                ),
                shadcn::menubar_menu(
                    "View",
                    vec![
                        shadcn::dropdown_checkbox_item(
                            "Always Show Bookmarks Bar",
                            ctx.context_bookmarks,
                            Message::SetContextBookmarks,
                        ),
                        shadcn::dropdown_checkbox_item(
                            "Always Show Full URLs",
                            ctx.context_full_urls,
                            Message::SetContextFullUrls,
                        ),
                        shadcn::dropdown_separator(),
                        shadcn::dropdown_item_with_shortcut("Reload", "⌘R"),
                        shadcn::dropdown_separator(),
                        shadcn::dropdown_item_with_shortcut("Toggle Fullscreen", ""),
                        shadcn::dropdown_separator(),
                        shadcn::dropdown_item("Hide Sidebar"),
                    ],
                ),
                shadcn::menubar_menu(
                    "Profiles",
                    vec![
                        shadcn::dropdown_radio_item(
                            "Andy",
                            "andy",
                            ctx.context_person.clone(),
                            Message::SetContextPerson,
                        ),
                        shadcn::dropdown_radio_item(
                            "Benoit",
                            "benoit",
                            ctx.context_person.clone(),
                            Message::SetContextPerson,
                        ),
                        shadcn::dropdown_radio_item(
                            "Luis",
                            "luis",
                            ctx.context_person,
                            Message::SetContextPerson,
                        ),
                        shadcn::dropdown_separator(),
                        shadcn::dropdown_item("Edit..."),
                        shadcn::dropdown_separator(),
                        shadcn::dropdown_item("Add Profile..."),
                    ],
                ),
            ],
            ctx.menubar_active,
            Message::SetMenubarActive,
        ),
        [16.0, 16.0, 16.0, 16.0],
        true,
    )
}
