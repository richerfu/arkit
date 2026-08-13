//! Alert — shadcn-style contextual feedback container.
//!
//! Migrated from the original Elm builder API to dioxus 0.7 `#[component]` +
//! `rsx!`. Preserves the original variants (`Default`, `Destructive`), the
//! per-variant tone mapping, icon positioning, and the title/description/list
//! spacing constants.

use crate::appearance::{AlertAppearance, AlertStyleInput};
use crate::style::{typography, use_style_kit};
use arkit_prelude::*;

pub use crate::appearance::AlertVariant;

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

/// Props for [`Alert`].
#[derive(Props, Clone, PartialEq)]
pub struct AlertProps {
    /// Lucide icon name rendered at the top-left of the alert.
    pub icon: String,
    #[props(default)]
    pub variant: AlertVariant,
    #[props(default)]
    pub appearance: Option<AlertAppearance>,
    pub children: Element,
}

/// Alert root — a bordered, rounded card with an icon pinned top-left and a
/// padded content column.
#[component]
pub fn Alert(props: AlertProps) -> Element {
    let kit = use_style_kit();
    let appearance = props.appearance.unwrap_or_else(|| {
        kit.alert(&AlertStyleInput {
            variant: props.variant,
        })
    });
    let icon = props.icon.clone();
    let icon_position = format!("{ALERT_ICON_LEFT},{ALERT_ICON_TOP}");
    rsx! {
        stack {
            width: "100%",
            border_radius: appearance.radius,
            border_width: appearance.border_width,
            border_color: appearance.border_color,
            background_color: appearance.background,
            row {
                width: ALERT_ICON_SIZE,
                height: ALERT_ICON_SIZE,
                position: icon_position,
                align_items: "center",
                justify_content: "center",
                {arkit_icon::icon(icon, ALERT_ICON_SIZE, appearance.icon_color)}
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
    #[props(default)]
    pub appearance: Option<AlertAppearance>,
}

/// Alert title — small, medium-weight text colored by the variant tone.
#[component]
pub fn AlertTitle(props: AlertTitleProps) -> Element {
    let kit = use_style_kit();
    let appearance = props.appearance.unwrap_or_else(|| {
        kit.alert(&AlertStyleInput {
            variant: props.variant,
        })
    });
    rsx! {
        text {
            content: props.content.clone(),
            width: "100%",
            font_size: typography::SM,
            font_weight: 500,
            font_color: appearance.title_color,
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
    #[props(default)]
    pub appearance: Option<AlertAppearance>,
}

/// Alert description — small, relaxed-line-height supporting text.
#[component]
pub fn AlertDescription(props: AlertDescriptionProps) -> Element {
    let kit = use_style_kit();
    let appearance = props.appearance.unwrap_or_else(|| {
        kit.alert(&AlertStyleInput {
            variant: props.variant,
        })
    });
    rsx! {
        text {
            content: props.content.clone(),
            width: "100%",
            font_size: typography::SM,
            font_color: appearance.description_color,
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
    #[props(default)]
    pub appearance: Option<AlertAppearance>,
}

/// Alert list — a bulleted (`•`) column of small text lines, with a small gap
/// between non-first rows.
#[component]
pub fn AlertList(props: AlertListProps) -> Element {
    let kit = use_style_kit();
    let appearance = props.appearance.unwrap_or_else(|| {
        kit.alert(&AlertStyleInput {
            variant: props.variant,
        })
    });
    let title_color = appearance.title_color;
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
