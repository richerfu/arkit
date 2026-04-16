use super::menu_common::{
    menu_action_entry, menu_checkbox_entry, menu_label_entry, menu_popup, menu_radio_entry,
    menu_separator_entry, menu_submenu_entry, MenuEntry, MenuStyle,
};
use super::*;
use std::rc::Rc;

const MENU_PANEL_WIDTH: f32 = 208.0;
const SUBMENU_PANEL_WIDTH: f32 = MENU_PANEL_WIDTH - (spacing::XXS * 2.0);

pub type ContextMenuEntry = MenuEntry;

pub fn context_menu_message<Message>(
    trigger: Element<Message>,
    items: Vec<ContextMenuEntry>,
    open: bool,
    on_open_change: impl Fn(bool) -> Message + 'static,
) -> Element<Message>
where
    Message: Send + 'static,
{
    menu_popup(
        trigger,
        items,
        open,
        move |value| dispatch_message(on_open_change(value)),
        super::floating_layer::FloatingAlign::Start,
        MenuStyle {
            width: MENU_PANEL_WIDTH,
            submenu_width: SUBMENU_PANEL_WIDTH,
            side_offset_vp: spacing::XXS,
        },
    )
}

pub fn context_menu_item(title: impl Into<String>) -> ContextMenuEntry {
    menu_action_entry(title, None, false, false, false, None)
}

pub fn context_menu_item_destructive(title: impl Into<String>) -> ContextMenuEntry {
    menu_action_entry(title, None, true, false, false, None)
}

pub fn context_menu_item_inset(title: impl Into<String>) -> ContextMenuEntry {
    menu_action_entry(title, None, false, false, true, None)
}

pub fn context_menu_item_inset_destructive(title: impl Into<String>) -> ContextMenuEntry {
    menu_action_entry(title, None, true, false, true, None)
}

pub fn context_menu_item_with_shortcut(
    title: impl Into<String>,
    shortcut: impl Into<String>,
) -> ContextMenuEntry {
    menu_action_entry(title, Some(shortcut.into()), false, false, false, None)
}

pub fn context_menu_item_inset_with_shortcut(
    title: impl Into<String>,
    shortcut: impl Into<String>,
) -> ContextMenuEntry {
    menu_action_entry(title, Some(shortcut.into()), false, false, true, None)
}

pub fn context_menu_item_inset_with_shortcut_action_message<Message>(
    title: impl Into<String>,
    shortcut: impl Into<String>,
    on_select: Message,
) -> ContextMenuEntry
where
    Message: Clone + Send + 'static,
{
    menu_action_entry(
        title,
        Some(shortcut.into()),
        false,
        false,
        true,
        Some(Rc::new(move || dispatch_message(on_select.clone()))),
    )
}

pub fn disabled_context_menu_item_inset_with_shortcut(
    title: impl Into<String>,
    shortcut: impl Into<String>,
) -> ContextMenuEntry {
    menu_action_entry(title, Some(shortcut.into()), false, true, true, None)
}

pub fn context_menu_sub_trigger_inset(title: impl Into<String>) -> ContextMenuEntry {
    menu_submenu_entry(title, true, Vec::new())
}

pub fn context_menu_subcontent(items: Vec<ContextMenuEntry>) -> Vec<ContextMenuEntry> {
    items
}

pub fn context_menu_submenu_inset_message(
    title: impl Into<String>,
    items: Vec<ContextMenuEntry>,
) -> ContextMenuEntry {
    menu_submenu_entry(title, true, items)
}

pub fn context_menu_checkbox_item_message<Message>(
    title: impl Into<String>,
    checked: bool,
    on_toggle: impl Fn(bool) -> Message + 'static,
) -> ContextMenuEntry
where
    Message: Send + 'static,
{
    menu_checkbox_entry(
        title,
        checked,
        Rc::new(move |value| dispatch_message(on_toggle(value))),
    )
}

pub fn context_menu_label(title: impl Into<String>) -> ContextMenuEntry {
    menu_label_entry(title, false)
}

pub fn context_menu_label_inset(title: impl Into<String>) -> ContextMenuEntry {
    menu_label_entry(title, true)
}

pub fn context_menu_separator() -> ContextMenuEntry {
    menu_separator_entry()
}

pub fn context_menu_radio_item_message<Message>(
    title: impl Into<String>,
    value: impl Into<String>,
    selected: impl Into<String>,
    on_select: impl Fn(String) -> Message + 'static,
) -> ContextMenuEntry
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
