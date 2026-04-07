use super::floating_layer::{floating_panel_aligned, FloatingAlign, FloatingSide};
use super::menu_common::{
    current_menu_surface, dismiss_menu_row, fill_slot, interactive_menu_row, item_text,
    leading_slot, menu_action_row, menu_content_with_width, menu_dismiss_context, menu_row,
    provided_menu_content, root_menu_context, root_menu_surfaces, shortcut_text,
    submenu_menu_context, submenu_menu_surfaces, MenuInteractionVariant,
};
use super::*;
use arkit::component;
use arkit_icon as lucide;
use std::rc::Rc;

#[derive(Clone)]
struct DropdownSubmenuMarker;

const MENU_PANEL_WIDTH: f32 = 224.0;
const SUBMENU_PANEL_WIDTH: f32 = MENU_PANEL_WIDTH - (spacing::XXS * 2.0);

pub fn dropdown_menu(
    trigger: Element,
    items: Vec<Element>,
    open: bool,
    on_open_change: impl Fn(bool) + 'static,
) -> Element {
    let dismiss = {
        Rc::new(move || {
            on_open_change(false);
        })
    };
    let menu_context = root_menu_context(dismiss.clone(), open);
    floating_panel_aligned(
        trigger,
        provided_menu_content(MENU_PANEL_WIDTH, items, menu_context.clone()),
        open,
        FloatingSide::Bottom,
        FloatingAlign::Start,
        Some(dismiss),
        false,
        root_menu_surfaces(&menu_context),
        None,
    )
}

#[component]
pub fn dropdown_item(title: impl Into<String>) -> Element {
    dismiss_menu_row(interactive_menu_row(
        vec![fill_slot(item_text(
            title,
            color::POPOVER_FOREGROUND,
            3_i32,
        ))],
        false,
        MenuInteractionVariant::Default,
        None,
    ))
    .into()
}

#[component]
pub fn dropdown_item_destructive(title: impl Into<String>) -> Element {
    dismiss_menu_row(interactive_menu_row(
        vec![fill_slot(item_text(title, color::DESTRUCTIVE, 3_i32))],
        false,
        MenuInteractionVariant::Destructive,
        None,
    ))
    .into()
}

#[component]
pub fn dropdown_item_inset(title: impl Into<String>) -> Element {
    dismiss_menu_row(interactive_menu_row(
        vec![
            leading_slot(None),
            fill_slot(item_text(title, color::POPOVER_FOREGROUND, 3_i32)),
        ],
        false,
        MenuInteractionVariant::Default,
        None,
    ))
    .into()
}

#[component]
pub fn dropdown_item_with_shortcut(
    title: impl Into<String>,
    shortcut: impl Into<String>,
) -> Element {
    dismiss_menu_row(interactive_menu_row(
        vec![
            fill_slot(item_text(title, color::POPOVER_FOREGROUND, 3_i32)),
            shortcut_text(shortcut),
        ],
        false,
        MenuInteractionVariant::Default,
        None,
    ))
    .into()
}

#[component]
pub fn dropdown_item_inset_with_shortcut(
    title: impl Into<String>,
    shortcut: impl Into<String>,
) -> Element {
    dismiss_menu_row(interactive_menu_row(
        vec![
            leading_slot(None),
            fill_slot(item_text(title, color::POPOVER_FOREGROUND, 3_i32)),
            shortcut_text(shortcut),
        ],
        false,
        MenuInteractionVariant::Default,
        None,
    ))
    .into()
}

pub fn disabled_dropdown_item(title: impl Into<String>) -> Element {
    interactive_menu_row(
        vec![fill_slot(item_text(
            title,
            color::POPOVER_FOREGROUND,
            3_i32,
        ))],
        true,
        MenuInteractionVariant::Default,
        None,
    )
    .into()
}

pub fn disabled_dropdown_item_with_shortcut(
    title: impl Into<String>,
    shortcut: impl Into<String>,
) -> Element {
    interactive_menu_row(
        vec![
            fill_slot(item_text(title, color::POPOVER_FOREGROUND, 3_i32)),
            shortcut_text(shortcut),
        ],
        true,
        MenuInteractionVariant::Default,
        None,
    )
    .into()
}

pub fn dropdown_subcontent(items: Vec<Element>) -> Element {
    menu_content_with_width(SUBMENU_PANEL_WIDTH, items)
}

