use super::menu_common::{
    menu_action_entry, menu_checkbox_entry, menu_label_entry, menu_popup, menu_radio_entry,
    menu_separator_entry, menu_submenu_entry, MenuEntry, MenuStyle,
};
use super::*;
use std::rc::Rc;

const MENU_PANEL_WIDTH: f32 = 224.0;
const SUBMENU_PANEL_WIDTH: f32 = MENU_PANEL_WIDTH - (spacing::XXS * 2.0);

pub type DropdownMenuEntry = MenuEntry;

pub fn dropdown_menu_message<Message>(
    trigger: Element<Message>,
    items: Vec<DropdownMenuEntry>,
    open: bool,
    on_open_change: impl Fn(bool) -> Message + 'static,
) -> Element<Message>
where
    Message: Send + 'static,
{
    dropdown_menu_aligned_message(
        trigger,
        items,
        open,
        on_open_change,
        super::floating_layer::FloatingAlign::Start,
    )
}

pub fn dropdown_menu_aligned_message<Message>(
    trigger: Element<Message>,
    items: Vec<DropdownMenuEntry>,
    open: bool,
    on_open_change: impl Fn(bool) -> Message + 'static,
    align: super::floating_layer::FloatingAlign,
) -> Element<Message>
where
    Message: Send + 'static,
{
    menu_popup(
        trigger,
        items,
        open,
        move |value| dispatch_message(on_open_change(value)),
        align,
        MenuStyle {
            width: MENU_PANEL_WIDTH,
            submenu_width: SUBMENU_PANEL_WIDTH,
            side_offset_vp: spacing::XXS,
        },
    )
}

pub fn dropdown_item(title: impl Into<String>) -> DropdownMenuEntry {
    menu_action_entry(title, None, false, false, false, None)
}

pub fn dropdown_item_destructive(title: impl Into<String>) -> DropdownMenuEntry {
    menu_action_entry(title, None, true, false, false, None)
}

pub fn dropdown_item_inset(title: impl Into<String>) -> DropdownMenuEntry {
    menu_action_entry(title, None, false, false, true, None)
}

pub fn dropdown_item_with_shortcut(
    title: impl Into<String>,
    shortcut: impl Into<String>,
) -> DropdownMenuEntry {
    menu_action_entry(title, Some(shortcut.into()), false, false, false, None)
}

pub fn dropdown_item_inset_with_shortcut(
    title: impl Into<String>,
    shortcut: impl Into<String>,
) -> DropdownMenuEntry {
    menu_action_entry(title, Some(shortcut.into()), false, false, true, None)
}

pub fn disabled_dropdown_item(title: impl Into<String>) -> DropdownMenuEntry {
    menu_action_entry(title, None, false, true, false, None)
}

pub fn disabled_dropdown_item_with_shortcut(
    title: impl Into<String>,
    shortcut: impl Into<String>,
) -> DropdownMenuEntry {
    menu_action_entry(title, Some(shortcut.into()), false, true, false, None)
}

pub fn dropdown_subcontent(items: Vec<DropdownMenuEntry>) -> Vec<DropdownMenuEntry> {
    items
}

pub fn dropdown_submenu_message(
    title: impl Into<String>,
    items: Vec<DropdownMenuEntry>,
) -> DropdownMenuEntry {
    menu_submenu_entry(title, false, items)
}

pub fn dropdown_checkbox_item_message<Message>(
    title: impl Into<String>,
    checked: bool,
    on_toggle: impl Fn(bool) -> Message + 'static,
) -> DropdownMenuEntry
where
    Message: Send + 'static,
{
    menu_checkbox_entry(
        title,
        checked,
        Rc::new(move |value| dispatch_message(on_toggle(value))),
    )
}

pub fn dropdown_label(title: impl Into<String>) -> DropdownMenuEntry {
    menu_label_entry(title, false)
}

pub fn dropdown_label_inset(title: impl Into<String>) -> DropdownMenuEntry {
    menu_label_entry(title, true)
}

pub fn dropdown_separator() -> DropdownMenuEntry {
    menu_separator_entry()
}

pub fn dropdown_radio_item_message<Message>(
    title: impl Into<String>,
    value: impl Into<String>,
    selected: impl Into<String>,
    on_select: impl Fn(String) -> Message + 'static,
) -> DropdownMenuEntry
where
    Message: Send + 'static,
{
    menu_radio_entry(
        title,
        value,
        selected,
        Rc::new(move |value| dispatch_message(on_select(value))),
    )
}
