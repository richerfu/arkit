//! Button — shadcn-style button.
//!
//! Migrated from the original Elm builder API to dioxus 0.7 `#[component]` +
//! `rsx!`. Preserves the original variants (`Default`, `Secondary`, `Outline`,
//! `Ghost`, `Destructive`, `Link`), sizes (`Default`, `Sm`, `Lg`, `Icon`), and
//! per-variant/size style computations (height, padding, text size, background,
//! foreground, border). Default density follows shadcn New York (`h-9` / 36vp).

use crate::theme::*;
use arkit_prelude::*;

use super::ARKUI_BORDER_STYLE_SOLID;

const TRANSPARENT: u32 = 0x00000000;

/// Button visual variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonVariant {
    /// `bg-primary text-primary_foreground`.
    #[default]
    Default,
    /// `bg-secondary text-secondary_foreground`.
    Secondary,
    /// `border border-border bg-background`.
    Outline,
    /// No background, no shadow.
    Ghost,
    /// `bg-destructive text-destructive_foreground`.
    Destructive,
    /// No background, no shadow, primary-colored text.
    Link,
}

/// Button size.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonSize {
    /// shadcn New York `h-9 px-4`, 36vp tall, `text-sm`.
    #[default]
    Default,
    /// `h-8 px-3`, 32vp tall, `text-xs`.
    Sm,
    /// `h-10 px-6`, 40vp tall, `text-sm`.
    Lg,
    /// 36×36 square, no padding (`size-9`).
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
            height: control::HEIGHT,
            width: None,
            padding: [0.0, 16.0, 0.0, 16.0],
            text_size: typography::SM,
        },
        ButtonSize::Sm => ButtonSizeStyle {
            height: control::HEIGHT_SM,
            width: None,
            padding: [0.0, 12.0, 0.0, 12.0],
            text_size: typography::XS,
        },
        ButtonSize::Lg => ButtonSizeStyle {
            height: control::HEIGHT_LG,
            width: None,
            padding: [0.0, 24.0, 0.0, 24.0],
            text_size: typography::SM,
        },
        ButtonSize::Icon => ButtonSizeStyle {
            height: control::ICON,
            width: Some(control::ICON),
            padding: [0.0, 0.0, 0.0, 0.0],
            text_size: typography::SM,
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
            shadow: false,
        },
        ButtonVariant::Secondary => ButtonVariantStyle {
            background: theme.colors.secondary,
            foreground: theme.colors.secondary_foreground,
            border_width: 0.0,
            border_color: TRANSPARENT,
            shadow: false,
        },
        ButtonVariant::Outline => ButtonVariantStyle {
            background: theme.colors.background,
            foreground: theme.colors.foreground,
            border_width: 1.0,
            border_color: theme.colors.border,
            shadow: false,
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
            shadow: false,
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
    /// Optional exact height override for compact host-app surfaces.
    #[props(default)]
    pub height: Option<f32>,
    /// Optional exact corner radius override.
    #[props(default)]
    pub border_radius: Option<f32>,
    pub disabled: Option<bool>,
    /// CSS width (`"100%"`, `"48%"`, `"120"`). When unset, size defaults apply.
    pub width: Option<String>,
    /// Override elevation. New York variants are flat by default; pass `true`
    /// to opt into a small drop shadow.
    #[props(default)]
    pub shadow: Option<bool>,
    /// Exact reference forwarded to the button's native root.
    #[props(default)]
    pub native_ref: Option<arkit_arkui::NativeElementRef>,
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
            native_ref: props.native_ref,
            button_type: "normal",
            focusable: false,
            focus_on_touch: false,
            height: props.height.unwrap_or(ss.height),
            width: if let Some(w) = props.width {
                w
            } else if let Some(w) = ss.width {
                format!("{w}")
            },
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
            border_radius: props.border_radius.unwrap_or(theme.radii.md),
            clip: true,
            alignment: "center",
            shadow: if shadow { "sm" },
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

#[cfg(test)]
mod tests {
    use super::{control, size_style, typography, variant_style, ButtonSize, ButtonVariant, Theme};

    #[test]
    fn new_york_sizes_match_control_tokens() {
        let default = size_style(ButtonSize::Default);
        assert_eq!(default.height, control::HEIGHT);
        assert_eq!(default.text_size, typography::SM);
        assert_eq!(default.padding, [0.0, 16.0, 0.0, 16.0]);

        let sm = size_style(ButtonSize::Sm);
        assert_eq!(sm.height, control::HEIGHT_SM);
        assert_eq!(sm.text_size, typography::XS);

        let lg = size_style(ButtonSize::Lg);
        assert_eq!(lg.height, control::HEIGHT_LG);
        assert_eq!(lg.text_size, typography::SM);

        let icon = size_style(ButtonSize::Icon);
        assert_eq!(icon.height, control::ICON);
        assert_eq!(icon.width, Some(control::ICON));
    }

    #[test]
    fn filled_variants_are_flat() {
        let theme = Theme::default();
        for variant in [
            ButtonVariant::Default,
            ButtonVariant::Secondary,
            ButtonVariant::Outline,
            ButtonVariant::Destructive,
            ButtonVariant::Ghost,
            ButtonVariant::Link,
        ] {
            assert!(!variant_style(variant, &theme).shadow);
        }
    }
}
