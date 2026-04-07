use super::floating_layer::{floating_panel_aligned, FloatingAlign, FloatingSide};
use super::menu_common::{
    dismiss_menu_row, fill_slot, interactive_menu_row, item_text, leading_slot, menu_action_row,
    menu_content_with_width, menu_row, provided_menu_content, shortcut_text,
    sync_submenu_with_root, MenuContext, MenuInteractionVariant, TRANSPARENT,
};
use super::*;
use arkit::{component, create_signal};
use arkit_icon as lucide;
use std::rc::Rc;

const MENU_PANEL_WIDTH: f32 = 224.0;
const SUBMENU_PANEL_WIDTH: f32 = MENU_PANEL_WIDTH - (spacing::XXS * 2.0);

pub fn dropdown_menu(trigger: Element, items: Vec<Element>, open: Signal<bool>) -> Element {
    let dismiss = {
        let open = open.clone();
        Rc::new(move || open.set(false))
    };
    let menu_context = MenuContext {
        dismiss: dismiss.clone(),
        root_open: open.clone(),
    };
    floating_panel_aligned(
        trigger,
        provided_menu_content(MENU_PANEL_WIDTH, items, menu_context),
        open,
        FloatingSide::Bottom,
        FloatingAlign::Start,
        Some(dismiss),
        false,
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
        TRANSPARENT,
    ))
    .into()
}

#[component]
pub fn dropdown_item_destructive(title: impl Into<String>) -> Element {
    dismiss_menu_row(interactive_menu_row(
        vec![fill_slot(item_text(title, color::DESTRUCTIVE, 3_i32))],
        false,
        MenuInteractionVariant::Destructive,
        TRANSPARENT,
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
        TRANSPARENT,
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
        TRANSPARENT,
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
        TRANSPARENT,
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
        TRANSPARENT,
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
        TRANSPARENT,
    )
    .into()
}

pub fn dropdown_subcontent(items: Vec<Element>) -> Element {
    menu_content_with_width(SUBMENU_PANEL_WIDTH, items)
}

pub fn dropdown_submenu_with_state(
    title: impl Into<String>,
    items: Vec<Element>,
    open: Signal<bool>,
) -> Element {
    let title = title.into();
    let toggle = open.clone();
    let content_open = open.clone();
    sync_submenu_with_root(open.clone());

    arkit::column_component()
        .percent_width(1.0)
        .align_items_start()
        .children(vec![
            arkit::dynamic({
                let open = open.clone();
                move || {
                    let is_open = open.get();
                    let mut trigger = interactive_menu_row(
                        vec![
                            fill_slot(item_text(title.clone(), color::POPOVER_FOREGROUND, 3_i32)),
                            lucide::icon(if is_open {
                                "chevron-up"
                            } else {
                                "chevron-down"
                            })
                            .size(16.0)
                            .color(color::FOREGROUND)
                            .render(),
                        ],
                        false,
                        MenuInteractionVariant::Default,
                        if is_open { color::ACCENT } else { TRANSPARENT },
                    );

                    if is_open {
                        trigger =
                            trigger.style(ArkUINodeAttributeType::Margin, vec![0.0, 0.0, 4.0, 0.0]);
                    }

                    let toggle = toggle.clone();
                    trigger
                        .on_click(move || toggle.update(|value| *value = !*value))
                        .into()
                }
            })
            .into(),
            visibility_gate(
                arkit::column_component()
                    .percent_width(1.0)
                    .align_items_start(),
                content_open,
            )
            .children(vec![dropdown_subcontent(items)])
            .into(),
        ])
        .into()
}

#[component]
pub fn dropdown_submenu(title: impl Into<String> + 'static, items: Vec<Element>) -> Element {
    let open = create_signal(false);
    dropdown_submenu_with_state(title, items, open)
}

pub fn dropdown_checkbox_item(title: impl Into<String>, checked: Signal<bool>) -> Element {
    let title = title.into();
    let toggle = checked.clone();

    arkit::dynamic(move || {
        let is_checked = checked.get();
        let toggle = toggle.clone();
        menu_action_row(
            interactive_menu_row(
                vec![
                    leading_slot(if is_checked {
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
                    fill_slot(item_text(title.clone(), color::POPOVER_FOREGROUND, 3_i32)),
                ],
                false,
                MenuInteractionVariant::Default,
                TRANSPARENT,
            ),
            move || toggle.update(|value| *value = !*value),
        )
        .into()
    })
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
    selected: Signal<String>,
) -> Element {
    let title = title.into();
    let value = value.into();
    let set_selected = selected.clone();
    let click_value = value.clone();

    arkit::dynamic(move || {
        let is_selected = selected.get() == value;
        let set_selected = set_selected.clone();
        let click_value = click_value.clone();
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
                    fill_slot(item_text(title.clone(), color::POPOVER_FOREGROUND, 3_i32)),
                ],
                false,
                MenuInteractionVariant::Default,
                TRANSPARENT,
            ),
            move || set_selected.set(click_value.clone()),
        )
        .into()
    })
}
