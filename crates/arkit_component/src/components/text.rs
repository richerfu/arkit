//! Text — shadcn-style typography primitives.
//!
//! Migrated from the original Elm builder API to dioxus 0.7 `#[component]` +
//! `rsx!`. Preserves all 11 original variants (`Default`, `H1`, `H2`, `H3`,
//! `P`, `Blockquote`, `Code`, `Lead`, `Large`, `Small`, `Muted`) with their
//! font sizes, weights, line heights, letter spacing, alignment and colors.

use crate::style::*;
use arkit_prelude::*;

const TRACKING_TIGHT: f32 = -0.35;

/// Text typography variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextVariant {
    /// `text-md` (16), foreground, 24px leading, start.
    Default,
    /// `text-4xl` (36), `W700`, 40px leading, tight tracking, center.
    H1,
    /// `text-3xl` (30), `W600`, 36px leading, tight tracking, with underline.
    H2,
    /// `text-2xl` (24), `W600`, 32px leading, tight tracking.
    H3,
    /// `text-md` (16), 28px leading. (Default struct variant.)
    #[default]
    P,
    /// Italic, 24px leading, left border accent.
    Blockquote,
    /// `bg-muted`, `text-sm` monospace `W600`, 18px leading, `sm` radius.
    Code,
    /// `text-xl` (20), muted foreground, 28px leading.
    Lead,
    /// `text-lg` (18), `W600`, 28px leading, tight tracking.
    Large,
    /// `text-sm` (14), `W500`, 14px leading.
    Small,
    /// `text-sm` (14), muted foreground, 20px leading.
    Muted,
}

/// Props for [`Text`].
#[derive(Props, Clone, PartialEq)]
pub struct TextProps {
    pub content: String,
    #[props(default)]
    pub variant: TextVariant,
}

/// Typography text with shadcn variants.
#[component]
pub fn Text(props: TextProps) -> Element {
    let theme = use_theme();
    let content = props.content.clone();
    match props.variant {
        TextVariant::Default => rsx! {
            text {
                content: content,
                font_size: typography::MD,
                font_color: theme.colors.foreground,
                line_height: 24.0,
                text_align: "start",
            }
        },
        TextVariant::H1 => rsx! {
            text {
                content: content,
                font_size: 36.0,
                font_weight: 700,
                font_color: theme.colors.foreground,
                line_height: 40.0,
                text_letter_spacing: TRACKING_TIGHT,
                text_align: "center",
            }
        },
        TextVariant::H2 => rsx! {
            column {
                width: "100%",
                text {
                    content: content,
                    font_size: 30.0,
                    font_weight: 600,
                    font_color: theme.colors.foreground,
                    line_height: 36.0,
                    text_letter_spacing: TRACKING_TIGHT,
                    text_align: "start",
                }
                row {
                    width: "100%",
                    height: 1.0,
                    margin_top: 8.0,
                    background_color: theme.colors.border,
                }
            }
        },
        TextVariant::H3 => rsx! {
            text {
                content: content,
                font_size: 24.0,
                font_weight: 600,
                font_color: theme.colors.foreground,
                line_height: 32.0,
                text_letter_spacing: TRACKING_TIGHT,
                text_align: "start",
            }
        },
        TextVariant::P => rsx! {
            text {
                content: content,
                font_size: typography::MD,
                font_color: theme.colors.foreground,
                line_height: 28.0,
                text_align: "start",
            }
        },
        TextVariant::Blockquote => rsx! {
            row {
                width: "100%",
                align_items: "start",
                column {
                    width: 2.0,
                    background_color: theme.colors.border,
                }
                text {
                    margin_left: 12.0,
                    content: content,
                    font_size: typography::MD,
                    font_color: theme.colors.foreground,
                    font_style: "italic",
                    line_height: 24.0,
                    text_align: "start",
                }
            }
        },
        TextVariant::Code => rsx! {
            row {
                background_color: theme.colors.muted,
                padding_top: 3.0,
                padding_right: 5.0,
                padding_bottom: 3.0,
                padding_left: 5.0,
                border_radius: theme.radii.sm,
                text {
                    content: content,
                    font_size: typography::SM,
                    font_family: "monospace",
                    font_weight: 600,
                    font_color: theme.colors.foreground,
                    line_height: 18.0,
                }
            }
        },
        TextVariant::Lead => rsx! {
            text {
                content: content,
                font_size: typography::XL,
                font_color: theme.colors.muted_foreground,
                line_height: 28.0,
                text_align: "start",
            }
        },
        TextVariant::Large => rsx! {
            text {
                content: content,
                font_size: typography::LG,
                font_weight: 600,
                font_color: theme.colors.foreground,
                line_height: 28.0,
                text_letter_spacing: TRACKING_TIGHT,
                text_align: "start",
            }
        },
        TextVariant::Small => rsx! {
            text {
                content: content,
                font_size: typography::SM,
                font_weight: 500,
                font_color: theme.colors.foreground,
                line_height: 14.0,
                text_align: "start",
            }
        },
        TextVariant::Muted => rsx! {
            text {
                content: content,
                font_size: typography::SM,
                font_color: theme.colors.muted_foreground,
                line_height: 20.0,
                text_align: "start",
            }
        },
    }
}
