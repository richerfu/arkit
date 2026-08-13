//! Form and field composition primitives.
//!
//! The form root keeps the original Arkit card-and-submit API for backwards
//! compatibility. The `Field*` family mirrors the current shadcn composition
//! model: forms own their state and validation, while these components provide
//! consistent mobile layout for labels, controls, descriptions, groups, and
//! validation messages.

use super::button::{Button, ButtonVariant};
use crate::style::*;
use arkit_prelude::*;

/// Props for [`Form`].
#[derive(Props, Clone, PartialEq)]
pub struct FormProps {
    pub submit_label: String,
    pub on_submit: Option<EventHandler<()>>,
    /// Disables the built-in submit action without changing its layout.
    #[props(default)]
    pub submit_disabled: bool,
    /// Renders the legacy card surface. Disable this when the surrounding page
    /// already provides its own form surface.
    #[props(default = true)]
    pub surface: bool,
    pub children: Element,
}

/// A mobile form surface with an optional built-in submit action.
///
/// Validation and field state remain controlled by the caller. An empty
/// `submit_label` omits the built-in action, allowing fully compositional use.
#[component]
pub fn Form(props: FormProps) -> Element {
    let theme = use_theme();
    let on_submit = props.on_submit;
    let submit_label = props.submit_label.clone();
    let padding = if props.surface { spacing::XXL } else { 0.0 };

    rsx! {
        column {
            width: "100%",
            align_items: "start",
            background_color: if props.surface { theme.colors.card } else { 0x00000000 },
            foreground_color: theme.colors.card_foreground,
            border_width: if props.surface { 1.0 } else { 0.0 },
            border_color: if props.surface { theme.colors.border } else { 0x00000000 },
            border_radius: if props.surface { theme.radii.xl } else { 0.0 },
            shadow: if props.surface { "sm" },
            padding_top: padding,
            padding_right: padding,
            padding_bottom: padding,
            padding_left: padding,
            {props.children}
            if !submit_label.is_empty() {
                row {
                    width: "100%",
                    Button {
                        variant: ButtonVariant::Default,
                        width: "100%",
                        disabled: Some(props.submit_disabled),
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

/// Layout direction for [`Field`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FieldOrientation {
    /// Label, control, description, and error are stacked for a mobile form.
    #[default]
    Vertical,
    /// Content and control share one row, suitable for switches and checkboxes.
    Horizontal,
}

/// Props for [`Field`].
#[derive(Props, Clone, PartialEq)]
pub struct FieldProps {
    #[props(default)]
    pub orientation: FieldOrientation,
    #[props(default)]
    pub invalid: bool,
    #[props(default)]
    pub disabled: bool,
    pub children: Element,
}

/// A single field layout primitive.
///
/// `invalid` exposes the field state to the native foreground context; controls
/// should also receive their own invalid state so their border is updated.
#[component]
pub fn Field(props: FieldProps) -> Element {
    let theme = use_theme();
    let foreground = if props.invalid {
        theme.colors.destructive
    } else {
        theme.colors.foreground
    };

    match props.orientation {
        FieldOrientation::Vertical => rsx! {
            column {
                width: "100%",
                align_items: "start",
                margin_bottom: spacing::XL,
                opacity: if props.disabled { 0.5 } else { 1.0 },
                foreground_color: foreground,
                {props.children}
            }
        },
        FieldOrientation::Horizontal => rsx! {
            row {
                width: "100%",
                align_items: "center",
                justify_content: "space_between",
                margin_bottom: spacing::XL,
                opacity: if props.disabled { 0.5 } else { 1.0 },
                foreground_color: foreground,
                {props.children}
            }
        },
    }
}

/// Props for [`FieldContent`].
#[derive(Props, Clone, PartialEq)]
pub struct FieldContentProps {
    pub children: Element,
}

/// Groups a field title and description beside a horizontal control.
#[component]
pub fn FieldContent(props: FieldContentProps) -> Element {
    rsx! {
        column {
            layout_weight: 1.0,
            align_items: "start",
            margin_right: spacing::LG,
            {props.children}
        }
    }
}

/// Props for [`FieldLabel`].
#[derive(Props, Clone, PartialEq)]
pub struct FieldLabelProps {
    pub content: String,
    #[props(default)]
    pub required: bool,
    #[props(default)]
    pub invalid: bool,
}

/// A field label with an optional required marker.
#[component]
pub fn FieldLabel(props: FieldLabelProps) -> Element {
    let theme = use_theme();
    let content = if props.required {
        format!("{} *", props.content)
    } else {
        props.content.clone()
    };

    rsx! {
        text {
            content,
            width: "100%",
            margin_bottom: spacing::SM,
            font_size: typography::SM,
            font_weight: 500_i32,
            font_color: if props.invalid { theme.colors.destructive } else { theme.colors.foreground },
            line_height: 20.0,
            text_align: "start",
        }
    }
}

/// Props for [`FieldTitle`].
#[derive(Props, Clone, PartialEq)]
pub struct FieldTitleProps {
    pub content: String,
}

/// A compact title used inside horizontal fields.
#[component]
pub fn FieldTitle(props: FieldTitleProps) -> Element {
    let theme = use_theme();
    rsx! {
        text {
            content: props.content.clone(),
            width: "100%",
            font_size: typography::SM,
            font_weight: 500_i32,
            font_color: theme.colors.foreground,
            line_height: 20.0,
            text_align: "start",
        }
    }
}

/// Props for [`FieldDescription`].
#[derive(Props, Clone, PartialEq)]
pub struct FieldDescriptionProps {
    pub content: String,
    /// Adds compact spacing above the description when it follows a control.
    #[props(default = true)]
    pub inset: bool,
}

/// Muted supporting text for a field or field set.
#[component]
pub fn FieldDescription(props: FieldDescriptionProps) -> Element {
    let theme = use_theme();
    rsx! {
        text {
            content: props.content.clone(),
            width: "100%",
            margin_top: if props.inset { spacing::XS } else { 0.0 },
            font_size: typography::XS,
            font_color: theme.colors.muted_foreground,
            line_height: 18.0,
            text_align: "start",
        }
    }
}

/// Props for [`FieldError`].
#[derive(Props, Clone, PartialEq)]
pub struct FieldErrorProps {
    #[props(default)]
    pub message: Option<String>,
    /// Multiple validation messages are rendered as a compact list.
    #[props(default)]
    pub errors: Vec<String>,
}

/// Destructive validation feedback for a field.
#[component]
pub fn FieldError(props: FieldErrorProps) -> Element {
    let theme = use_theme();
    let mut messages = props.errors.clone();
    if let Some(message) = props.message.as_ref().filter(|message| !message.is_empty()) {
        messages.insert(0, message.clone());
    }

    rsx! {
        if !messages.is_empty() {
            column {
                width: "100%",
                align_items: "start",
                margin_top: spacing::XS,
                for (index, message) in messages.iter().enumerate() {
                    text {
                        content: if messages.len() > 1 { format!("• {message}") } else { message.clone() },
                        width: "100%",
                        margin_top: if index == 0 { 0.0 } else { spacing::XXS },
                        font_size: typography::XS,
                        font_weight: 500_i32,
                        font_color: theme.colors.destructive,
                        line_height: 18.0,
                        text_align: "start",
                    }
                }
            }
        }
    }
}

/// Props for [`FieldGroup`].
#[derive(Props, Clone, PartialEq)]
pub struct FieldGroupProps {
    pub children: Element,
}

/// Groups related fields into one full-width stack.
#[component]
pub fn FieldGroup(props: FieldGroupProps) -> Element {
    rsx! {
        column {
            width: "100%",
            align_items: "start",
            {props.children}
        }
    }
}

/// Props for [`FieldSet`].
#[derive(Props, Clone, PartialEq)]
pub struct FieldSetProps {
    pub children: Element,
}

/// A semantic visual group for a legend, description, and related fields.
#[component]
pub fn FieldSet(props: FieldSetProps) -> Element {
    rsx! {
        column {
            width: "100%",
            align_items: "start",
            {props.children}
        }
    }
}

/// Visual treatment for [`FieldLegend`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FieldLegendVariant {
    /// Section heading treatment.
    #[default]
    Legend,
    /// Compact label treatment for nested groups.
    Label,
}

/// Props for [`FieldLegend`].
#[derive(Props, Clone, PartialEq)]
pub struct FieldLegendProps {
    pub content: String,
    #[props(default)]
    pub variant: FieldLegendVariant,
}

/// Heading for a field set.
#[component]
pub fn FieldLegend(props: FieldLegendProps) -> Element {
    let theme = use_theme();
    let (font_size, font_weight, line_height) = match props.variant {
        FieldLegendVariant::Legend => (typography::LG, 600_i32, 24.0),
        FieldLegendVariant::Label => (typography::SM, 500_i32, 20.0),
    };

    rsx! {
        text {
            content: props.content.clone(),
            width: "100%",
            margin_bottom: spacing::XS,
            font_size,
            font_weight,
            font_color: theme.colors.foreground,
            line_height,
            text_align: "start",
        }
    }
}

/// Props for [`FieldSeparator`].
#[derive(Props, Clone, PartialEq)]
pub struct FieldSeparatorProps {
    #[props(default)]
    pub label: Option<String>,
}

/// A field-group divider with an optional centered label.
#[component]
pub fn FieldSeparator(props: FieldSeparatorProps) -> Element {
    let theme = use_theme();
    let label = props.label.clone().filter(|label| !label.is_empty());

    rsx! {
        row {
            width: "100%",
            align_items: "center",
            margin_top: spacing::XXS,
            margin_bottom: spacing::XL,
            row {
                layout_weight: 1.0,
                height: 1.0,
                background_color: theme.colors.border,
            }
            if let Some(label) = label {
                text {
                    content: label,
                    margin_right: spacing::MD,
                    margin_left: spacing::MD,
                    font_size: typography::XS,
                    font_color: theme.colors.muted_foreground,
                    line_height: 18.0,
                }
                row {
                    layout_weight: 1.0,
                    height: 1.0,
                    background_color: theme.colors.border,
                }
            }
        }
    }
}

/// Props for [`FormItem`].
#[derive(Props, Clone, PartialEq)]
pub struct FormItemProps {
    pub label: String,
    #[props(default)]
    pub description: Option<String>,
    #[props(default)]
    pub error: Option<String>,
    #[props(default)]
    pub required: bool,
    #[props(default)]
    pub disabled: bool,
    pub children: Element,
}

/// Backwards-compatible labelled field convenience wrapper.
///
/// New code can use the `Field*` primitives directly when it needs a custom
/// composition. `FormItem` covers the common label-control-description-error
/// stack and keeps the original `label + children` call shape intact.
#[component]
pub fn FormItem(props: FormItemProps) -> Element {
    let invalid = props.error.as_ref().is_some_and(|error| !error.is_empty());

    rsx! {
        Field {
            invalid,
            disabled: props.disabled,
            FieldLabel {
                content: props.label.clone(),
                required: props.required,
                invalid,
            }
            {props.children}
            if let Some(description) = props.description.as_ref().filter(|description| !description.is_empty()) {
                FieldDescription { content: description.clone() }
            }
            if let Some(error) = props.error.as_ref().filter(|error| !error.is_empty()) {
                FieldError { message: Some(error.clone()) }
            }
        }
    }
}
