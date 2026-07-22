//! Card — shadcn-style surface container.
//!
//! Migrated from the original Elm builder API to dioxus 0.7 `#[component]` +
//! `rsx!`. Mirrors React Native Reusables: the card shell owns only surface
//! styling, while Header/Content/Footer own their own `p-6` padding.

use crate::theme::*;
use arkit_prelude::*;

/// A card surface container. Wraps its children in a bordered, rounded,
/// shadowed column with the theme's `card` background.
#[derive(Props, Clone, PartialEq)]
pub struct CardProps {
    /// Override the default small card elevation. Set to `false` for the
    /// border-only shadcn surface commonly used in dense mobile layouts.
    #[props(default)]
    pub shadow: Option<bool>,
    pub children: Element,
}

#[component]
pub fn Card(props: CardProps) -> Element {
    let theme = use_theme();
    let shadow = props.shadow.unwrap_or(true);
    rsx! {
        column {
            width: "100%",
            align_items: "start",
            background_color: theme.colors.card,
            foreground_color: theme.colors.card_foreground,
            padding_top: 0.0,
            padding_right: 0.0,
            padding_bottom: 0.0,
            padding_left: 0.0,
            border_width: 1.0,
            border_color: theme.colors.border,
            border_radius: theme.radii.lg,
            shadow: if shadow { "sm" },
            {props.children}
        }
    }
}

/// Props for [`CardHeader`].
#[derive(Props, Clone, PartialEq)]
pub struct CardHeaderProps {
    pub title: String,
    pub description: String,
}

/// Card header — `p-6` with title and muted description stacked at `space-y-1.5`.
#[component]
pub fn CardHeader(props: CardHeaderProps) -> Element {
    let theme = use_theme();
    rsx! {
        row {
            width: "100%",
            justify_content: "start",
            column {
                width: "100%",
                align_items: "start",
                padding_top: spacing::XXL,
                padding_right: spacing::XXL,
                padding_bottom: spacing::XXL,
                padding_left: spacing::XXL,
                row {
                    width: "100%",
                    justify_content: "start",
                    text {
                        content: props.title.clone(),
                        font_size: typography::XXL,
                        font_weight: 600,
                        font_color: theme.colors.card_foreground,
                        line_height: 24.0,
                        text_letter_spacing: -0.35,
                        text_align: "start",
                    }
                }
                row {
                    width: "100%",
                    margin_top: spacing::XS,
                    justify_content: "start",
                    text {
                        content: props.description.clone(),
                        font_size: typography::SM,
                        font_color: theme.colors.muted_foreground,
                        line_height: 20.0,
                        text_align: "start",
                    }
                }
            }
        }
    }
}

/// Props for [`CardTitle`].
#[derive(Props, Clone, PartialEq)]
pub struct CardTitleProps {
    pub content: String,
}

/// Standalone card title — `text-2xl font-semibold leading-none tracking-tight`.
#[component]
pub fn CardTitle(props: CardTitleProps) -> Element {
    let theme = use_theme();
    rsx! {
        text {
            content: props.content.clone(),
            font_size: typography::XXL,
            font_weight: 600,
            font_color: theme.colors.card_foreground,
            line_height: 24.0,
            text_letter_spacing: -0.35,
            text_align: "start",
        }
    }
}

/// Props for [`CardDescription`].
#[derive(Props, Clone, PartialEq)]
pub struct CardDescriptionProps {
    pub content: String,
}

/// Standalone card description — muted, small supporting text.
#[component]
pub fn CardDescription(props: CardDescriptionProps) -> Element {
    let theme = use_theme();
    rsx! {
        text {
            content: props.content.clone(),
            font_size: typography::SM,
            font_color: theme.colors.muted_foreground,
            line_height: 20.0,
            text_align: "start",
        }
    }
}

/// Props for [`CardContent`].
#[derive(Props, Clone, PartialEq)]
pub struct CardContentProps {
    pub children: Element,
}

/// Card content region — `p-6 pt-0`.
#[component]
pub fn CardContent(props: CardContentProps) -> Element {
    rsx! {
        column {
            width: "100%",
            align_items: "start",
            padding_top: 0.0,
            padding_right: spacing::XXL,
            padding_bottom: spacing::XXL,
            padding_left: spacing::XXL,
            row {
                width: "100%",
                justify_content: "start",
                {props.children}
            }
        }
    }
}

/// Props for [`CardFooter`].
#[derive(Props, Clone, PartialEq)]
pub struct CardFooterProps {
    pub children: Element,
}

/// Card footer — `flex-row items-center p-6 pt-0`.
#[component]
pub fn CardFooter(props: CardFooterProps) -> Element {
    rsx! {
        row {
            width: "100%",
            align_items: "center",
            justify_content: "start",
            padding_top: 0.0,
            padding_right: spacing::XXL,
            padding_bottom: spacing::XXL,
            padding_left: spacing::XXL,
            {props.children}
        }
    }
}
