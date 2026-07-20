//! Button — shadcn-style button.
//!
//! Migrated from the original Elm builder API to dioxus 0.7 `#[component]` +
//! `rsx!`. Preserves the original variants (`Default`, `Secondary`, `Outline`,
//! `Ghost`, `Destructive`, `Link`), sizes (`Default`, `Sm`, `Lg`, `Icon`), and
//! per-variant/size style computations (height, padding, text size, background,
//! foreground, border, shadow).

use crate::theme::*;
use arkit_prelude::*;

use super::{ARKUI_BORDER_STYLE_SOLID, ARKUI_BUTTON_TYPE_NORMAL};

const TRANSPARENT: u32 = 0x00000000;

/// Button visual variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonVariant {
    /// `bg-primary text-primary_foreground shadow-sm`.
    #[default]
    Default,
    /// `bg-secondary text-secondary_foreground shadow-sm`.
    Secondary,
    /// `border border-border bg-background shadow-sm`.
    Outline,
    /// No background, no shadow.
    Ghost,
    /// `bg-destructive text-destructive_foreground shadow-sm`.
    Destructive,
    /// No background, no shadow, primary-colored text.
    Link,
}

/// Button size.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonSize {
    /// Native `h-12 px-5 py-3`, 48px tall, `text-base`.
    #[default]
    Default,
    /// `h-9 px-3`, 36px tall, `text-base` on native text.
    Sm,
    /// Native `h-14 px-8`, 56px tall, `text-lg`.
    Lg,
    /// 40x40 square, no padding.
    Icon,
}

#[derive(Debug, Clone, Copy)]
struct ButtonSizeStyle {
    height: f32,
    width: Option<f32>,
    padding: [f32; 4],
    text_size: f32,
}

#[derive(Debug, Clone, Copy)]
struct ButtonVariantStyle {
    background: u32,
    foreground: u32,
    border_width: f32,
    border_color: u32,
    shadow: bool,
}

fn size_style(size: ButtonSize) -> ButtonSizeStyle {
    match size {
        ButtonSize::Default => ButtonSizeStyle {
            height: 48.0,
            width: None,
            padding: [12.0, 20.0, 12.0, 20.0],
            text_size: typography::MD,
        },
        ButtonSize::Sm => ButtonSizeStyle {
            height: 36.0,
            width: None,
            padding: [0.0, 12.0, 0.0, 12.0],
            text_size: typography::MD,
        },
        ButtonSize::Lg => ButtonSizeStyle {
            height: 56.0,
            width: None,
            padding: [0.0, 32.0, 0.0, 32.0],
            text_size: typography::LG,
        },
        ButtonSize::Icon => ButtonSizeStyle {
            height: 40.0,
            width: Some(40.0),
            padding: [0.0, 0.0, 0.0, 0.0],
            text_size: typography::MD,
        },
    }
}

fn variant_style(variant: ButtonVariant, theme: &Theme) -> ButtonVariantStyle {
    match variant {
        ButtonVariant::Default => ButtonVariantStyle {
            background: theme.colors.primary,
            foreground: theme.colors.primary_foreground,
            border_width: 0.0,
            border_color: TRANSPARENT,
            shadow: true,
        },
        ButtonVariant::Secondary => ButtonVariantStyle {
            background: theme.colors.secondary,
            foreground: theme.colors.secondary_foreground,
            border_width: 0.0,
            border_color: TRANSPARENT,
            shadow: true,
        },
        ButtonVariant::Outline => ButtonVariantStyle {
            background: theme.colors.background,
            foreground: theme.colors.foreground,
            border_width: 1.0,
            border_color: theme.colors.border,
            shadow: true,
        },
        ButtonVariant::Ghost => ButtonVariantStyle {
            background: TRANSPARENT,
            foreground: theme.colors.foreground,
            border_width: 0.0,
            border_color: TRANSPARENT,
            shadow: false,
        },
        ButtonVariant::Destructive => ButtonVariantStyle {
            background: theme.colors.destructive,
            foreground: theme.colors.destructive_foreground,
            border_width: 0.0,
            border_color: TRANSPARENT,
            shadow: true,
        },
        ButtonVariant::Link => ButtonVariantStyle {
            background: TRANSPARENT,
            foreground: theme.colors.primary,
            border_width: 0.0,
            border_color: TRANSPARENT,
            shadow: false,
        },
    }
}

/// Props for [`Button`].
#[derive(Props, Clone, PartialEq)]
pub struct ButtonProps {
    #[props(default)]
    pub variant: ButtonVariant,
    #[props(default)]
    pub size: ButtonSize,
    pub disabled: Option<bool>,
    pub percent_width: Option<f32>,
    /// Override the variant's default elevation. Passing `false` keeps the
    /// shadcn geometry and colors while rendering a flat mobile surface.
    #[props(default)]
    pub shadow: Option<bool>,
    pub onclick: Option<EventHandler<()>>,
    pub children: Element,
}

/// A button with shadcn variants and sizes.
#[component]
pub fn Button(props: ButtonProps) -> Element {
    let theme = use_theme();
    let vs = variant_style(props.variant, &theme);
    let ss = size_style(props.size);
    let disabled = props.disabled.unwrap_or(false);
    let shadow = props.shadow.unwrap_or(vs.shadow);
    let onclick = props.onclick;

    rsx! {
        button {
            button_type: ARKUI_BUTTON_TYPE_NORMAL,
            focusable: false,
            focus_on_touch: false,
            height: ss.height,
            width: if let Some(w) = ss.width { w },
            percent_width: if let Some(w) = props.percent_width { w },
            padding_top: ss.padding[0],
            padding_right: ss.padding[1],
            padding_bottom: ss.padding[2],
            padding_left: ss.padding[3],
            font_size: ss.text_size,
            font_weight: 500,
            font_color: vs.foreground,
            foreground_color: vs.foreground,
            background_color: vs.background,
            border_style: ARKUI_BORDER_STYLE_SOLID,
            border_width: vs.border_width,
            border_color: vs.border_color,
            border_radius: theme.radii.md,
            clip: true,
            alignment: 4,
            shadow: if shadow { 1 },
            opacity: if disabled { 0.5 } else { 1.0 },
            enabled: !disabled,
            onclick: move |_| {
                if !disabled {
                    if let Some(handler) = onclick {
                        handler.call(());
                    }
                }
            },
            row {
                align_items: "center",
                justify_content: "center",
                {props.children}
            }
        }
    }
}
