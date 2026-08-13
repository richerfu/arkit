//! Dropdown / context / menubar — ColorUI action sheet (`.cu-list.menu`).

use arkit_component::components::MenuEntry;
use arkit_prelude::*;

use super::chrome::{colorui_bottom_portal, dialog_fill, provide_close, CuBarHeader};
use crate::spec;
use crate::theme::use_colorui_theme;

fn render_entries(items: Vec<MenuEntry>, dismiss: EventHandler<()>) -> Vec<Element> {
    items
        .into_iter()
        .flat_map(|entry| match entry {
            MenuEntry::Separator => vec![rsx! {
                row {
                    width: "100%",
                    height: 1.0,
                    background_color: spec::FORM_LINE,
                }
            }],
            MenuEntry::Label(label) => vec![rsx! {
                row {
                    width: "100%",
                    min_height: 36.0,
                    padding_left: spec::PADDING,
                    align_items: "center",
                    background_color: spec::PAGE_BG,
                    text {
                        content: label.title,
                        font_size: spec::TEXT_SM,
                        font_color: spec::TEXT_MUTED,
                    }
                }
            }],
            MenuEntry::Action(action) => {
                let on_select = action.on_select;
                let disabled = action.disabled;
                let color = if action.destructive {
                    spec::BG_RED
                } else {
                    spec::TEXT
                };
                vec![rsx! {
                    row {
                        width: "100%",
                        min_height: spec::LIST_ITEM,
                        padding_left: spec::PADDING,
                        padding_right: spec::PADDING,
                        align_items: "center",
                        background_color: spec::BG_WHITE,
                        opacity: if disabled { 0.5 } else { 1.0 },
                        onclick: move |_| {
                            if disabled {
                                return;
                            }
                            if let Some(handler) = on_select {
                                handler.call(());
                            }
                            dismiss.call(());
                        },
                        text {
                            content: action.title,
                            font_size: spec::TEXT_DF,
                            font_color: color,
                        }
                    }
                }]
            }
            MenuEntry::Checkbox(box_entry) => {
                let on_toggle = box_entry.on_toggle;
                let checked = box_entry.checked;
                vec![rsx! {
                    row {
                        width: "100%",
                        min_height: spec::LIST_ITEM,
                        padding_left: spec::PADDING,
                        padding_right: spec::PADDING,
                        align_items: "center",
                        justify_content: "space-between",
                        background_color: spec::BG_WHITE,
                        onclick: move |_| {
                            on_toggle.call(!checked);
                            dismiss.call(());
                        },
                        text {
                            content: box_entry.title,
                            font_size: spec::TEXT_DF,
                            font_color: spec::TEXT,
                        }
                        text {
                            content: if checked { "✓" } else { "" },
                            font_size: spec::TEXT_LG,
                            font_color: spec::BG_GREEN,
                        }
                    }
                }]
            }
            MenuEntry::Radio(radio) => {
                let on_select = radio.on_select;
                let value = radio.value.clone();
                let active = radio.selected == radio.value;
                vec![rsx! {
                    row {
                        width: "100%",
                        min_height: spec::LIST_ITEM,
                        padding_left: spec::PADDING,
                        padding_right: spec::PADDING,
                        align_items: "center",
                        justify_content: "space-between",
                        background_color: spec::BG_WHITE,
                        onclick: move |_| {
                            on_select.call(value.clone());
                            dismiss.call(());
                        },
                        text {
                            content: radio.title,
                            font_size: spec::TEXT_DF,
                            font_color: spec::TEXT,
                        }
                        text {
                            content: if active { "✓" } else { "" },
                            font_size: spec::TEXT_LG,
                            font_color: spec::BG_GREEN,
                        }
                    }
                }]
            }
            MenuEntry::Submenu(sub) => {
                let mut rows = vec![rsx! {
                    row {
                        width: "100%",
                        min_height: 36.0,
                        padding_left: spec::PADDING,
                        align_items: "center",
                        background_color: spec::PAGE_BG,
                        text {
                            content: sub.title,
                            font_size: spec::TEXT_SM,
                            font_color: spec::TEXT_MUTED,
                        }
                    }
                }];
                rows.extend(render_entries(sub.items, dismiss));
                rows
            }
        })
        .collect()
}

