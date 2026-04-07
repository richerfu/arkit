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

const MENU_PANEL_WIDTH: f32 = 208.0;
const SUBMENU_PANEL_WIDTH: f32 = MENU_PANEL_WIDTH - (spacing::XXS * 2.0);

pub fn context_menu(trigger: Element, items: Vec<Element>, open: Signal<bool>) -> Element {
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
        true, // pass_through_dismiss — no backdrop, touches pass through
    )
}

#[component]
pub fn context_menu_item(title: impl Into<String>) -> Element {
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
pub fn context_menu_item_destructive(title: impl Into<String>) -> Element {
    dismiss_menu_row(interactive_menu_row(
        vec![fill_slot(item_text(title, color::DESTRUCTIVE, 3_i32))],
        false,
        MenuInteractionVariant::Destructive,
        TRANSPARENT,
    ))
    .into()
}

#[component]
pub fn context_menu_item_inset(title: impl Into<String>) -> Element {
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
pub fn context_menu_item_inset_destructive(title: impl Into<String>) -> Element {
    dismiss_menu_row(interactive_menu_row(
        vec![
            leading_slot(None),
            fill_slot(item_text(title, color::DESTRUCTIVE, 3_i32)),
        ],
        false,
        MenuInteractionVariant::Destructive,
        TRANSPARENT,
    ))
    .into()
}

#[component]
pub fn context_menu_item_with_shortcut(
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
pub fn context_menu_item_inset_with_shortcut(
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

pub fn context_menu_item_inset_with_shortcut_action(
    title: impl Into<String>,
    shortcut: impl Into<String>,
    on_select: impl Fn() + 'static,
) -> Element {
    menu_action_row(
        interactive_menu_row(
            vec![
                leading_slot(None),
                fill_slot(item_text(title, color::POPOVER_FOREGROUND, 3_i32)),
                shortcut_text(shortcut),
            ],
            false,
            MenuInteractionVariant::Default,
            TRANSPARENT,
        ),
        on_select,
    )
    .into()
}

pub fn disabled_context_menu_item_inset_with_shortcut(
    title: impl Into<String>,
    shortcut: impl Into<String>,
) -> Element {
    interactive_menu_row(
        vec![
            leading_slot(None),
            fill_slot(item_text(title, color::POPOVER_FOREGROUND, 3_i32)),
            shortcut_text(shortcut),
        ],
        true,
        MenuInteractionVariant::Default,
        TRANSPARENT,
    )
    .into()
}

pub fn context_menu_sub_trigger_inset(title: impl Into<String>) -> Element {
    interactive_menu_row(
        vec![
            leading_slot(None),
            fill_slot(item_text(title, color::POPOVER_FOREGROUND, 3_i32)),
            lucide::icon("chevron-right")
                .size(16.0)
                .color(color::FOREGROUND)
                .render(),
        ],
        false,
        MenuInteractionVariant::Default,
        TRANSPARENT,
    )
    .into()
}

pub fn context_menu_subcontent(items: Vec<Element>) -> Element {
    menu_content_with_width(SUBMENU_PANEL_WIDTH, items)
}

pub fn context_menu_submenu_inset_with_state(
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
                            leading_slot(None),
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
            .children(vec![context_menu_subcontent(items)])
            .into(),
        ])
        .into()
}

#[component]
pub fn context_menu_submenu_inset(
    title: impl Into<String> + 'static,
    items: Vec<Element>,
) -> Element {
    let open = create_signal(false);
    context_menu_submenu_inset_with_state(title, items, open)
}

pub fn context_menu_checkbox_item(title: impl Into<String>, checked: Signal<bool>) -> Element {
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

pub fn context_menu_label(title: impl Into<String>) -> Element {
    menu_row(
        vec![fill_slot(item_text(title, color::FOREGROUND, 4_i32))],
        false,
    )
    .into()
}

pub fn context_menu_label_inset(title: impl Into<String>) -> Element {
    menu_row(
        vec![
            leading_slot(None),
            fill_slot(item_text(title, color::FOREGROUND, 4_i32)),
        ],
        false,
    )
    .into()
}

pub fn context_menu_separator() -> Element {
    arkit::row_component()
        .height(1.0)
        .percent_width(1.0)
        .style(ArkUINodeAttributeType::Margin, vec![4.0, 0.0, 4.0, 0.0])
        .background_color(color::BORDER)
        .into()
}

pub fn context_menu_radio_item(
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
