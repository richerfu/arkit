//! ColorUI steps.

use arkit_component::style::PaletteColor;

use crate::theme::use_colorui_theme;
use arkit_prelude::*;

use crate::theme::swatch;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StepState {
    #[default]
    Wait,
    Process,
    Finish,
    Error,
}

#[derive(Props, Clone, PartialEq)]
pub struct StepsProps {
    pub children: Element,
}

#[component]
pub fn Steps(props: StepsProps) -> Element {
    rsx! {
        row {
            width: "100%",
            align_items: "start",
            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct StepItemProps {
    pub title: String,
    pub index: u32,
    #[props(default)]
    pub state: StepState,
    #[props(default)]
    pub color: PaletteColor,
    pub first: Option<bool>,
}

#[component]
pub fn StepItem(props: StepItemProps) -> Element {
    let theme = use_colorui_theme().tokens();
    let hue = swatch(props.color);
    let active = !matches!(props.state, StepState::Wait);
    let ink = if active { hue.fill } else { 0xFF8799A3 };
    let num_bg = if active { hue.fill } else { 0x00000000 };
    let num_fg = if active { hue.ink } else { ink };
    let label = match props.state {
        StepState::Finish => "✓".to_string(),
        StepState::Error => "✕".to_string(),
        _ => format!("{}", props.index),
    };
    rsx! {
        column {
            layout_weight: 1.0,
            align_items: "center",
            row {
                width: "100%",
                align_items: "center",
                row {
                    layout_weight: 1.0,
                    height: 1.0,
                    background_color: if props.first.unwrap_or(false) {
                        0x00000000
                    } else if active {
                        hue.fill
                    } else {
                        theme.colors.border
                    },
                }
                stack {
                    width: 20.0,
                    height: 20.0,
                    alignment: "center",
                    border_radius: 999.0,
                    border_width: 1.0,
                    border_color: ink,
                    background_color: num_bg,
                    text {
                        content: label,
                        font_size: 10.0,
                        font_color: num_fg,
                    }
                }
                row {
                    layout_weight: 1.0,
                    height: 1.0,
                    background_color: theme.colors.border,
                }
            }
            text {
                content: props.title.clone(),
                font_size: 12.0,
                font_color: ink,
                margin_top: 6.0,
                text_align: "center",
            }
        }
    }
}