pub fn dropdown_submenu_with_state(
    title: impl Into<String>,
    items: Vec<Element>,
    open: Rc<std::cell::Cell<bool>>,
) -> Element {
    let title = title.into();
    let toggle = open.clone();
    let Some(parent_menu) = menu_dismiss_context() else {
        return interactive_menu_row(
            vec![
                fill_slot(item_text(title, color::POPOVER_FOREGROUND, 3_i32)),
                lucide::icon("chevron-right")
                    .size(16.0)
                    .color(color::FOREGROUND)
                    .render(),
            ],
            false,
            MenuInteractionVariant::Default,
            None,
        )
        .into();
    };
    if !parent_menu.root_open && open.get() {
        open.set(false);
    }
    let submenu_surfaces = super::floating_layer::FloatingSurfaceRegistry::new();
    let submenu_context = submenu_menu_context(&parent_menu, submenu_surfaces.clone());
    let dismiss_submenu = {
        let open = open.clone();
        Rc::new(move || {
            open.set(false);
            request_runtime_rerender();
        })
    };

    floating_panel_aligned(
        interactive_menu_row(
            vec![
                fill_slot(item_text(title, color::POPOVER_FOREGROUND, 3_i32)),
                lucide::icon("chevron-right")
                    .size(16.0)
                    .color(color::FOREGROUND)
                    .render(),
            ],
            false,
            MenuInteractionVariant::Default,
            Some(open.get()),
        )
        .on_click(move || {
            toggle.set(!toggle.get());
            request_runtime_rerender();
        })
        .into(),
        provided_menu_content(SUBMENU_PANEL_WIDTH, items, submenu_context.clone()),
        open.get(),
        FloatingSide::Right,
        FloatingAlign::Start,
        Some(dismiss_submenu),
        true,
        submenu_menu_surfaces(&parent_menu, &submenu_surfaces),
        Some(current_menu_surface(&submenu_context)),
    )
}

#[component]
pub fn dropdown_submenu(title: impl Into<String> + 'static, items: Vec<Element>) -> Element {
    let open = local_bool_state(DropdownSubmenuMarker, false);
    dropdown_submenu_with_state(title, items, open)
}

pub fn dropdown_checkbox_item(
    title: impl Into<String>,
    checked: bool,
    on_toggle: impl Fn(bool) + 'static,
) -> Element {
    let title = title.into();
    menu_action_row(
        interactive_menu_row(
            vec![
                leading_slot(if checked {
                    Some(
                        lucide::icon("check")
                            .size(16.0)
                            .stroke_width(3.0)
                            .color(color::FOREGROUND)
                            .render(),
                    )
                } else {
                    None
                }),
                fill_slot(item_text(title, color::POPOVER_FOREGROUND, 3_i32)),
            ],
            false,
            MenuInteractionVariant::Default,
            None,
        ),
        move || on_toggle(!checked),
    )
    .into()
}

pub fn dropdown_label(title: impl Into<String>) -> Element {
    menu_row(
        vec![fill_slot(item_text(title, color::FOREGROUND, 4_i32))],
        false,
    )
    .into()
}

pub fn dropdown_label_inset(title: impl Into<String>) -> Element {
    menu_row(
        vec![
            leading_slot(None),
            fill_slot(item_text(title, color::FOREGROUND, 4_i32)),
        ],
        false,
    )
    .into()
}

pub fn dropdown_separator() -> Element {
    arkit::row_component()
        .height(1.0)
        .percent_width(1.0)
        .style(ArkUINodeAttributeType::Margin, vec![4.0, 0.0, 4.0, 0.0])
        .background_color(color::BORDER)
        .into()
}

pub fn dropdown_radio_item(
    title: impl Into<String>,
    value: impl Into<String>,
    selected: impl Into<String>,
    on_select: impl Fn(String) + 'static,
) -> Element {
    let title = title.into();
    let value = value.into();
    let click_value = value.clone();
    let is_selected = selected.into() == value;

    menu_action_row(
        interactive_menu_row(
            vec![
                leading_slot(if is_selected {
                    Some(
                        arkit::row_component()
                            .width(8.0)
                            .height(8.0)
                            .style(
                                ArkUINodeAttributeType::BorderRadius,
                                vec![radius::FULL, radius::FULL, radius::FULL, radius::FULL],
                            )
                            .background_color(color::FOREGROUND)
                            .into(),
                    )
                } else {
                    None
                }),
                fill_slot(item_text(title, color::POPOVER_FOREGROUND, 3_i32)),
            ],
            false,
            MenuInteractionVariant::Default,
            None,
        ),
        move || on_select(click_value.clone()),
    )
    .into()
}
