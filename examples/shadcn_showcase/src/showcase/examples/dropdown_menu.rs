use super::super::layout::fixed_width;
use super::shared::{top_center_canvas, DemoContext};
use crate::prelude::*;
use crate::Message;
use arkit_shadcn as shadcn;

pub(crate) fn render(ctx: DemoContext) -> Element {
    top_center_canvas(
        fixed_width(
            shadcn::dropdown_menu(
                shadcn::button("Open", shadcn::ButtonVariant::Outline)
                    .on_press(Message::SetDropdownMenuOpen(!ctx.dropdown_menu_open))
                    .into(),
                vec![
                    shadcn::dropdown_label("My Account"),
                    shadcn::dropdown_separator(),
                    shadcn::dropdown_item_with_shortcut("Profile", "⇧⌘P"),
                    shadcn::dropdown_item_with_shortcut("Billing", "⌘B"),
                    shadcn::dropdown_item_with_shortcut("Settings", "⌘S"),
                    shadcn::dropdown_item_with_shortcut("Keyboard shortcuts", "⌘K"),
                    shadcn::dropdown_separator(),
                    shadcn::dropdown_item("Team"),
                    shadcn::dropdown_submenu(
                        "Invite users",
                        vec![
                            shadcn::dropdown_item("Email"),
                            shadcn::dropdown_item("Message"),
                            shadcn::dropdown_separator(),
                            shadcn::dropdown_item("More..."),
                        ],
                        ctx.dropdown_submenu_open,
                        Message::SetDropdownSubmenuOpen,
                    ),
                    shadcn::dropdown_item_with_shortcut("New Team", "⌘+T"),
                    shadcn::dropdown_separator(),
                    shadcn::dropdown_item("GitHub"),
                    shadcn::dropdown_item("Support"),
                    shadcn::disabled_dropdown_item("API"),
                    shadcn::dropdown_separator(),
                    shadcn::dropdown_item_with_shortcut("Log out", "⇧⌘Q"),
                ],
                ctx.dropdown_menu_open,
                Message::SetDropdownMenuOpen,
            ),
            384.0,
        ),
        [24.0, 24.0, 24.0, 24.0],
        true,
    )
}
