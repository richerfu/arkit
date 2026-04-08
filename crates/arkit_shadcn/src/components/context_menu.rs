use super::floating_layer::{floating_panel_aligned, FloatingAlign, FloatingSide};
use super::menu_common::{
    current_menu_surface, dismiss_menu_row, fill_slot, interactive_menu_row, item_text,
    leading_slot, menu_action_row, menu_content_with_width, menu_dismiss_context, menu_row,
    menu_surface_registry, provided_menu_content, root_menu_context, root_menu_surfaces,
    shortcut_text, submenu_menu_context, submenu_menu_surfaces, MenuInteractionVariant,
};
use super::*;
use arkit_icon as lucide;
use std::rc::Rc;

const MENU_PANEL_WIDTH: f32 = 208.0;
const SUBMENU_PANEL_WIDTH: f32 = MENU_PANEL_WIDTH - (spacing::XXS * 2.0);

pub fn context_menu<Message: 'static>(
    trigger: Element<Message>,
    items: Vec<Element<Message>>,
    open: bool,
    on_open_change: impl Fn(bool) + 'static,
) -> Element<Message> {
    arkit_widget::scope(move || {
        let root_surfaces = menu_surface_registry();
        let dismiss = {
            Rc::new(move || {
                on_open_change(false);
            })
        };
        let menu_context = root_menu_context(dismiss.clone(), open, root_surfaces);
        floating_panel_aligned(
            trigger,
            provided_menu_content(MENU_PANEL_WIDTH, items, menu_context.clone()),
            open,
            FloatingSide::Bottom,
            FloatingAlign::Start,
            Some(dismiss),
            true, // pass_through_dismiss — no backdrop, touches pass through
            root_menu_surfaces(&menu_context),
            Some(current_menu_surface(&menu_context)),
        )
    })
}

pub fn context_menu_message<Message>(
    trigger: Element<Message>,
    items: Vec<Element<Message>>,
    open: bool,
    on_open_change: impl Fn(bool) -> Message + 'static,
) -> Element<Message>
where
    Message: Send + 'static,
{
    context_menu(trigger, items, open, move |value| {
        dispatch_message(on_open_change(value))
    })
}

pub fn context_menu_item<Message: 'static>(title: impl Into<String>) -> Element<Message> {
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

pub fn context_menu_item_destructive<Message: 'static>(
    title: impl Into<String>,
) -> Element<Message> {
    dismiss_menu_row(interactive_menu_row(
        vec![fill_slot(item_text(title, color::DESTRUCTIVE, 3_i32))],
        false,
        MenuInteractionVariant::Destructive,
        None,
    ))
    .into()
}

pub fn context_menu_item_inset<Message: 'static>(title: impl Into<String>) -> Element<Message> {
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

pub fn context_menu_item_inset_destructive<Message: 'static>(
    title: impl Into<String>,
) -> Element<Message> {
    dismiss_menu_row(interactive_menu_row(
        vec![
            leading_slot(None),
            fill_slot(item_text(title, color::DESTRUCTIVE, 3_i32)),
        ],
        false,
        MenuInteractionVariant::Destructive,
        None,
    ))
    .into()
}

pub fn context_menu_item_with_shortcut<Message: 'static>(
    title: impl Into<String>,
    shortcut: impl Into<String>,
) -> Element<Message> {
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

pub fn context_menu_item_inset_with_shortcut<Message: 'static>(
    title: impl Into<String>,
    shortcut: impl Into<String>,
) -> Element<Message> {
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

pub fn context_menu_item_inset_with_shortcut_action<Message: 'static>(
    title: impl Into<String>,
    shortcut: impl Into<String>,
    on_select: impl Fn() + 'static,
) -> Element<Message> {
    menu_action_row(
        interactive_menu_row(
            vec![
                leading_slot(None),
                fill_slot(item_text(title, color::POPOVER_FOREGROUND, 3_i32)),
                shortcut_text(shortcut),
            ],
            false,
            MenuInteractionVariant::Default,
            None,
        ),
        on_select,
    )
    .into()
}

pub fn context_menu_item_inset_with_shortcut_action_message<Message>(
    title: impl Into<String>,
    shortcut: impl Into<String>,
    on_select: Message,
) -> Element<Message>
where
    Message: Clone + Send + 'static,
{
    context_menu_item_inset_with_shortcut_action(title, shortcut, move || {
        dispatch_message(on_select.clone())
    })
}

pub fn disabled_context_menu_item_inset_with_shortcut<Message: 'static>(
    title: impl Into<String>,
    shortcut: impl Into<String>,
) -> Element<Message> {
    interactive_menu_row(
        vec![
            leading_slot(None),
            fill_slot(item_text(title, color::POPOVER_FOREGROUND, 3_i32)),
            shortcut_text(shortcut),
        ],
        true,
        MenuInteractionVariant::Default,
        None,
    )
    .into()
}

