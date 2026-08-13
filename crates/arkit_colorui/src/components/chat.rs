//! ColorUI chat bubbles.

use arkit_component::style::PaletteColor;

use crate::theme::use_colorui_theme;
use arkit_prelude::*;

use crate::theme::swatch;

#[derive(Props, Clone, PartialEq)]
pub struct ChatProps {
    pub children: Element,
}

#[component]
pub fn Chat(props: ChatProps) -> Element {
    rsx! {
        column {
            width: "100%",
            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct ChatItemProps {
    pub content: String,
    pub avatar: Option<String>,
    #[props(default)]
    pub self_side: bool,
    #[props(default)]
    pub color: Option<PaletteColor>,
    pub date: Option<String>,
}

#[component]
pub fn ChatItem(props: ChatItemProps) -> Element {
    let theme = use_colorui_theme().tokens();
    let bubble = if let Some(color) = props.color {
        swatch(color).fill
    } else if props.self_side {
        theme.colors.primary
    } else {
        theme.colors.card
    };
    let ink = if props.self_side || props.color.is_some() {
        0xFFFFFFFF
    } else {
        theme.colors.foreground
    };
    let avatar = props.avatar.clone().unwrap_or_default();
    rsx! {
        column {
            width: "100%",
            padding_left: 15.0,
            padding_right: 15.0,
            padding_top: 15.0,
            padding_bottom: 28.0,
            row {
                width: "100%",
                justify_content: if props.self_side { "end" } else { "start" },
                align_items: "start",
                if !props.self_side && !avatar.is_empty() {
                    image {
                        src: avatar.clone(),
                        width: 40.0,
                        height: 40.0,
                        border_radius: 999.0,
                        object_fit: "cover",
                    }
                }
                column {
                    max_width: "70%",
                    margin_left: 12.0,
                    margin_right: 12.0,
                    padding: 10.0,
                    background_color: bubble,
                    border_radius: 6.0,
                    text {
                        content: props.content.clone(),
                        font_size: 15.0,
                        font_color: ink,
                        line_height: 22.0,
                    }
                }
                if props.self_side && !avatar.is_empty() {
                    image {
                        src: avatar,
                        width: 40.0,
                        height: 40.0,
                        border_radius: 999.0,
                        object_fit: "cover",
                    }
                }
            }
            if let Some(date) = props.date.clone() {
                text {
                    content: date,
                    font_size: 12.0,
                    font_color: 0xFF8799A3u32,
                    margin_top: 4.0,
                    margin_left: if props.self_side { 0.0 } else { 64.0 },
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct ChatInfoProps {
    pub content: String,
}

#[component]
pub fn ChatInfo(props: ChatInfoProps) -> Element {
    rsx! {
        row {
            width: "100%",
            justify_content: "center",
            margin_top: 10.0,
            margin_bottom: 10.0,
            text {
                content: props.content.clone(),
                font_size: 12.0,
                font_color: 0xFFFFFFFFu32,
                background_color: 0x33000000u32,
                padding_left: 8.0,
                padding_right: 8.0,
                padding_top: 4.0,
                padding_bottom: 4.0,
                border_radius: 6.0,
            }
        }
    }
}
