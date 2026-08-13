//! ColorUI tag and capsule.

use arkit_component::style::PaletteColor;

use crate::theme::use_colorui_theme;
use arkit_prelude::*;

use crate::theme::swatch;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TagSize {
    #[default]
    Default,
    Sm,
}

#[derive(Props, Clone, PartialEq)]
pub struct TagProps {
    pub content: String,
    #[props(default)]
    pub color: PaletteColor,
    #[props(default)]
    pub line: bool,
    #[props(default)]
    pub round: bool,
    #[props(default)]
    pub light: bool,
    #[props(default)]
    pub size: TagSize,
    #[props(default)]
    pub badge: bool,
}

#[component]
pub fn Tag(props: TagProps) -> Element {
    let theme = use_colorui_theme().tokens();
    let swatch = swatch(props.color);
    let (background, foreground, border_width, border_color) = if props.line {
        (0x00000000, swatch.fill, 1.0, swatch.fill)
    } else if props.light {
        (swatch.light_fill, swatch.light_ink, 0.0, 0x00000000)
    } else if matches!(props.color, PaletteColor::Default) && !props.badge {
        (0xFFF1F1F1, theme.colors.foreground, 0.0, 0x00000000)
    } else {
        (swatch.fill, swatch.ink, 0.0, 0x00000000)
    };
    let (height, pad, font) = if props.badge {
        (14.0, 5.0, 10.0)
    } else {
        match props.size {
            TagSize::Sm => (16.0, 6.0, 10.0),
            TagSize::Default => (24.0, 8.0, 12.0),
        }
    };
    let radius = if props.round || props.badge {
        999.0
    } else {
        3.0
    };
    rsx! {
        row {
            constraint_size: format!("0,100000,{height},100000"),
            align_items: "center",
            justify_content: "center",
            height,
            padding_left: pad,
            padding_right: pad,
            background_color: background,
            border_width,
            border_color,
            border_radius: radius,
            clip: true,
            text {
                content: props.content.clone(),
                font_size: font,
                font_color: foreground,
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct CapsuleProps {
    pub left: String,
    pub right: String,
    #[props(default)]
    pub color: PaletteColor,
    #[props(default)]
    pub round: bool,
}

#[component]
pub fn Capsule(props: CapsuleProps) -> Element {
    let radius = if props.round { 999.0 } else { 3.0 };
    rsx! {
        row {
            align_items: "center",
            clip: true,
            border_radius: radius,
            Tag {
                content: props.left.clone(),
                color: props.color,
                line: false,
                round: false,
            }
            Tag {
                content: props.right.clone(),
                color: props.color,
                line: true,
                round: false,
            }
        }
    }
}
