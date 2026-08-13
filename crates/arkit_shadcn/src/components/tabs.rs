//! Tabs — official `tabs.tsx` muted track + active pill.

use arkit_prelude::*;

use crate::spec;
use crate::theme::use_theme;

pub use arkit_component::components::{TabsContent, TabsProps};

#[component]
pub fn TabsList(children: Element) -> Element {
    let theme = use_theme();
    rsx! {
        row {
            width: "100%",
            height: spec::BTN_HEIGHT_SM,
            align_items: "center",
            background_color: theme.colors.muted,
            border_radius: spec::RADIUS_MD,
            padding_left: 4.0,
            padding_right: 4.0,
            {children}
        }
    }
}

#[component]
pub fn TabsTrigger(
    label: String,
    active: bool,
    #[props(default)] on_press: EventHandler<()>,
) -> Element {
    let theme = use_theme();
    rsx! {
        row {
            layout_weight: 1.0,
            height: 28.0,
            align_items: "center",
            justify_content: "center",
            background_color: if active {
                theme.colors.background
            } else {
                0x00000000u32
            },
            border_radius: spec::RADIUS_MD,
            shadow: if active { "sm" },
            onclick: move |_| on_press.call(()),
            text {
                content: label,
                font_size: spec::TEXT_SM,
                font_weight: spec::FONT_MEDIUM,
                font_color: theme.colors.foreground,
            }
        }
    }
}

#[component]
pub fn Tabs(props: TabsProps) -> Element {
    let controlled = props.active.is_some();
    let local = use_signal(|| props.default_active);
    let active = props.active.unwrap_or_else(|| *local.read());
    let on_change = props.on_change;
    let triggers: Vec<Element> = props
        .labels
        .iter()
        .enumerate()
        .map(|(index, label)| {
            let mut local = local;
            rsx! {
                TabsTrigger {
                    key: "{index}",
                    label: label.clone(),
                    active: active == index,
                    on_press: move |_| {
                        if !controlled {
                            local.set(index);
                        }
                        on_change.call(index);
                    },
                }
            }
        })
        .collect();
    let panels: Vec<Element> = props
        .panels
        .iter()
        .enumerate()
        .map(|(index, panel)| {
            rsx! {
                TabsContent {
                    key: "{index}",
                    active: active == index,
                    {panel.clone()}
                }
            }
        })
        .collect();
    rsx! {
        column {
            width: "100%",
            TabsList { {triggers.into_iter()} }
            column {
                width: "100%",
                margin_top: 8.0,
                {panels.into_iter()}
            }
        }
    }
}
