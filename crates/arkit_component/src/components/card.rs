//! Card — unstyled surface container.

use crate::appearance::{CardAppearance, CardStyleInput};
use crate::style::use_style_kit;
use arkit_prelude::*;

/// A card surface container.
#[derive(Props, Clone, PartialEq)]
pub struct CardProps {
    #[props(default)]
    pub shadow: Option<bool>,
    #[props(default)]
    pub appearance: Option<CardAppearance>,
    #[props(default)]
    pub native_ref: Option<arkit_arkui::NativeElementRef>,
    pub children: Element,
}

#[component]
pub fn Card(props: CardProps) -> Element {
    let kit = use_style_kit();
    let appearance: CardAppearance = props.appearance.unwrap_or_else(|| {
        kit.card(&CardStyleInput {
            shadow: props.shadow.unwrap_or(true),
        })
    });
    rsx! {
        column {
            native_ref: props.native_ref,
            width: "100%",
            align_items: "start",
            background_color: appearance.background,
            foreground_color: appearance.foreground,
            padding_top: 0.0,
            padding_right: 0.0,
            padding_bottom: 0.0,
            padding_left: 0.0,
            border_width: appearance.border_width,
            border_color: appearance.border_color,
            border_radius: appearance.radius,
            shadow: if appearance.shadow { "sm" },
            {props.children}
        }
    }
}

/// Props for [`CardHeader`].
#[derive(Props, Clone, PartialEq)]
pub struct CardHeaderProps {
    pub title: String,
    pub description: String,
    #[props(default)]
    pub appearance: Option<CardAppearance>,
}

#[component]
pub fn CardHeader(props: CardHeaderProps) -> Element {
    let kit = use_style_kit();
    let appearance = props
        .appearance
        .unwrap_or_else(|| kit.card(&CardStyleInput { shadow: false }));
    rsx! {
        row {
            width: "100%",
            justify_content: "start",
            column {
                width: "100%",
                align_items: "start",
                padding_top: appearance.header_padding[0],
                padding_right: appearance.header_padding[1],
                padding_bottom: appearance.header_padding[2],
                padding_left: appearance.header_padding[3],
                row {
                    width: "100%",
                    justify_content: "start",
                    text {
                        content: props.title.clone(),
                        font_size: appearance.title_size,
                        font_weight: appearance.title_weight,
                        font_color: appearance.foreground,
                        line_height: appearance.title_line_height,
                        text_letter_spacing: -0.35,
                        text_align: "start",
                    }
                }
                row {
                    width: "100%",
                    margin_top: 6.0,
                    justify_content: "start",
                    text {
                        content: props.description.clone(),
                        font_size: appearance.description_size,
                        font_color: appearance.description_color,
                        line_height: 20.0,
                        text_align: "start",
                    }
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct CardTitleProps {
    pub content: String,
    #[props(default)]
    pub appearance: Option<CardAppearance>,
}

#[component]
pub fn CardTitle(props: CardTitleProps) -> Element {
    let kit = use_style_kit();
    let appearance = props
        .appearance
        .unwrap_or_else(|| kit.card(&CardStyleInput { shadow: false }));
    rsx! {
        text {
            content: props.content.clone(),
            font_size: appearance.title_size,
            font_weight: appearance.title_weight,
            font_color: appearance.foreground,
            line_height: appearance.title_line_height,
            text_letter_spacing: -0.35,
            text_align: "start",
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct CardDescriptionProps {
    pub content: String,
    #[props(default)]
    pub appearance: Option<CardAppearance>,
}

#[component]
pub fn CardDescription(props: CardDescriptionProps) -> Element {
    let kit = use_style_kit();
    let appearance = props
        .appearance
        .unwrap_or_else(|| kit.card(&CardStyleInput { shadow: false }));
    rsx! {
        text {
            content: props.content.clone(),
            font_size: appearance.description_size,
            font_color: appearance.description_color,
            line_height: 20.0,
            text_align: "start",
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct CardContentProps {
    pub children: Element,
    #[props(default)]
    pub appearance: Option<CardAppearance>,
}

#[component]
pub fn CardContent(props: CardContentProps) -> Element {
    let kit = use_style_kit();
    let appearance = props
        .appearance
        .unwrap_or_else(|| kit.card(&CardStyleInput { shadow: false }));
    rsx! {
        column {
            width: "100%",
            align_items: "start",
            padding_top: appearance.content_padding[0],
            padding_right: appearance.content_padding[1],
            padding_bottom: appearance.content_padding[2],
            padding_left: appearance.content_padding[3],
            row {
                width: "100%",
                justify_content: "start",
                {props.children}
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct CardFooterProps {
    pub children: Element,
    #[props(default)]
    pub appearance: Option<CardAppearance>,
}

#[component]
pub fn CardFooter(props: CardFooterProps) -> Element {
    let kit = use_style_kit();
    let appearance = props
        .appearance
        .unwrap_or_else(|| kit.card(&CardStyleInput { shadow: false }));
    rsx! {
        row {
            width: "100%",
            align_items: "center",
            justify_content: "start",
            padding_top: appearance.footer_padding[0],
            padding_right: appearance.footer_padding[1],
            padding_bottom: appearance.footer_padding[2],
            padding_left: appearance.footer_padding[3],
            {props.children}
        }
    }
}
