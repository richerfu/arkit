//! Badge — unstyled status chip.

use crate::appearance::{BadgeAppearance, BadgeStyleInput};
use crate::style::{use_style_kit, PaletteColor};
use arkit_prelude::*;

pub use crate::appearance::BadgeVariant;

/// Props for [`Badge`].
#[derive(Props, Clone, PartialEq)]
pub struct BadgeProps {
    pub content: String,
    #[props(default)]
    pub variant: BadgeVariant,
    pub icon: Option<String>,
    pub icon_colors: Option<(u32, u32)>,
    pub pill: Option<bool>,
    #[props(default)]
    pub color: Option<PaletteColor>,
    #[props(default)]
    pub appearance: Option<BadgeAppearance>,
}

/// A small status badge.
#[component]
pub fn Badge(props: BadgeProps) -> Element {
    let kit = use_style_kit();
    let appearance: BadgeAppearance = props.appearance.unwrap_or_else(|| {
        kit.badge(&BadgeStyleInput {
            variant: props.variant,
            pill: props.pill.unwrap_or(false),
            color: props.color,
            icon_colors: props.icon_colors,
        })
    });
    let icon = props.icon.clone();
    let content = props.content.clone();

    rsx! {
        row {
            constraint_size: format!("0,100000,{},100000", appearance.min_height),
            align_items: "center",
            justify_content: "center",
            border_radius: appearance.radius,
            background_color: appearance.background,
            border_width: appearance.border_width,
            border_color: appearance.border_color,
            clip: true,
            padding_top: appearance.padding[0],
            padding_right: appearance.padding[1],
            padding_bottom: appearance.padding[2],
            padding_left: appearance.padding[3],
            if let Some(name) = icon.as_ref() {
                {crate::icon::icon_placeholder(name, appearance.icon_size, appearance.foreground)}
                row { width: appearance.icon_gap }
            }
            text {
                content: content,
                font_size: appearance.font_size,
                font_weight: appearance.font_weight,
                font_color: appearance.foreground,
                line_height: appearance.line_height,
            }
        }
    }
}
