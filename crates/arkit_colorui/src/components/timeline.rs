//! ColorUI timeline.

use arkit_component::style::PaletteColor;

use crate::theme::use_colorui_theme;
use arkit_prelude::*;

use crate::theme::swatch;

#[derive(Props, Clone, PartialEq)]
pub struct TimelineProps {
    pub children: Element,
}

#[component]
pub fn Timeline(props: TimelineProps) -> Element {
    let theme = use_colorui_theme().tokens();
    rsx! {
        column {
            width: "100%",
            background_color: theme.colors.card,
            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct TimelineItemProps {
    pub time: Option<String>,
    pub content: String,
    #[props(default)]
    pub color: PaletteColor,
    pub icon: Option<String>,
}

#[component]
pub fn TimelineItem(props: TimelineItemProps) -> Element {
    let theme = use_colorui_theme().tokens();
    let hue = swatch(props.color);
    rsx! {
        row {
            width: "100%",
            align_items: "start",
            if let Some(time) = props.time.clone() {
                text {
                    content: time,
                    width: 60.0,
                    font_size: 13.0,
                    font_color: theme.colors.muted_foreground,
                    text_align: "center",
                    padding_top: 10.0,
                }
            }
            column {
                width: 25.0,
                align_items: "center",
                row {
                    width: 1.0,
                    height: 10.0,
                    background_color: theme.colors.border,
                }
                stack {
                    width: 25.0,
                    height: 25.0,
                    alignment: "center",
                    background_color: theme.colors.card,
                    if let Some(icon) = props.icon.as_ref() {
                        {arkit_icon::icon(icon, 16.0, hue.fill)}
                    } else {
                        row {
                            width: 10.0,
                            height: 10.0,
                            border_radius: 999.0,
                            background_color: hue.fill,
                        }
                    }
                }
                row {
                    width: 1.0,
                    layout_weight: 1.0,
                    min_height: 24.0,
                    background_color: theme.colors.border,
                }
            }
            column {
                layout_weight: 1.0,
                margin_left: 8.0,
                margin_top: 8.0,
                margin_bottom: 12.0,
                padding: 12.0,
                background_color: 0xFFF1F1F1u32,
                border_radius: 6.0,
                text {
                    content: props.content.clone(),
                    font_size: 14.0,
                    font_color: theme.colors.foreground,
                    line_height: 22.0,
                }
            }
        }
    }
}
