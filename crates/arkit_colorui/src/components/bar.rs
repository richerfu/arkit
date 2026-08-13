//! ColorUI bar — title, search, tabbar, and footer.

use arkit_component::style::PaletteColor;

use crate::theme::use_colorui_theme;
use arkit_prelude::*;

use crate::theme::{swatch, GradualColor};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BarKind {
    #[default]
    Nav,
    Search,
    Tabbar,
    Foot,
}

#[derive(Props, Clone, PartialEq)]
pub struct BarProps {
    #[props(default)]
    pub kind: BarKind,
    pub title: Option<String>,
    pub back: Option<bool>,
    pub on_back: Option<EventHandler<()>>,
    #[props(default)]
    pub color: Option<PaletteColor>,
    #[props(default)]
    pub gradual: Option<GradualColor>,
    pub search_value: Option<String>,
    pub search_placeholder: Option<String>,
    pub on_search: Option<EventHandler<String>>,
    pub left: Option<Element>,
    pub right: Option<Element>,
    pub children: Element,
}

#[component]
pub fn Bar(props: BarProps) -> Element {
    let theme = use_colorui_theme().tokens();
    let background = if let Some(gradual) = props.gradual {
        gradual.fill()
    } else if let Some(color) = props.color {
        swatch(color).fill
    } else {
        theme.colors.card
    };
    let ink = if props.gradual.is_some() || props.color.is_some() {
        0xFFFFFFFF
    } else {
        theme.colors.foreground
    };
    let min_height = match props.kind {
        BarKind::Tabbar => 56.0,
        _ => 50.0,
    };
    let on_back = props.on_back;
    let back = props.back.unwrap_or(false);

    rsx! {
        row {
            width: "100%",
            min_height,
            align_items: "center",
            justify_content: "space-between",
            background_color: background,
            padding_left: 15.0,
            padding_right: 15.0,
            if back || props.left.is_some() {
                row {
                    align_items: "center",
                    if back {
                        button {
                            button_type: "normal",
                            background_color: 0x00000000,
                            border_width: 0.0,
                            padding: 0.0,
                            font_color: ink,
                            onclick: move |_| {
                                if let Some(handler) = on_back {
                                    handler.call(());
                                }
                            },
                            text { content: "‹", font_size: 22.0, font_color: ink }
                        }
                    }
                    if let Some(left) = props.left {
                        {left}
                    }
                }
            }
            if let Some(title) = props.title.clone() {
                text {
                    content: title,
                    font_size: 16.0,
                    font_weight: 500,
                    font_color: ink,
                }
            } else if matches!(props.kind, BarKind::Search) {
                row {
                    layout_weight: 1.0,
                    height: 32.0,
                    margin_left: 8.0,
                    margin_right: 8.0,
                    align_items: "center",
                    background_color: 0xFFF5F5F5u32,
                    border_radius: 999.0,
                    padding_left: 12.0,
                    padding_right: 12.0,
                    textinput {
                        value: if let Some(value) = props.search_value.clone() { value },
                        placeholder: props.search_placeholder.clone().unwrap_or_else(|| "搜索".into()),
                        font_size: 13.0,
                        font_color: theme.colors.foreground,
                        background_color: 0x00000000,
                        border_width: 0.0,
                        height: 32.0,
                        width: "100%",
                        on_change: move |evt| {
                            if let Some(handler) = props.on_search {
                                handler.call(evt.data().string_value.clone());
                            }
                        },
                    }
                }
            } else {
                row { layout_weight: 1.0, {props.children} }
            }
            row {
                align_items: "center",
                if let Some(right) = props.right {
                    {right}
                }
            }
        }
    }
}
