//! Form / Field — official `field.tsx` stacked mobile form.

use arkit_component::components::{Button, ButtonVariant, FieldOrientation};
use arkit_prelude::*;

use crate::spec;
use crate::theme::use_theme;

#[component]
pub fn Form(
    submit_label: String,
    on_submit: Option<EventHandler<()>>,
    #[props(default)] submit_disabled: bool,
    #[props(default = true)] surface: bool,
    children: Element,
) -> Element {
    let theme = use_theme();
    let pad = if surface { spec::DIALOG_PAD } else { 0.0 };
    rsx! {
        column {
            width: "100%",
            align_items: "start",
            background_color: if surface { theme.colors.card } else { 0x00000000u32 },
            border_width: if surface { 1.0 } else { 0.0 },
            border_color: theme.colors.border,
            border_radius: if surface { spec::RADIUS_XL } else { 0.0 },
            shadow: if surface { "sm" },
            padding_top: pad,
            padding_right: pad,
            padding_bottom: pad,
            padding_left: pad,
            {children}
            if !submit_label.is_empty() {
                row {
                    width: "100%",
                    margin_top: 16.0,
                    Button {
                        variant: ButtonVariant::Default,
                        width: "100%",
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
    let theme = use_theme();
    let fg = if invalid {
        theme.colors.destructive
    } else {
        theme.colors.foreground
    };
    match orientation {
        FieldOrientation::Vertical => rsx! {
            column {
                width: "100%",
                align_items: "start",
                margin_bottom: 20.0,
                opacity: if disabled { spec::DISABLED_OPACITY } else { 1.0 },
                foreground_color: fg,
                {children}
            }
        },
        FieldOrientation::Horizontal => rsx! {
            row {
                width: "100%",
                align_items: "center",
                justify_content: "space-between",
                margin_bottom: 20.0,
                opacity: if disabled { spec::DISABLED_OPACITY } else { 1.0 },
                foreground_color: fg,
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
    let theme = use_theme();
    let content = if required {
        format!("{content} *")
    } else {
        content
    };
    rsx! {
        text {
            content,
            width: "100%",
            margin_bottom: 8.0,
            font_size: spec::TEXT_SM,
            font_weight: spec::FONT_MEDIUM,
            font_color: if invalid {
                theme.colors.destructive
            } else {
                theme.colors.foreground
            },
            line_height: 20.0,
        }
    }
}
