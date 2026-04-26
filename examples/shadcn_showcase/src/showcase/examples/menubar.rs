use super::shared::{top_center_canvas, DemoContext};
use crate::prelude::*;
use crate::Message;
use arkit_shadcn as shadcn;

pub(crate) struct MenubarExample {
    ctx: DemoContext,
}

impl MenubarExample {
    pub(crate) fn new(ctx: DemoContext) -> Self {
        Self { ctx }
    }
}

impl arkit::advanced::Widget<crate::Message, arkit::Theme, arkit::Renderer> for MenubarExample {
    fn body(
        &self,
        _tree: &mut arkit::advanced::widget::Tree,
        _renderer: &arkit::Renderer,
    ) -> Option<Element> {
        let ctx = self.ctx.clone();
        Some({
            top_center_canvas(
                shadcn::Menubar::new(vec![
                    shadcn::MenubarMenuSpec::new(
                        "File",
                        vec![
                            shadcn::MenuEntry::action("New Tab").shortcut("⌘T"),
                            shadcn::MenuEntry::action("New Window").shortcut("⌘N"),
                            shadcn::MenuEntry::action("New Incognito Window"),
                            shadcn::MenuEntry::separator(),
                            shadcn::MenuEntry::submenu(
                                "Share",
                                vec![
                                    shadcn::MenuEntry::action("Email link"),
                                    shadcn::MenuEntry::action("Messages"),
                                    shadcn::MenuEntry::action("Notes"),
                                ],
                            ),
                            shadcn::MenuEntry::separator(),
                            shadcn::MenuEntry::action("Print...").shortcut("⌘P"),
                        ],
                    ),
                    shadcn::MenubarMenuSpec::new(
                        "Edit",
                        vec![
                            shadcn::MenuEntry::action("Undo").shortcut("⌘Z"),
                            shadcn::MenuEntry::action("Redo").shortcut("⇧⌘Z"),
                            shadcn::MenuEntry::separator(),
                            shadcn::MenuEntry::submenu(
                                "Find",
                                vec![
                                    shadcn::MenuEntry::action("Search the web"),
                                    shadcn::MenuEntry::separator(),
                                    shadcn::MenuEntry::action("Find..."),
                                    shadcn::MenuEntry::action("Find Next"),
                                    shadcn::MenuEntry::action("Find Previous"),
                                ],
                            ),
                            shadcn::MenuEntry::separator(),
                            shadcn::MenuEntry::action("Cut"),
                            shadcn::MenuEntry::action("Copy"),
                            shadcn::MenuEntry::action("Paste"),
                        ],
                    ),
                    shadcn::MenubarMenuSpec::new(
                        "View",
                        vec![
                            shadcn::MenuEntry::checkbox(
                                "Always Show Bookmarks Bar",
                                ctx.context_bookmarks,
                                Message::SetContextBookmarks,
                            ),
                            shadcn::MenuEntry::checkbox(
                                "Always Show Full URLs",
                                ctx.context_full_urls,
                                Message::SetContextFullUrls,
                            ),
                            shadcn::MenuEntry::separator(),
                            shadcn::MenuEntry::action("Reload").shortcut("⌘R"),
                            shadcn::MenuEntry::separator(),
                            shadcn::MenuEntry::action("Toggle Fullscreen").shortcut(""),
                            shadcn::MenuEntry::separator(),
                            shadcn::MenuEntry::action("Hide Sidebar"),
                        ],
                    ),
                    shadcn::MenubarMenuSpec::new(
                        "Profiles",
                        vec![
                            shadcn::MenuEntry::radio(
                                "Andy",
                                "andy",
                                ctx.context_person.clone(),
                                Message::SetContextPerson,
                            ),
                            shadcn::MenuEntry::radio(
                                "Benoit",
                                "benoit",
                                ctx.context_person.clone(),
                                Message::SetContextPerson,
                            ),
                            shadcn::MenuEntry::radio(
                                "Luis",
                                "luis",
                                ctx.context_person,
                                Message::SetContextPerson,
                            ),
                            shadcn::MenuEntry::separator(),
                            shadcn::MenuEntry::action("Edit..."),
                            shadcn::MenuEntry::separator(),
                            shadcn::MenuEntry::action("Add Profile..."),
                        ],
                    ),
                ])
                .active(ctx.menubar_active)
                .on_active_change(Message::SetMenubarActive)
                .into(),
                [16.0, 16.0, 16.0, 16.0],
                true,
            )
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn into_any(self: Box<Self>) -> Box<dyn std::any::Any> {
        self
    }
}

// struct component render
