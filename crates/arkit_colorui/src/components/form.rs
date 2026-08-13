//! ColorUI form — `.cu-form-group` rows instead of a shadcn card.

use arkit_component::components::{Button, ButtonVariant, FieldOrientation};
use arkit_prelude::*;

use super::chrome::PADDING;
use crate::theme::use_colorui_theme;

#[component]
pub fn Form(
    submit_label: String,
    on_submit: Option<EventHandler<()>>,
    #[props(default)] submit_disabled: bool,
    #[props(default = true)] surface: bool,
    children: Element,
) -> Element {
    let tokens = use_colorui_theme().tokens();
    let on_submit = on_submit;
    let submit_label = submit_label.clone();

    rsx! {
        column {
            width: "100%",
            align_items: "start",
            background_color: if surface { tokens.colors.card } else { 0x00000000u32 },
            {children}
            if !submit_label.is_empty() {
                column {
                    width: "100%",
                    padding_top: PADDING,
                    padding_right: PADDING,
                    padding_bottom: PADDING,
                    padding_left: PADDING,
                    Button {
                        variant: ButtonVariant::Default,
                        width: "100%",
                        block: Some(true),
                        disabled: Some(submit_disabled),
                        onclick: move |_| {
                            if let Some(handler) = on_submit {
                                handler.call(());
                            }
                        },
                        "{submit_label}"
                    }
                }
            }
        }
    }
}

#[component]
pub fn Field(
    #[props(default)] orientation: FieldOrientation,
    #[props(default)] invalid: bool,
    #[props(default)] disabled: bool,
    children: Element,
) -> Element {
    let tokens = use_colorui_theme().tokens();
    let foreground = if invalid {
        tokens.colors.destructive
    } else {
        tokens.colors.foreground
    };

    match orientation {
        FieldOrientation::Vertical => rsx! {
            column {
                width: "100%",
                align_items: "start",
                background_color: tokens.colors.card,
                padding_top: PADDING,
                padding_right: PADDING,
                padding_bottom: PADDING,
                padding_left: PADDING,
                opacity: if disabled { 0.6 } else { 1.0 },
                foreground_color: foreground,
                {children}
            }
        },
        FieldOrientation::Horizontal => rsx! {
            row {
                width: "100%",
                min_height: 50.0,
                align_items: "center",
                justify_content: "space-between",
                background_color: tokens.colors.card,
                padding_left: PADDING,
                padding_right: PADDING,
                opacity: if disabled { 0.6 } else { 1.0 },
                foreground_color: foreground,
                {children}
            }
        },
    }
}

#[component]
pub fn FieldLabel(
    content: String,
    #[props(default)] required: bool,
    #[props(default)] invalid: bool,
) -> Element {
    let tokens = use_colorui_theme().tokens();
    let content = if required {
        format!("{content} *")
    } else {
        content
    };

    rsx! {
        text {
            content,
            font_size: 15.0,
            font_weight: 400,
            font_color: if invalid {
                tokens.colors.destructive
            } else {
                tokens.colors.foreground
            },
            line_height: 20.0,
            text_align: "start",
            margin_right: 12.0,
        }
    }
}
