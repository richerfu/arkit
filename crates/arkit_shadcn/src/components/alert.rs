//! Alert — shadcn-style contextual feedback container.
//!
//! Migrated from the original Elm builder API to dioxus 0.7 `#[component]` +
//! `rsx!`. Preserves the original variants (`Default`, `Destructive`), the
//! per-variant tone mapping, icon positioning, and the title/description/list
//! spacing constants.

use crate::theme::*;
use arkit_prelude::*;

const ALERT_BORDER_WIDTH: f32 = 1.0;
const ALERT_ICON_SIZE: f32 = 16.0;
const ALERT_ICON_LEFT: f32 = 14.0;
const ALERT_ICON_TOP: f32 = 12.0;
const ALERT_PADDING_TOP: f32 = 14.0;
const ALERT_PADDING_RIGHT: f32 = 16.0;
const ALERT_PADDING_BOTTOM: f32 = 8.0;
const ALERT_PADDING_LEFT: f32 = 16.0;
const ALERT_CONTENT_LEFT: f32 = 24.0;
const ALERT_CONTENT_OFFSET: f32 = 2.0;
const ALERT_TITLE_BOTTOM: f32 = 4.0;
const ALERT_DESCRIPTION_BOTTOM: f32 = 6.0;
const ALERT_LIST_BOTTOM: f32 = 8.0;
const ALERT_TRACKING_TIGHT: f32 = -0.2;
const ALERT_TITLE_LINE_HEIGHT: f32 = 14.0;
// Tailwind `leading-relaxed` for `text-sm`: 14 * 1.625 = 22.75
const ALERT_DESCRIPTION_LINE_HEIGHT: f32 = 22.75;
const ALERT_LIST_LINE_HEIGHT: f32 = 20.0;

/// Alert visual variant. `Default` uses foreground tones; `Destructive` uses
/// the destructive color for title/icon and a translucent destructive tone for
/// the description.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AlertVariant {
    #[default]
    Default,
    Destructive,
}

#[derive(Debug, Clone, Copy)]
struct AlertTone {
    title_color: u32,
    description_color: u32,
    icon_color: u32,
}

fn alert_tone(variant: AlertVariant, theme: &Theme) -> AlertTone {
    match variant {
        AlertVariant::Default => AlertTone {
            title_color: theme.colors.foreground,
            description_color: theme.colors.muted_foreground,
            icon_color: theme.colors.foreground,
        },
        AlertVariant::Destructive => AlertTone {
            title_color: theme.colors.destructive,
            description_color: with_alpha(theme.colors.destructive, 0xE6),
            icon_color: theme.colors.destructive,
        },
    }
}

/// Props for [`Alert`].
#[derive(Props, Clone, PartialEq)]
pub struct AlertProps {
    /// Lucide icon name rendered at the top-left of the alert.
    pub icon: String,
    #[props(default)]
    pub variant: AlertVariant,
    pub children: Element,
}

/// Alert root — a bordered, rounded card with an icon pinned top-left and a
/// padded content column.
#[component]
pub fn Alert(props: AlertProps) -> Element {
    let theme = use_theme();
    let tone = alert_tone(props.variant, &theme);
    let icon = props.icon.clone();
    let icon_position = format!("{ALERT_ICON_LEFT},{ALERT_ICON_TOP}");
    let border_color = match props.variant {
        AlertVariant::Default => theme.colors.border,
        AlertVariant::Destructive => with_alpha(theme.colors.destructive, 0x80),
    };
    rsx! {
        stack {
            width: "100%",
            border_radius: theme.radii.lg,
            border_width: ALERT_BORDER_WIDTH,
            border_color,
            background_color: theme.colors.card,
            row {
                width: ALERT_ICON_SIZE,
                height: ALERT_ICON_SIZE,
                position: icon_position,
                align_items: "center",
                justify_content: "center",
                {arkit_icon::icon(icon, ALERT_ICON_SIZE, tone.icon_color)}
            }
            column {
                width: "100%",
                align_items: "start",
                padding_top: ALERT_PADDING_TOP,
                padding_right: ALERT_PADDING_RIGHT,
                padding_bottom: ALERT_PADDING_BOTTOM,
                padding_left: ALERT_PADDING_LEFT,
                {props.children}
            }
        }
    }
}

