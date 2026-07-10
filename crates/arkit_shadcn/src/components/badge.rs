//! Badge — shadcn-style badge.
//!
//! Migrated from the original Elm builder API to dioxus 0.7 `#[component]` +
//! `rsx!`. Preserves the original variants (`Default`, `Secondary`,
//! `Destructive`, `Outline`), the pill option, the icon+label layout, and the
//! spacing/typography constants (`XS` text, `W500`, 16px line height, 2px
//! vertical padding, `SM` horizontal padding, 22px min height).

use crate::theme::*;
use arkit_prelude::*;

const BADGE_ICON_SIZE: f32 = 12.0;
const BADGE_VERTICAL_PADDING: f32 = 2.0;
const BADGE_ICON_GAP: f32 = 4.0;
const BADGE_TEXT_LINE_HEIGHT: f32 = 16.0;
const BADGE_MIN_HEIGHT: f32 = 22.0;

/// Badge visual variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BadgeVariant {
    /// `bg-primary text-primary_foreground`.
    #[default]
    Default,
    /// `bg-secondary text-secondary_foreground`.
    Secondary,
    /// `bg-destructive text-destructive_foreground`.
    Destructive,
    /// `bg-background text-foreground border-border`.
    Outline,
}

#[derive(Debug, Clone, Copy)]
struct BadgeStyle {
    background: u32,
    foreground: u32,
    border_width: f32,
    border_color: u32,
}

fn badge_style(variant: BadgeVariant, theme: &Theme) -> BadgeStyle {
    match variant {
        BadgeVariant::Default => BadgeStyle {
            background: theme.colors.primary,
            foreground: theme.colors.primary_foreground,
            border_width: 1.0,
            border_color: 0x00000000,
        },
        BadgeVariant::Secondary => BadgeStyle {
            background: theme.colors.secondary,
            foreground: theme.colors.secondary_foreground,
            border_width: 1.0,
            border_color: 0x00000000,
        },
        BadgeVariant::Destructive => BadgeStyle {
            background: theme.colors.destructive,
            foreground: theme.colors.destructive_foreground,
            border_width: 1.0,
            border_color: 0x00000000,
        },
        BadgeVariant::Outline => BadgeStyle {
            background: theme.colors.background,
            foreground: theme.colors.foreground,
            border_width: 1.0,
            border_color: theme.colors.border,
        },
    }
}

/// Props for [`Badge`].
#[derive(Props, Clone, PartialEq)]
pub struct BadgeProps {
    pub content: String,
    #[props(default)]
    pub variant: BadgeVariant,
    pub icon: Option<String>,
    pub icon_colors: Option<(u32, u32)>,
    pub pill: Option<bool>,
}

/// A small status badge.
#[component]
pub fn Badge(props: BadgeProps) -> Element {
    let theme = use_theme();
    let style = if let Some((background, foreground)) = props.icon_colors {
        BadgeStyle {
            background,
            foreground,
            border_width: 1.0,
            border_color: 0x00000000,
        }
    } else {
        badge_style(props.variant, &theme)
    };
    let pill = props.pill.unwrap_or(false);
    let radius = if pill {
        theme.radii.full
    } else {
        theme.radii.md
    };
    let hpad = if pill { spacing::XXS } else { spacing::SM };
    let icon = props.icon.clone();
    let content = props.content.clone();

    rsx! {
        row {
            constraint_size: format!("0,100000,{BADGE_MIN_HEIGHT},100000"),
            align_items: "center",
            justify_content: "center",
            border_radius: radius,
            background_color: style.background,
            border_width: style.border_width,
            border_color: style.border_color,
            clip: true,
            padding_top: BADGE_VERTICAL_PADDING,
            padding_right: hpad,
            padding_bottom: BADGE_VERTICAL_PADDING,
            padding_left: hpad,
            if let Some(name) = icon.as_ref() {
                {crate::icon::icon_placeholder(name, BADGE_ICON_SIZE, style.foreground)}
                row { width: BADGE_ICON_GAP }
            }
            text {
                content: content,
                font_size: typography::XS,
                font_weight: 500,
                font_color: style.foreground,
                line_height: BADGE_TEXT_LINE_HEIGHT,
            }
        }
    }
}