fn action_sheet(
    title: &str,
    items: Vec<MenuEntry>,
    open: bool,
    dismiss: EventHandler<()>,
) -> Element {
    let theme = use_colorui_theme();
    let dark = matches!(theme.mode, arkit_component::style::ThemeMode::Dark);
    let rows = render_entries(items, dismiss);
    let panel = provide_close(
        dismiss,
        rsx! {
            column {
                width: "100%",
                background_color: dialog_fill(dark),
                CuBarHeader {
                    title: title.to_string(),
                    show_close: Some(true),
                }
                column {
                    width: "100%",
                    {rows.into_iter()}
                }
            }
        },
    );
    colorui_bottom_portal(open, panel, dismiss)
}

#[component]
pub fn DropdownMenu(
    items: Vec<MenuEntry>,
    children: Element,
    open: Option<bool>,
    default_open: bool,
    on_open_change: Option<EventHandler<bool>>,
    trigger_capture: Option<bool>,
    #[props(default)] width: Option<f32>,
) -> Element {
    let _ = (trigger_capture, width);
    let mut internal = use_signal(|| default_open);
    let controlled = open.is_some();
    let current = open.unwrap_or_else(|| *internal.read());
    let set_open = EventHandler::new(move |value: bool| {
        if !controlled {
            internal.set(value);
        }
        if let Some(handler) = on_open_change {
            handler.call(value);
        }
    });
    let dismiss = EventHandler::new(move |_: ()| set_open.call(false));

    rsx! {
        row {
            onclick: move |_| set_open.call(!current),
            {children}
        }
        {action_sheet("请选择", items, current, dismiss)}
    }
}

#[component]
pub fn ContextMenu(
    items: Vec<MenuEntry>,
    children: Element,
    open: Option<bool>,
    default_open: bool,
    on_open_change: Option<EventHandler<bool>>,
    #[props(default)] width: Option<f32>,
) -> Element {
    let _ = width;
    let mut internal = use_signal(|| default_open);
    let controlled = open.is_some();
    let current = open.unwrap_or_else(|| *internal.read());
    let set_open = EventHandler::new(move |value: bool| {
        if !controlled {
            internal.set(value);
        }
        if let Some(handler) = on_open_change {
            handler.call(value);
        }
    });
    let dismiss = EventHandler::new(move |_: ()| set_open.call(false));

    rsx! {
        row {
            on_long_press: move |_| set_open.call(true),
            {children}
        }
        {action_sheet("请选择", items, current, dismiss)}
    }
}

#[component]
pub fn Menubar(
    menus: Vec<arkit_component::components::MenubarMenuSpec>,
    active: Option<Option<usize>>,
    default_active: Option<usize>,
    on_active_change: Option<EventHandler<Option<usize>>>,
) -> Element {
    let mut internal = use_signal(|| default_active);
    let controlled = active.is_some();
    let current = active.unwrap_or_else(|| *internal.read());
    let set_active = EventHandler::new(move |value: Option<usize>| {
        if !controlled {
            internal.set(value);
        }
        if let Some(handler) = on_active_change {
            handler.call(value);
        }
    });

    let triggers: Vec<Element> = menus
        .iter()
        .enumerate()
        .map(|(index, menu)| {
            let active_now = current == Some(index);
            let title = menu.title.clone();
            rsx! {
                row {
                    height: spec::BAR_HEIGHT,
                    padding_left: spec::PADDING,
                    padding_right: spec::PADDING,
                    align_items: "center",
                    background_color: if active_now { spec::PAGE_BG } else { spec::BG_WHITE },
                    onclick: move |_| {
                        set_active.call(if active_now { None } else { Some(index) });
                    },
                    text {
                        content: title,
                        font_size: spec::TEXT_DF,
                        font_color: spec::TEXT,
                    }
                }
            }
        })
        .collect();

    let sheet = if let Some(index) = current {
        menus.get(index).map(|menu| {
            let dismiss = EventHandler::new(move |_: ()| set_active.call(None));
            action_sheet(&menu.title, menu.items.clone(), true, dismiss)
        })
    } else {
        None
    };

    rsx! {
        column {
            width: "100%",
            row {
                width: "100%",
                height: spec::BAR_HEIGHT,
                background_color: spec::BG_WHITE,
                {triggers.into_iter()}
            }
            if let Some(sheet) = sheet {
                {sheet}
            }
        }
    }
}
