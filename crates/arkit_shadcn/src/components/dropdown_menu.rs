use super::menu_common::{
    menu_action_entry, menu_checkbox_entry, menu_label_entry, menu_popup, menu_radio_entry,
    menu_separator_entry, menu_submenu_entry, MenuEntry, MenuStyle,
};
use super::*;
use std::rc::Rc;

const MENU_PANEL_WIDTH: f32 = 224.0;
const SUBMENU_PANEL_WIDTH: f32 = MENU_PANEL_WIDTH - (spacing::XXS * 2.0);

pub type DropdownMenuEntry = MenuEntry;

fn dropdown_menu_message<Message>(
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

fn dropdown_menu_aligned_message<Message>(
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

fn dropdown_item(title: impl Into<String>) -> DropdownMenuEntry {
    menu_action_entry(title, None, false, false, false, None)
}

fn dropdown_item_destructive(title: impl Into<String>) -> DropdownMenuEntry {
    menu_action_entry(title, None, true, false, false, None)
}

fn dropdown_item_inset(title: impl Into<String>) -> DropdownMenuEntry {
    menu_action_entry(title, None, false, false, true, None)
}

fn dropdown_item_with_shortcut(
    title: impl Into<String>,
    shortcut: impl Into<String>,
) -> DropdownMenuEntry {
    menu_action_entry(title, Some(shortcut.into()), false, false, false, None)
}

fn dropdown_item_inset_with_shortcut(
    title: impl Into<String>,
    shortcut: impl Into<String>,
) -> DropdownMenuEntry {
    menu_action_entry(title, Some(shortcut.into()), false, false, true, None)
}

fn disabled_dropdown_item(title: impl Into<String>) -> DropdownMenuEntry {
    menu_action_entry(title, None, false, true, false, None)
}

fn disabled_dropdown_item_with_shortcut(
    title: impl Into<String>,
    shortcut: impl Into<String>,
) -> DropdownMenuEntry {
    menu_action_entry(title, Some(shortcut.into()), false, true, false, None)
}

fn dropdown_subcontent(items: Vec<DropdownMenuEntry>) -> Vec<DropdownMenuEntry> {
    items
}

fn dropdown_submenu_message(
    title: impl Into<String>,
    items: Vec<DropdownMenuEntry>,
) -> DropdownMenuEntry {
    menu_submenu_entry(title, false, items)
}

fn dropdown_checkbox_item_message<Message>(
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

fn dropdown_label(title: impl Into<String>) -> DropdownMenuEntry {
    menu_label_entry(title, false)
}

fn dropdown_label_inset(title: impl Into<String>) -> DropdownMenuEntry {
    menu_label_entry(title, true)
}

fn dropdown_separator() -> DropdownMenuEntry {
    menu_separator_entry()
}

fn dropdown_radio_item_message<Message>(
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

// Struct component API
pub struct DropdownMenu<Message = ()> {
    trigger: std::cell::RefCell<Option<Element<Message>>>,
    items: Vec<super::menu_common::MenuEntry>,
    open: Option<bool>,
    default_open: bool,
    align: super::floating_layer::FloatingAlign,
    on_open_change: Option<std::rc::Rc<dyn Fn(bool) -> Message>>,
}

impl<Message> DropdownMenu<Message> {
    pub fn new(trigger: Element<Message>, items: Vec<super::menu_common::MenuEntry>) -> Self {
        Self {
            trigger: std::cell::RefCell::new(Some(trigger)),
            items,
            open: None,
            default_open: false,
            align: super::floating_layer::FloatingAlign::Start,
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

    pub fn align(mut self, align: super::floating_layer::FloatingAlign) -> Self {
        self.align = align;
        self
    }

    pub fn on_open_change(mut self, handler: impl Fn(bool) -> Message + 'static) -> Self {
        self.on_open_change = Some(std::rc::Rc::new(handler));
        self
    }
}

impl<Message: Send + 'static> arkit::advanced::Widget<Message, arkit::Theme, arkit::Renderer>
    for DropdownMenu<Message>
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
        let mut trigger = super::take_component_slot(&self.trigger, "dropdown trigger");
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
            self.align,
            MenuStyle {
                width: MENU_PANEL_WIDTH,
                submenu_width: SUBMENU_PANEL_WIDTH,
                side_offset_vp: spacing::XXS,
            },
        ))
    }
}

impl<Message: Send + 'static> From<DropdownMenu<Message>> for Element<Message> {
    fn from(value: DropdownMenu<Message>) -> Self {
        Element::new(value)
    }
}
