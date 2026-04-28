use super::super::layout::fixed_width;
use super::shared::{top_center_canvas, DemoContext};
use crate::prelude::*;
use arkit_shadcn as shadcn;

pub(crate) struct DropdownMenuExample {
    ctx: DemoContext,
}

impl DropdownMenuExample {
    pub(crate) fn new(ctx: DemoContext) -> Self {
        Self { ctx }
    }
}

impl arkit::advanced::Widget<crate::Message, arkit::Theme, arkit::Renderer>
    for DropdownMenuExample
{
    fn body(
        &self,
        _tree: &mut arkit::advanced::widget::Tree,
        _renderer: &arkit::Renderer,
    ) -> Option<Element> {
        let _ctx = self.ctx.clone();
        Some({
            top_center_canvas(
                fixed_width(
                    shadcn::DropdownMenu::new(
                        shadcn::Button::new("Open")
                            .theme(shadcn::ButtonVariant::Outline)
                            .into(),
                        vec![
                            shadcn::MenuEntry::label("My Account"),
                            shadcn::MenuEntry::separator(),
                            shadcn::MenuEntry::action("Profile").shortcut("⇧⌘P"),
                            shadcn::MenuEntry::action("Billing").shortcut("⌘B"),
                            shadcn::MenuEntry::action("Settings").shortcut("⌘S"),
                            shadcn::MenuEntry::action("Keyboard shortcuts").shortcut("⌘K"),
                            shadcn::MenuEntry::separator(),
                            shadcn::MenuEntry::action("Team"),
                            shadcn::MenuEntry::submenu(
                                "Invite users",
                                vec![
                                    shadcn::MenuEntry::action("Email"),
                                    shadcn::MenuEntry::action("Message"),
                                    shadcn::MenuEntry::separator(),
                                    shadcn::MenuEntry::action("More..."),
                                ],
                            ),
                            shadcn::MenuEntry::action("New Team").shortcut("⌘+T"),
                            shadcn::MenuEntry::separator(),
                            shadcn::MenuEntry::action("GitHub"),
                            shadcn::MenuEntry::action("Support"),
                            shadcn::MenuEntry::action("API"),
                            shadcn::MenuEntry::separator(),
                            shadcn::MenuEntry::action("Log out").shortcut("⇧⌘Q"),
                        ],
                    )
                    .default_open(false)
                    .into(),
                    384.0,
                ),
                [24.0, 24.0, 24.0, 24.0],
                true,
            )
        })
    }
}

// struct component render
