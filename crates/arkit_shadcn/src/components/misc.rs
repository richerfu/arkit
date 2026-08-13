//! Remaining shadcn chrome: toggle group, collapsible, breadcrumb, spinner, nav, sidebar.

use arkit_component::appearance::ButtonVariant;
use arkit_component::components::Spinner as HeadlessSpinner;
use arkit_prelude::*;

use super::primitives::Button;
use crate::spec;
use crate::theme::use_theme;

#[component]
pub fn ToggleGroup(
    options: Vec<String>,
    #[props(default)] selected: Option<Vec<String>>,
    #[props(default)] default_selected: Vec<String>,
    #[props(default)] icons: bool,
    #[props(default)] multi: bool,
    #[props(default)] width: Option<String>,
    #[props(default)] height: Option<f32>,
    #[props(default)] shadow: Option<bool>,
    #[props(default)] on_change: EventHandler<Vec<String>>,
) -> Element {
    let _ = width;
    let theme = use_theme();
    let controlled = selected.is_some();
    let local = use_signal(|| default_selected.clone());
    let current = selected.unwrap_or_else(|| local.read().clone());
    let item_h = height.unwrap_or(spec::BTN_HEIGHT_SM);
    let items: Vec<Element> = options
        .iter()
        .enumerate()
        .map(|(index, option)| {
            let active = current.contains(option);
            let value = option.clone();
            let now = current.clone();
            let mut local = local;
            rsx! {
                button {
                    button_type: "normal",
                    height: item_h,
                    padding_left: 10.0,
                    padding_right: 10.0,
                    background_color: if active {
                        theme.colors.secondary
                    } else {
                        theme.colors.background
                    },
                    border_width: 1.0,
                    border_color: theme.colors.input,
                    border_radius: if index == 0 {
                        format!("{},0,0,{}", spec::RADIUS_MD, spec::RADIUS_MD)
                    } else if index + 1 == options.len() {
                        format!("0,{},{},0", spec::RADIUS_MD, spec::RADIUS_MD)
                    } else {
                        "0,0,0,0".into()
                    },
                    shadow: if shadow.unwrap_or(true) && index == 0 { "sm" },
                    focusable: false,
                    onclick: move |_| {
                        let mut next = now.clone();
                        if multi {
                            if let Some(i) = next.iter().position(|item| item == &value) {
                                next.remove(i);
                            } else {
                                next.push(value.clone());
                            }
                        } else {
                            next = vec![value.clone()];
                        }
                        if !controlled {
                            local.set(next.clone());
                        }
                        on_change.call(next);
                    },
                    if icons {
                        {arkit_icon::icon(option.clone(), 16.0, theme.colors.foreground)}
                    } else {
                        text {
                            content: option.clone(),
                            font_size: spec::TEXT_SM,
                            font_color: theme.colors.foreground,
                        }
                    }
                }
            }
        })
        .collect();
    rsx! { row { align_items: "center", {items.into_iter()} } }
}

#[component]
pub fn Collapsible(
    title: String,
    children: Element,
    #[props(default)] open: Option<bool>,
    #[props(default)] default_open: bool,
    #[props(default)] on_open_change: EventHandler<bool>,
) -> Element {
    let theme = use_theme();
    let mut local = use_signal(|| default_open);
    let current = open.unwrap_or_else(|| *local.read());
    let controlled = open.is_some();
    rsx! {
        column {
            width: "100%",
            row {
                width: "100%",
                padding_top: 16.0,
                padding_bottom: 16.0,
                align_items: "center",
                justify_content: "space-between",
                onclick: move |_| {
                    let next = !current;
                    if !controlled {
                        local.set(next);
                    }
                    on_open_change.call(next);
                },
                text {
                    content: title,
                    font_size: spec::TEXT_SM,
                    font_weight: spec::FONT_MEDIUM,
                    font_color: theme.colors.foreground,
                }
                {arkit_icon::icon("chevrons-up-down", 16.0, theme.colors.muted_foreground)}
            }
            if current {
                column { width: "100%", {children} }
            }
        }
    }
}

#[component]
pub fn Breadcrumb(items: Vec<String>) -> Element {
    let theme = use_theme();
    let total = items.len();
    let parts: Vec<Element> = items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let last = index + 1 == total;
            rsx! {
                text {
                    content: item.clone(),
                    font_size: if last { spec::TEXT_BASE } else { spec::TEXT_SM },
                    font_color: if last {
                        theme.colors.foreground
                    } else {
                        theme.colors.muted_foreground
                    },
                }
                if !last {
                    text {
                        content: " / ",
                        font_size: spec::TEXT_SM,
                        font_color: theme.colors.muted_foreground,
                    }
                }
            }
        })
        .collect();
    rsx! { row { align_items: "center", {parts.into_iter()} } }
}

#[component]
pub fn Spinner(
    #[props(default = 16.0)] size: f32,
    #[props(default)] color: Option<u32>,
    #[props(default)] icon: Option<String>,
    #[props(default = 2.0)] stroke_width: f32,
    #[props(default = true)] spinning: bool,
) -> Element {
    let theme = use_theme();
    let color = color.unwrap_or(theme.colors.foreground);
    rsx! {
        HeadlessSpinner {
            size,
            color: Some(color),
            icon,
            stroke_width,
            spinning,
        }
    }
}

#[component]
pub fn BottomNavigation(
    items: Vec<arkit_component::components::BottomNavigationItem>,
    #[props(default)] selected: Option<usize>,
    #[props(default)] default_selected: usize,
    #[props(default)] on_select: EventHandler<usize>,
) -> Element {
    let theme = use_theme();
    let count = items.len();
    let mut local = use_signal(move || default_selected.min(count.saturating_sub(1)));
    let controlled = selected.is_some();
    let current = selected
        .unwrap_or_else(|| *local.read())
        .min(count.saturating_sub(1));
    let dests: Vec<Element> = items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let active = index == current;
            let color = if active {
                theme.colors.primary
            } else {
                theme.colors.muted_foreground
            };
            let label = item.label.clone();
            let icon = item.icon.clone();
            rsx! {
                column {
                    layout_weight: 1.0,
                    align_items: "center",
                    justify_content: "center",
                    onclick: move |_| {
                        if !controlled {
                            local.set(index);
                        }
                        on_select.call(index);
                    },
                    {arkit_icon::icon(icon, 22.0, color)}
                    text {
                        content: label,
                        font_size: 11.0,
                        font_color: color,
                        margin_top: 3.0,
                    }
                }
            }
        })
        .collect();
    rsx! {
        row {
            width: "100%",
            height: 64.0,
            background_color: theme.colors.background,
            border_width: 0.0,
            {dests.into_iter()}
        }
    }
}

#[component]
pub fn Sidebar(sidebar: Element, children: Element) -> Element {
    let theme = use_theme();
    rsx! {
        row {
            width: "100%",
            column {
                width: 180.0,
                background_color: theme.colors.popover,
                border_width: 1.0,
                border_color: theme.colors.border,
                border_radius: spec::RADIUS_MD,
                shadow: "sm",
                {sidebar}
            }
            {children}
        }
    }
}

#[component]
pub fn SidebarItem(
    title: String,
    active: Option<bool>,
    onclick: Option<EventHandler<()>>,
) -> Element {
    rsx! {
        Button {
            variant: if active.unwrap_or(false) {
                ButtonVariant::Secondary
            } else {
                ButtonVariant::Ghost
            },
            width: Some("100%".into()),
            onclick,
            "{title}"
        }
    }
}
