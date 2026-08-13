//! ColorUI tabs — underline nav on the headless tab state.

use arkit_prelude::*;

use crate::kit::resolve_color;
use crate::theme::{swatch, use_colorui_theme};
use crate::PaletteColor;

pub use arkit_component::components::{TabsContent, TabsProps};

#[derive(Props, Clone, PartialEq)]
pub struct TabsListProps {
    pub children: Element,
}

#[component]
pub fn TabsList(props: TabsListProps) -> Element {
    let tokens = use_colorui_theme().tokens();
    rsx! {
        row {
            width: "100%",
            align_items: "center",
            justify_content: "start",
            height: 40.0,
            border_width: 0.0,
            border_color: tokens.colors.border,
            background_color: tokens.colors.card,
            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct TabsTriggerProps {
    pub label: String,
    pub active: bool,
    #[props(default)]
    pub on_press: EventHandler<()>,
    #[props(default)]
    pub color: Option<PaletteColor>,
}

#[component]
pub fn TabsTrigger(props: TabsTriggerProps) -> Element {
    let theme = use_colorui_theme();
    let fill = swatch(resolve_color(props.color, &theme)).fill;
    let tokens = theme.tokens();
    let on_press = props.on_press;
    rsx! {
        column {
            height: 40.0,
            padding_left: 15.0,
            padding_right: 15.0,
            align_items: "center",
            justify_content: "center",
            onclick: move |_| on_press.call(()),
            text {
                content: props.label.clone(),
                font_size: 14.0,
                font_weight: if props.active { 600 } else { 400 },
                font_color: if props.active { fill } else { tokens.colors.foreground },
            }
            row {
                width: 20.0,
                height: 2.0,
                margin_top: 4.0,
                background_color: if props.active { fill } else { 0x00000000u32 },
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
            TabsList {
                {triggers.into_iter()}
            }
            column {
                width: "100%",
                {panels.into_iter()}
            }
        }
    }
}
