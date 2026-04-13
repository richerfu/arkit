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

pub fn menubar_menu(title: impl Into<String>, items: Vec<MenubarEntry>) -> MenubarMenuSpec {
    MenubarMenuSpec {
        title: title.into(),
        items,
    }
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
        .border_radius([radius::SM, radius::SM, radius::SM, radius::SM])
        .background_color(if active {
            color::ACCENT
        } else {
            MENUBAR_ITEM_TRANSPARENT
        })
        .children(vec![arkit::text::<Message, arkit::Theme>(title)
            .font_size(typography::SM)
            .font_weight(FontWeight::W500)
            .font_color(color::FOREGROUND)
            .line_height(20.0)
            .into()])
        .into()
}

pub fn menubar_message<Message>(
    menus: Vec<MenubarMenuSpec>,
    active: Option<usize>,
    on_active_change: impl Fn(Option<usize>) -> Message + 'static,
) -> Element<Message>
where
    Message: Send + 'static,
{
    let on_active_change =
        Rc::new(move |value: Option<usize>| dispatch_message(on_active_change(value)));

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

pub fn menubar<Message: 'static>(items: Vec<Element<Message>>) -> Element<Message> {
    shadow_sm(rounded_menubar_surface(
        arkit::row_component::<Message, arkit::Theme>().children(inline(items, spacing::XXS)),
    ))
    .into()
}

pub fn menubar_item<Message: 'static>(title: impl Into<String>) -> Element<Message> {
    menubar_trigger_surface(title, false)
}

pub fn menubar_item_active<Message: 'static>(title: impl Into<String>) -> Element<Message> {
    menubar_trigger_surface(title, true)
}
