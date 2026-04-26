use super::menu_common::{
    menu_action_entry, menu_checkbox_entry, menu_label_entry, menu_popup, menu_radio_entry,
    menu_separator_entry, menu_submenu_entry, MenuEntry, MenuStyle,
};
use super::*;
use std::rc::Rc;

const MENU_PANEL_WIDTH: f32 = 208.0;
const SUBMENU_PANEL_WIDTH: f32 = MENU_PANEL_WIDTH - (spacing::XXS * 2.0);

pub type ContextMenuEntry = MenuEntry;

fn context_menu_message<Message>(
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

fn context_menu_item(title: impl Into<String>) -> ContextMenuEntry {
    menu_action_entry(title, None, false, false, false, None)
}

fn context_menu_item_destructive(title: impl Into<String>) -> ContextMenuEntry {
    menu_action_entry(title, None, true, false, false, None)
}

fn context_menu_item_inset(title: impl Into<String>) -> ContextMenuEntry {
    menu_action_entry(title, None, false, false, true, None)
}

fn context_menu_item_inset_destructive(title: impl Into<String>) -> ContextMenuEntry {
    menu_action_entry(title, None, true, false, true, None)
}

fn context_menu_item_with_shortcut(
    title: impl Into<String>,
    shortcut: impl Into<String>,
) -> ContextMenuEntry {
    menu_action_entry(title, Some(shortcut.into()), false, false, false, None)
}

fn context_menu_item_inset_with_shortcut(
    title: impl Into<String>,
    shortcut: impl Into<String>,
) -> ContextMenuEntry {
    menu_action_entry(title, Some(shortcut.into()), false, false, true, None)
}

fn context_menu_item_inset_with_shortcut_action_message<Message>(
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

fn disabled_context_menu_item_inset_with_shortcut(
    title: impl Into<String>,
    shortcut: impl Into<String>,
) -> ContextMenuEntry {
    menu_action_entry(title, Some(shortcut.into()), false, true, true, None)
}

fn context_menu_sub_trigger_inset(title: impl Into<String>) -> ContextMenuEntry {
    menu_submenu_entry(title, true, Vec::new())
}

fn context_menu_subcontent(items: Vec<ContextMenuEntry>) -> Vec<ContextMenuEntry> {
    items
}

fn context_menu_submenu_inset_message(
    title: impl Into<String>,
    items: Vec<ContextMenuEntry>,
) -> ContextMenuEntry {
    menu_submenu_entry(title, true, items)
}

fn context_menu_checkbox_item_message<Message>(
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

fn context_menu_label(title: impl Into<String>) -> ContextMenuEntry {
    menu_label_entry(title, false)
}

fn context_menu_label_inset(title: impl Into<String>) -> ContextMenuEntry {
    menu_label_entry(title, true)
}

fn context_menu_separator() -> ContextMenuEntry {
    menu_separator_entry()
}

fn context_menu_radio_item_message<Message>(
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

// Struct component API
pub struct ContextMenu<Message = ()> {
    trigger: std::cell::RefCell<Option<Element<Message>>>,
    items: Vec<super::menu_common::MenuEntry>,
    open: Option<bool>,
    default_open: bool,
    on_open_change: Option<std::rc::Rc<dyn Fn(bool) -> Message>>,
}

impl<Message> ContextMenu<Message> {
    pub fn new(trigger: Element<Message>, items: Vec<super::menu_common::MenuEntry>) -> Self {
        Self {
            trigger: std::cell::RefCell::new(Some(trigger)),
            items,
            open: None,
            default_open: false,
            on_open_change: None,
        }
    }

    pub fn open(mut self, open: bool) -> Self {
        self.open = Some(open);
        self
    }

    pub fn default_open(mut self, open: bool) -> Self {
        self.default_open = open;
        self
    }

    pub fn on_open_change(mut self, handler: impl Fn(bool) -> Message + 'static) -> Self {
        self.on_open_change = Some(std::rc::Rc::new(handler));
        self
    }
}

impl<Message: Send + 'static> arkit::advanced::Widget<Message, arkit::Theme, arkit::Renderer>
    for ContextMenu<Message>
{
    fn body(
        &self,
        tree: &mut arkit::advanced::widget::Tree,
        _renderer: &arkit::Renderer,
    ) -> Option<Element<Message>> {
        let state = super::widget_state(tree, || self.default_open);
        let is_controlled = self.open.is_some();
        let open = self.open.unwrap_or_else(|| *state.borrow());
        let handler = self.on_open_change.clone();
        let mut trigger = super::take_component_slot(&self.trigger, "context menu trigger");
        if !is_controlled {
            let trigger_state = state.clone();
            let trigger_handler = handler.clone();
            trigger = arkit::row_component::<Message, arkit::Theme>()
                .on_click(move || {
                    let next = !open;
                    *trigger_state.borrow_mut() = next;
                    super::request_widget_rerender();
                    if let Some(handler) = trigger_handler.as_ref() {
                        dispatch_message(handler(next));
                    }
                })
                .children(vec![trigger])
                .into();
        }

        Some(menu_popup(
            trigger,
            self.items.clone(),
            open,
            move |value| {
                if !is_controlled {
                    *state.borrow_mut() = value;
                    super::request_widget_rerender();
                }
                if let Some(handler) = handler.as_ref() {
                    dispatch_message(handler(value));
                }
            },
            super::floating_layer::FloatingAlign::Start,
            MenuStyle {
                width: MENU_PANEL_WIDTH,
                submenu_width: SUBMENU_PANEL_WIDTH,
                side_offset_vp: spacing::XXS,
            },
        ))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn into_any(self: Box<Self>) -> Box<dyn std::any::Any> {
        self
    }
}

impl<Message: Send + 'static> From<ContextMenu<Message>> for Element<Message> {
    fn from(value: ContextMenu<Message>) -> Self {
        Element::new(value)
    }
}
