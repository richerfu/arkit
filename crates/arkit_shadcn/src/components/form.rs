//! Form — shadcn-style form wrapper.
//!
//! Migrated from the original Elm builder API to dioxus 0.7 `#[component]` +
//! `rsx!`. A form renders a card surface (card background, border, `xl` radius,
//! `shadow-sm`, `XXL` horizontal padding) containing its field children plus a
//! full-width submit button. [`FormItem`] stacks a label above a field.

use super::button::{Button, ButtonVariant};
use super::label::Label;
use crate::theme::*;
use arkit_prelude::*;

/// Props for [`Form`].
#[derive(Props, Clone, PartialEq)]
pub struct FormProps {
    pub submit_label: String,
    pub on_submit: Option<EventHandler<()>>,
    pub children: Element,
}

/// A card-surfaced form with a submit button.
#[component]
pub fn Form(props: FormProps) -> Element {
    let theme = use_theme();
    let on_submit = props.on_submit;
    let submit_label = props.submit_label.clone();
    rsx! {
        column {
            percent_width: 1.0,
            background_color: theme.colors.card,
            foreground_color: theme.colors.card_foreground,
            border_width: 1.0,
            border_color: theme.colors.border,
            border_radius: theme.radii.xl,
            shadow: 1,
            padding_top: 0.0,
            padding_right: spacing::XXL,
            padding_bottom: 0.0,
            padding_left: spacing::XXL,
            {props.children}
            row {
                margin_top: spacing::SM,
                Button {
                    variant: ButtonVariant::Default,
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

/// Props for [`FormItem`].
#[derive(Props, Clone, PartialEq)]
pub struct FormItemProps {
    pub label: String,
    pub children: Element,
}

/// A labelled form field — a label above the field input.
#[component]
pub fn FormItem(props: FormItemProps) -> Element {
    rsx! {
        column {
            percent_width: 1.0,
            Label { content: props.label.clone() }
            row {
                margin_top: spacing::XXS,
                {props.children}
            }
        }
    }
}
