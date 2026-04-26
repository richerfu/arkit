use super::menu_common::{menu_popup, MenuEntry, MenuStyle};
use super::*;
use std::rc::Rc;

const MENU_PANEL_WIDTH: f32 = 224.0;
const SUBMENU_PANEL_WIDTH: f32 = MENU_PANEL_WIDTH - (spacing::XXS * 2.0);
const MENU_PANEL_SIDE_OFFSET: f32 = spacing::SM;
const MENUBAR_ITEM_TRANSPARENT: u32 = 0x00000000;

pub type MenubarEntry = MenuEntry;

#[derive(Clone)]
pub struct MenubarMenuSpec {
    pub title: String,
    pub items: Vec<MenubarEntry>,
}

impl MenubarMenuSpec {
    pub fn new(title: impl Into<String>, items: Vec<MenubarEntry>) -> Self {
        Self {
            title: title.into(),
            items,
        }
    }
}

fn menubar_menu(title: impl Into<String>, items: Vec<MenubarEntry>) -> MenubarMenuSpec {
    MenubarMenuSpec::new(title, items)
}

fn menubar_trigger_surface<Message: 'static>(
    title: impl Into<String>,
    active: bool,
) -> Element<Message> {
    arkit::row_component::<Message, arkit::Theme>()
        .height(28.0)
        .align_items_center()
        .justify_content_center()
        .padding([spacing::XXS, spacing::SM, spacing::XXS, spacing::SM])
        .border_radius([radii().sm, radii().sm, radii().sm, radii().sm])
        .background_color(if active {
            colors().accent
        } else {
            MENUBAR_ITEM_TRANSPARENT
        })
        .children(vec![arkit::text::<Message, arkit::Theme>(title)
            .font_size(typography::SM)
            .font_weight(FontWeight::W500)
            .font_color(colors().foreground)
            .line_height(20.0)
            .into()])
        .into()
}

fn menubar_impl<Message>(
    menus: Vec<MenubarMenuSpec>,
    active: Option<usize>,
    on_active_change: impl Fn(Option<usize>) + 'static,
) -> Element<Message>
where
    Message: 'static,
{
    let on_active_change = Rc::new(on_active_change);

    let items: Vec<Element<Message>> = menus
        .into_iter()
        .enumerate()
        .map(|(index, spec)| {
            let is_active = active == Some(index);
            let on_open = on_active_change.clone();
            let on_close = on_active_change.clone();
            let trigger = menubar_trigger_surface::<Message>(spec.title, is_active).into();

            let trigger_with_click: Element<Message> =
                arkit::row_component::<Message, arkit::Theme>()
                    .on_click({
                        let on_open = on_open.clone();
                        move || {
                            if is_active {
                                on_open(None);
                            } else {
                                on_open(Some(index));
                            }
                        }
                    })
                    .children(vec![trigger])
                    .into();

            menu_popup(
                trigger_with_click,
                spec.items,
                is_active,
                move |open| {
                    if open {
                        on_close(Some(index));
                    } else {
                        on_close(None);
                    }
                },
                super::floating_layer::FloatingAlign::Start,
                MenuStyle {
                    width: MENU_PANEL_WIDTH,
                    submenu_width: SUBMENU_PANEL_WIDTH,
                    side_offset_vp: MENU_PANEL_SIDE_OFFSET,
                },
            )
        })
        .collect();

    shadow_sm(rounded_menubar_surface(
        arkit::row_component::<Message, arkit::Theme>().children(inline(items, spacing::XXS)),
    ))
    .into()
}

fn menubar_message<Message>(
    menus: Vec<MenubarMenuSpec>,
    active: Option<usize>,
    on_active_change: impl Fn(Option<usize>) -> Message + 'static,
) -> Element<Message>
where
    Message: Send + 'static,
{
    menubar_impl(menus, active, move |value| {
        dispatch_message(on_active_change(value))
    })
}

fn menubar<Message: 'static>(items: Vec<Element<Message>>) -> Element<Message> {
    shadow_sm(rounded_menubar_surface(
        arkit::row_component::<Message, arkit::Theme>().children(inline(items, spacing::XXS)),
    ))
    .into()
}

fn menubar_item<Message: 'static>(title: impl Into<String>) -> Element<Message> {
    menubar_trigger_surface(title, false)
}

fn menubar_item_active<Message: 'static>(title: impl Into<String>) -> Element<Message> {
    menubar_trigger_surface(title, true)
}

// Struct component API
pub struct Menubar<Message = ()> {
    menus: Vec<MenubarMenuSpec>,
    active: Option<Option<usize>>,
    default_active: Option<usize>,
    on_active_change: Option<std::rc::Rc<dyn Fn(Option<usize>) -> Message>>,
}

impl<Message> Menubar<Message> {
    pub fn new(menus: Vec<MenubarMenuSpec>) -> Self {
        Self {
            menus,
            active: None,
            default_active: None,
            on_active_change: None,
        }
    }

    pub fn active(mut self, active: Option<usize>) -> Self {
        self.active = Some(active);
        self
    }

    pub fn default_active(mut self, active: Option<usize>) -> Self {
        self.default_active = active;
        self
    }

    pub fn on_active_change(
        mut self,
        handler: impl Fn(Option<usize>) -> Message + 'static,
    ) -> Self {
        self.on_active_change = Some(std::rc::Rc::new(handler));
        self
    }
}

impl<Message: Send + 'static> arkit::advanced::Widget<Message, arkit::Theme, arkit::Renderer>
    for Menubar<Message>
{
    fn body(
        &self,
        tree: &mut arkit::advanced::widget::Tree,
        _renderer: &arkit::Renderer,
    ) -> Option<Element<Message>> {
        let state = super::widget_state(tree, || self.default_active);
        let is_controlled = self.active.is_some();
        let active = self.active.unwrap_or_else(|| *state.borrow());
        let handler = self.on_active_change.clone();
        Some(menubar_impl(self.menus.clone(), active, move |value| {
            if !is_controlled {
                *state.borrow_mut() = value;
                super::request_widget_rerender();
            }
            if let Some(handler) = handler.as_ref() {
                dispatch_message(handler(value));
            }
        }))
    }
}

impl<Message: Send + 'static> From<Menubar<Message>> for Element<Message> {
    fn from(value: Menubar<Message>) -> Self {
        Element::new(value)
    }
}