pub fn context_menu_sub_trigger_inset<Message: 'static>(
    title: impl Into<String>,
) -> Element<Message> {
    interactive_menu_row(
        vec![
            leading_slot(None),
            fill_slot(item_text(title, color::POPOVER_FOREGROUND, 3_i32)),
            lucide::icon("chevron-right")
                .size(16.0)
                .color(color::FOREGROUND)
                .render::<Message, arkit::Theme>(),
        ],
        false,
        MenuInteractionVariant::Default,
        None,
    )
    .into()
}

pub fn context_menu_subcontent<Message: 'static>(items: Vec<Element<Message>>) -> Element<Message> {
    menu_content_with_width(SUBMENU_PANEL_WIDTH, items)
}

pub fn context_menu_submenu_inset<Message: 'static>(
    title: impl Into<String>,
    items: Vec<Element<Message>>,
    open: bool,
    on_open_change: impl Fn(bool) + 'static,
) -> Element<Message> {
    let title = title.into();
    arkit_widget::scope(move || {
        let Some(parent_menu) = menu_dismiss_context() else {
            return context_menu_sub_trigger_inset(title);
        };
        let submenu_open = open && parent_menu.root_open;
        let submenu_surfaces = menu_surface_registry();
        let submenu_context = submenu_menu_context(&parent_menu, submenu_surfaces.clone());
        let on_open_change = Rc::new(on_open_change);
        let dismiss_submenu = {
            let on_open_change = on_open_change.clone();
            Rc::new(move || on_open_change(false))
        };
        let toggle_submenu = on_open_change.clone();

        floating_panel_aligned(
            interactive_menu_row(
                vec![
                    leading_slot(None),
                    fill_slot(item_text(title, color::POPOVER_FOREGROUND, 3_i32)),
                    lucide::icon("chevron-right")
                        .size(16.0)
                        .color(color::FOREGROUND)
                        .render::<Message, arkit::Theme>(),
                ],
                false,
                MenuInteractionVariant::Default,
                Some(submenu_open),
            )
            .on_click({
                let toggle_submenu = toggle_submenu.clone();
                move || toggle_submenu(!submenu_open)
            })
            .into(),
            provided_menu_content(SUBMENU_PANEL_WIDTH, items, submenu_context.clone()),
            submenu_open,
            FloatingSide::Right,
            FloatingAlign::Start,
            Some(dismiss_submenu),
            true,
            submenu_menu_surfaces(&parent_menu, &submenu_surfaces),
            Some(current_menu_surface(&submenu_context)),
        )
    })
}

pub fn context_menu_submenu_inset_message<Message>(
    title: impl Into<String>,
    items: Vec<Element<Message>>,
    open: bool,
    on_open_change: impl Fn(bool) -> Message + 'static,
) -> Element<Message>
where
    Message: Send + 'static,
{
    context_menu_submenu_inset(title, items, open, move |value| {
        dispatch_message(on_open_change(value))
    })
}

pub fn context_menu_checkbox_item<Message: 'static>(
    title: impl Into<String>,
    checked: bool,
    on_toggle: impl Fn(bool) + 'static,
) -> Element<Message> {
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
                            .render::<Message, arkit::Theme>(),
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

pub fn context_menu_checkbox_item_message<Message>(
    title: impl Into<String>,
    checked: bool,
    on_toggle: impl Fn(bool) -> Message + 'static,
) -> Element<Message>
where
    Message: Send + 'static,
{
    context_menu_checkbox_item(title, checked, move |value| {
        dispatch_message(on_toggle(value))
    })
}

pub fn context_menu_label<Message: 'static>(title: impl Into<String>) -> Element<Message> {
    menu_row(
        vec![fill_slot(item_text(title, color::FOREGROUND, 4_i32))],
        false,
    )
    .into()
}

pub fn context_menu_label_inset<Message: 'static>(title: impl Into<String>) -> Element<Message> {
    menu_row(
        vec![
            leading_slot(None),
            fill_slot(item_text(title, color::FOREGROUND, 4_i32)),
        ],
        false,
    )
    .into()
}

pub fn context_menu_separator<Message: 'static>() -> Element<Message> {
    arkit::row_component::<Message, arkit::Theme>()
        .height(1.0)
        .percent_width(1.0)
        .style(ArkUINodeAttributeType::Margin, vec![4.0, 0.0, 4.0, 0.0])
        .background_color(color::BORDER)
        .into()
}

pub fn context_menu_radio_item<Message: 'static>(
    title: impl Into<String>,
    value: impl Into<String>,
    selected: impl Into<String>,
    on_select: impl Fn(String) + 'static,
) -> Element<Message> {
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

pub fn context_menu_radio_item_message<Message>(
    title: impl Into<String>,
    value: impl Into<String>,
    selected: impl Into<String>,
    on_select: impl Fn(String) -> Message + 'static,
) -> Element<Message>
where
    Message: Send + 'static,
{
    context_menu_radio_item(title, value, selected, move |value| {
        dispatch_message(on_select(value))
    })
}