/// Props for [`AlertTitle`].
#[derive(Props, Clone, PartialEq)]
pub struct AlertTitleProps {
    pub content: String,
    #[props(default)]
    pub variant: AlertVariant,
}

/// Alert title — small, medium-weight text colored by the variant tone.
#[component]
pub fn AlertTitle(props: AlertTitleProps) -> Element {
    let theme = use_theme();
    let tone = alert_tone(props.variant, &theme);
    rsx! {
        text {
            content: props.content.clone(),
            width: "100%",
            font_size: typography::SM,
            font_weight: 500,
            font_color: tone.title_color,
            line_height: ALERT_TITLE_LINE_HEIGHT,
            text_letter_spacing: ALERT_TRACKING_TIGHT,
            text_align: "start",
            margin_top: 0.0,
            margin_right: 0.0,
            margin_bottom: ALERT_TITLE_BOTTOM,
            margin_left: ALERT_CONTENT_OFFSET,
            padding_left: ALERT_CONTENT_LEFT,
        }
    }
}

/// Props for [`AlertDescription`].
#[derive(Props, Clone, PartialEq)]
pub struct AlertDescriptionProps {
    pub content: String,
    #[props(default)]
    pub variant: AlertVariant,
}

/// Alert description — small, relaxed-line-height supporting text.
#[component]
pub fn AlertDescription(props: AlertDescriptionProps) -> Element {
    let theme = use_theme();
    let tone = alert_tone(props.variant, &theme);
    rsx! {
        text {
            content: props.content.clone(),
            width: "100%",
            font_size: typography::SM,
            font_color: tone.description_color,
            line_height: ALERT_DESCRIPTION_LINE_HEIGHT,
            text_align: "start",
            margin_top: 0.0,
            margin_right: 0.0,
            margin_bottom: 0.0,
            margin_left: ALERT_CONTENT_OFFSET,
            padding_top: 0.0,
            padding_right: 0.0,
            padding_bottom: ALERT_DESCRIPTION_BOTTOM,
            padding_left: ALERT_CONTENT_LEFT,
        }
    }
}

/// Props for [`AlertList`].
#[derive(Props, Clone, PartialEq)]
pub struct AlertListProps {
    pub items: Vec<String>,
    #[props(default)]
    pub variant: AlertVariant,
}

/// Alert list — a bulleted (`•`) column of small text lines, with a small gap
/// between non-first rows.
#[component]
pub fn AlertList(props: AlertListProps) -> Element {
    let theme = use_theme();
    let tone = alert_tone(props.variant, &theme);
    let title_color = tone.title_color;
    let rows: Vec<Element> = props
        .items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let text = format!("\u{2022} {item}");
            if index == 0 {
                rsx! {
                    text {
                        content: text,
                        width: "100%",
                        font_size: typography::SM,
                        font_color: title_color,
                        line_height: ALERT_LIST_LINE_HEIGHT,
                        text_align: "start",
                    }
                }
            } else {
                rsx! {
                    row {
                        width: "100%",
                        align_items: "start",
                        margin_top: 2.0,
                        text {
                            content: text,
                            width: "100%",
                            font_size: typography::SM,
                            font_color: title_color,
                            line_height: ALERT_LIST_LINE_HEIGHT,
                            text_align: "start",
                        }
                    }
                }
            }
        })
        .collect();
    rsx! {
        column {
            width: "100%",
            align_items: "start",
            margin_top: 0.0,
            margin_right: 0.0,
            margin_bottom: 0.0,
            margin_left: ALERT_CONTENT_OFFSET,
            padding_top: 0.0,
            padding_right: 0.0,
            padding_bottom: ALERT_LIST_BOTTOM,
            padding_left: ALERT_CONTENT_LEFT,
            {rows.iter().cloned()}
        }
    }
}
