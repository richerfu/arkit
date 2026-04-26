use super::super::layout::fixed_width;
use super::shared::{top_center_canvas, DemoContext};
use crate::prelude::*;
use crate::Message;
use arkit_shadcn as shadcn;

pub(crate) struct ContextMenuExample {
    ctx: DemoContext,
}

impl ContextMenuExample {
    pub(crate) fn new(ctx: DemoContext) -> Self {
        Self { ctx }
    }
}

impl arkit::advanced::Widget<crate::Message, arkit::Theme, arkit::Renderer> for ContextMenuExample {
    fn body(
        &self,
        _tree: &mut arkit::advanced::widget::Tree,
        _renderer: &arkit::Renderer,
    ) -> Option<Element> {
        let ctx = self.ctx.clone();
        Some({
            top_center_canvas(
                fixed_width(
                    shadcn::ContextMenu::new(
                        arkit::column_component()
                            .width(300.0)
                            .height(150.0)
                            .align_items_center()
                            .justify_content(JustifyContent::Center)
                            .border_width([1.0, 1.0, 1.0, 1.0])
                            .border_color(shadcn::theme::colors().border)
                            .border_radius([
                                shadcn::theme::radii().md,
                                shadcn::theme::radii().md,
                                shadcn::theme::radii().md,
                                shadcn::theme::radii().md,
                            ])
                            .border_style(BorderStyle::Dashed)
                            .clip(true)
                            .on_long_press_message(Message::SetContextMenuOpen(true))
                            .children(vec![shadcn::Text::small("Long press here").into()])
                            .into(),
                        vec![
                            shadcn::MenuEntry::action("Back")
                                .inset()
                                .shortcut("CMD+[")
                                .on_select_message(Message::Back),
                            shadcn::MenuEntry::action("Forward")
                                .inset()
                                .shortcut("CMD+]")
                                .disabled(),
                            shadcn::MenuEntry::action("Reload").shortcut("CMD+R"),
                            shadcn::MenuEntry::submenu(
                                "More Tools",
                                vec![
                                    shadcn::MenuEntry::action("Save Page..."),
                                    shadcn::MenuEntry::action("Create Shortcut..."),
                                    shadcn::MenuEntry::action("Name Window..."),
                                    shadcn::MenuEntry::separator(),
                                    shadcn::MenuEntry::action("Developer Tools"),
                                    shadcn::MenuEntry::separator(),
                                    shadcn::MenuEntry::action("Delete").destructive(),
                                ],
                            ),
                            shadcn::MenuEntry::separator(),
                            shadcn::MenuEntry::checkbox(
                                "Show Bookmarks",
                                ctx.context_bookmarks,
                                Message::SetContextBookmarks,
                            ),
                            shadcn::MenuEntry::checkbox(
                                "Show Full URLs",
                                ctx.context_full_urls,
                                Message::SetContextFullUrls,
                            ),
                            shadcn::MenuEntry::separator(),
                            shadcn::MenuEntry::label("People").inset(),
                            shadcn::MenuEntry::radio(
                                "Pedro Duarte",
                                "pedro",
                                ctx.context_person.clone(),
                                Message::SetContextPerson,
                            ),
                            shadcn::MenuEntry::radio(
                                "Colm Tuite",
                                "colm",
                                ctx.context_person,
                                Message::SetContextPerson,
                            ),
                        ],
                    )
                    .open(ctx.context_menu_open)
                    .on_open_change(Message::SetContextMenuOpen)
                    .into(),
                    300.0,
                ),
                [24.0, 24.0, 24.0, 24.0],
                true,
            )
        })
    }
}

// struct component render
