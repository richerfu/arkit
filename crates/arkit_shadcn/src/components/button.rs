use super::*;
use arkit::ohos_arkui_binding::arkui_input_binding::UIInputAction;
use arkit::ohos_arkui_binding::common::attribute::{
    ArkUINodeAttributeItem, ArkUINodeAttributeNumber,
};
use arkit::ohos_arkui_binding::common::node::ArkUINode;
use arkit::ohos_arkui_binding::component::attribute::{
    ArkUIAttributeBasic, ArkUICommonAttribute, ArkUICommonFontAttribute,
};
use arkit::ohos_arkui_binding::types::text_alignment::TextAlignment;
use arkit_icon as lucide;
use std::cell::RefCell;
use std::rc::Rc;

const TRANSPARENT: u32 = 0x00000000;
const WHITE: u32 = 0xFFFFFFFF;
const FONT_WEIGHT_MEDIUM: i32 = 4;
const FLEX_ALIGN_CENTER: i32 = 2;
const SHADOW_OUTER_DEFAULT_SM: i32 = 1;
const TEXT_DECORATION_NONE: i32 = 0;
const TEXT_DECORATION_UNDERLINE: i32 = 1;
const TEXT_DECORATION_STYLE_SOLID: i32 = 0;

fn edge_all(value: f32) -> Vec<f32> {
    vec![value, value, value, value]
}

fn color_all(value: u32) -> Vec<u32> {
    vec![value, value, value, value]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonVariant {
    Default,
    Secondary,
    Outline,
    Ghost,
    Destructive,
    Link,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonSize {
    #[default]
    Default,
    Sm,
    Lg,
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

#[derive(Debug, Clone, Copy)]
struct ButtonInteractionStyle {
    background: u32,
    foreground: u32,
    border_width: f32,
    border_color: u32,
    opacity: f32,
    text_decoration: i32,
}

struct RuntimeButtonNode(ArkUINode);
impl ArkUIAttributeBasic for RuntimeButtonNode {
    fn raw(&self) -> &ArkUINode {
        &self.0
    }

    fn borrow_mut(&mut self) -> &mut ArkUINode {
        &mut self.0
    }
}

impl ArkUICommonAttribute for RuntimeButtonNode {}
impl ArkUICommonFontAttribute for RuntimeButtonNode {}

pub fn button(label: impl Into<String>, variant: ButtonVariant) -> ButtonElement {
    button_with_options(label, variant, ButtonSize::Default, false)
}

pub fn button_with_size(
    label: impl Into<String>,
    variant: ButtonVariant,
    size: ButtonSize,
) -> ButtonElement {
    button_with_options(label, variant, size, false)
}

pub fn disabled_button(label: impl Into<String>, variant: ButtonVariant) -> ButtonElement {
    button_with_options(label, variant, ButtonSize::Default, true)
}

pub fn button_with_options(
    label: impl Into<String>,
    variant: ButtonVariant,
    size: ButtonSize,
    disabled: bool,
) -> ButtonElement {
    let label = label.into();
    let size_style = size_style(size);
    let variant_style = variant_style(variant);
    let pressed_style = pressed_style(variant);
    let initial_text_decoration = TEXT_DECORATION_NONE;
    let button = button_surface(arkit::button(label), size_style)
        .background_color(variant_style.background)
        .font_size(size_style.text_size)
        .style(ArkUINodeAttributeType::FontWeight, FONT_WEIGHT_MEDIUM)
        .style(ArkUINodeAttributeType::FontColor, variant_style.foreground)
        .style(
            ArkUINodeAttributeType::TextAlign,
            i32::from(TextAlignment::Center),
        )
        .style(
            ArkUINodeAttributeType::TextDecoration,
            text_decoration(initial_text_decoration, variant_style.foreground),
        );

    finalize_button(button, size_style, variant_style, pressed_style, disabled)
}

pub fn button_with_icon(
    label: impl Into<String>,
    icon_name: impl Into<String>,
    variant: ButtonVariant,
) -> ButtonElement {
    button_with_icon_size(label, icon_name, variant, ButtonSize::Default)
}

pub fn button_with_icon_size(
    label: impl Into<String>,
    icon_name: impl Into<String>,
    variant: ButtonVariant,
    size: ButtonSize,
) -> ButtonElement {
    let label = label.into();
    let icon_name = icon_name.into();
    let size_style = size_style(size);
    let variant_style = variant_style(variant);
    let pressed_style = pressed_style(variant);

    let button = button_surface(arkit::button_component(), size_style)
        .background_color(variant_style.background)
        .children(vec![button_content_row(
            Some(label),
            Some(icon_name),
            variant_style.foreground,
            icon_size(size),
        )]);

    finalize_button(button, size_style, variant_style, pressed_style, false)
}

pub fn icon_button(icon: impl Into<String>) -> ButtonElement {
    icon_button_with_variant(icon, ButtonVariant::Outline)
}

pub fn icon_button_with_variant(icon: impl Into<String>, variant: ButtonVariant) -> ButtonElement {
    let size_style = size_style(ButtonSize::Icon);
    let variant_style = variant_style(variant);
    let pressed_style = pressed_style(variant);

    let button = button_surface(arkit::button_component(), size_style)
        .background_color(variant_style.background)
        .children(vec![button_content_row(
            None,
            Some(icon.into()),
            variant_style.foreground,
            icon_size(ButtonSize::Icon),
        )]);

    finalize_button(button, size_style, variant_style, pressed_style, false)
}

fn button_surface(element: ButtonElement, size_style: ButtonSizeStyle) -> ButtonElement {
    let mut button = element
        // Follow the official ArkUI guidance: NORMAL + BorderRadius.
        .style(ArkUINodeAttributeType::ButtonType, 0_i32)
        .style(ArkUINodeAttributeType::Clip, true)
        .height(size_style.height)
        .style(ArkUINodeAttributeType::Padding, size_style.padding.to_vec())
        .style(ArkUINodeAttributeType::BorderStyle, 0_i32)
        .style(ArkUINodeAttributeType::BorderRadius, edge_all(radius::MD))
        .style(ArkUINodeAttributeType::BorderWidth, edge_all(0.0))
        .style(ArkUINodeAttributeType::BorderColor, color_all(TRANSPARENT))
        .style(ArkUINodeAttributeType::AlignSelf, 1_i32);

    if let Some(width) = size_style.width {
        button = button.width(width);
    }

    button
}

fn size_style(size: ButtonSize) -> ButtonSizeStyle {
    match size {
        ButtonSize::Default => ButtonSizeStyle {
            height: 40.0,
            width: None,
            padding: [8.0, 16.0, 8.0, 16.0],
            text_size: typography::SM,
        },
        ButtonSize::Sm => ButtonSizeStyle {
            height: 36.0,
            width: None,
            padding: [0.0, 12.0, 0.0, 12.0],
            text_size: typography::SM,
        },
        ButtonSize::Lg => ButtonSizeStyle {
            height: 44.0,
            width: None,
            padding: [0.0, 24.0, 0.0, 24.0],
            text_size: typography::SM,
        },
        ButtonSize::Icon => ButtonSizeStyle {
            height: 40.0,
            width: Some(40.0),
            padding: [0.0, 0.0, 0.0, 0.0],
            text_size: typography::MD,
        },
    }
}

fn variant_style(variant: ButtonVariant) -> ButtonVariantStyle {
    match variant {
        ButtonVariant::Default => ButtonVariantStyle {
            background: color::PRIMARY,
            foreground: color::PRIMARY_FOREGROUND,
            border_width: 0.0,
            border_color: TRANSPARENT,
            shadow: true,
        },
        ButtonVariant::Secondary => ButtonVariantStyle {
            background: color::SECONDARY,
            foreground: color::SECONDARY_FOREGROUND,
            border_width: 0.0,
            border_color: TRANSPARENT,
            shadow: true,
        },
        ButtonVariant::Outline => ButtonVariantStyle {
            background: color::BACKGROUND,
            foreground: color::FOREGROUND,
            border_width: 1.0,
            border_color: color::BORDER,
            shadow: true,
        },
        ButtonVariant::Ghost => ButtonVariantStyle {
            background: TRANSPARENT,
            foreground: color::FOREGROUND,
            border_width: 0.0,
            border_color: TRANSPARENT,
            shadow: false,
        },
        ButtonVariant::Destructive => ButtonVariantStyle {
            background: color::DESTRUCTIVE,
            foreground: WHITE,
            border_width: 0.0,
            border_color: TRANSPARENT,
            shadow: true,
        },
        ButtonVariant::Link => ButtonVariantStyle {
            background: TRANSPARENT,
            foreground: color::PRIMARY,
            border_width: 0.0,
            border_color: TRANSPARENT,
            shadow: false,
        },
    }
}

fn pressed_style(variant: ButtonVariant) -> Option<ButtonInteractionStyle> {
    match variant {
        ButtonVariant::Default => Some(ButtonInteractionStyle {
            background: alpha_blend(color::PRIMARY, color::BACKGROUND, 0.9),
            foreground: color::PRIMARY_FOREGROUND,
            border_width: 0.0,
            border_color: TRANSPARENT,
            opacity: 1.0,
            text_decoration: TEXT_DECORATION_NONE,
        }),
        ButtonVariant::Secondary => Some(ButtonInteractionStyle {
            background: alpha_blend(color::SECONDARY, color::BACKGROUND, 0.8),
            foreground: color::SECONDARY_FOREGROUND,
            border_width: 0.0,
            border_color: TRANSPARENT,
            opacity: 1.0,
            text_decoration: TEXT_DECORATION_NONE,
        }),
        ButtonVariant::Outline => Some(ButtonInteractionStyle {
            background: color::ACCENT,
            foreground: color::ACCENT_FOREGROUND,
            border_width: 1.0,
            border_color: color::BORDER,
            opacity: 1.0,
            text_decoration: TEXT_DECORATION_NONE,
        }),
        ButtonVariant::Ghost => Some(ButtonInteractionStyle {
            background: color::ACCENT,
            foreground: color::ACCENT_FOREGROUND,
            border_width: 0.0,
            border_color: TRANSPARENT,
            opacity: 1.0,
            text_decoration: TEXT_DECORATION_NONE,
        }),
        ButtonVariant::Destructive => Some(ButtonInteractionStyle {
            background: alpha_blend(color::DESTRUCTIVE, color::BACKGROUND, 0.9),
            foreground: WHITE,
            border_width: 0.0,
            border_color: TRANSPARENT,
            opacity: 1.0,
            text_decoration: TEXT_DECORATION_NONE,
        }),
        ButtonVariant::Link => Some(ButtonInteractionStyle {
            background: TRANSPARENT,
            foreground: color::PRIMARY,
            border_width: 0.0,
            border_color: TRANSPARENT,
            opacity: 1.0,
            text_decoration: TEXT_DECORATION_UNDERLINE,
        }),
    }
}

fn interaction_style(style: ButtonVariantStyle, opacity: f32) -> ButtonInteractionStyle {
    ButtonInteractionStyle {
        background: style.background,
        foreground: style.foreground,
        border_width: style.border_width,
        border_color: style.border_color,
        opacity,
        text_decoration: TEXT_DECORATION_NONE,
    }
}

fn finalize_button(
    mut button: ButtonElement,
    size_style: ButtonSizeStyle,
    variant_style: ButtonVariantStyle,
    pressed_style: Option<ButtonInteractionStyle>,
    disabled: bool,
) -> ButtonElement {
    let runtime_node = Rc::new(RefCell::new(None::<RuntimeButtonNode>));
    let capture_node = runtime_node.clone();
    let normal_style = ButtonInteractionStyle {
        text_decoration: TEXT_DECORATION_NONE,
        ..interaction_style(variant_style, if disabled { 0.5 } else { 1.0 })
    };

    button = button.native(move |node| {
        capture_node.replace(Some(RuntimeButtonNode(node.borrow_mut().clone())));
        Ok(())
    });

    let mount_node = runtime_node.clone();
    arkit::queue_after_mount(move || {
        apply_interaction_style_if_ready(&mount_node, normal_style);
        queue_interaction_style_passes(mount_node.clone(), normal_style, 4);
    });


    button = apply_border_style(
        button,
        variant_style.border_width,
        variant_style.border_color,
    );

    let attach_node = runtime_node.clone();
    button = button.on_event_no_param(arkit::prelude::NodeEventType::EventOnAttach, move || {
        queue_interaction_style_passes(attach_node.clone(), normal_style, 2);
    });

    let appear_node = runtime_node.clone();
    button = button.on_event_no_param(arkit::prelude::NodeEventType::EventOnAppear, move || {
        queue_interaction_style_passes(appear_node.clone(), normal_style, 2);
    });

    let detach_node = runtime_node.clone();
    button = button.on_event_no_param(arkit::prelude::NodeEventType::EventOnDetach, move || {
        detach_node.borrow_mut().take();
    });

    let size_change_node = runtime_node.clone();
    button = button.on_event(arkit::prelude::NodeEventType::OnSizeChange, move |_| {
        queue_interaction_style_passes(size_change_node.clone(), normal_style, 2);
    });

    let area_change_node = runtime_node.clone();
    button = button.on_event(
        arkit::prelude::NodeEventType::EventOnAreaChange,
        move |_| {
            queue_interaction_style_passes(area_change_node.clone(), normal_style, 2);
        },
    );

    if !disabled {
        if let Some(pressed_style) = pressed_style {
            let touch_node = runtime_node.clone();
            button = button.on_event(arkit::prelude::NodeEventType::TouchEvent, move |event| {
                let Some(input_event) = event.input_event() else {
                    return;
                };
                let button_binding = touch_node.borrow();
                let Some(node) = button_binding.as_ref() else {
                    return;
                };
                match input_event.action {
                    UIInputAction::Down => apply_interaction_style(node, pressed_style),
                    UIInputAction::Up | UIInputAction::Cancel => {
                        apply_interaction_style(node, normal_style)
                    }
                    UIInputAction::Move => {}
                }
            });
        }
    }

    button = if variant_style.shadow {
        subtle_button_shadow(button)
    } else {
        clear_button_shadow(button)
    };

    let button = if disabled {
        button
            .style(ArkUINodeAttributeType::Enabled, false)
            .style(ArkUINodeAttributeType::Opacity, 0.5_f32)
    } else {
        button
            .style(ArkUINodeAttributeType::Enabled, true)
            .style(ArkUINodeAttributeType::Opacity, 1.0_f32)
    };

    // Keep icon/text content vertically centered when the button renders child nodes.
    button.height(size_style.height)
}

fn button_content_row(
    label: Option<String>,
    icon_name: Option<String>,
    foreground: u32,
    icon_size: f32,
) -> Element {
    let mut children = Vec::new();

    if let Some(icon_name) = icon_name {
        children.push(
            lucide::icon(icon_name)
                .size(icon_size)
                .color(foreground)
                .render(),
        );
    }

    if let Some(label) = label {
        let text = arkit::text(label)
            .font_size(typography::SM)
            .style(ArkUINodeAttributeType::FontColor, foreground)
            .style(ArkUINodeAttributeType::FontWeight, FONT_WEIGHT_MEDIUM)
            .style(ArkUINodeAttributeType::TextLineHeight, 20.0)
            .into();

        if children.is_empty() {
            children.push(text);
        } else {
            children.push(
                arkit::row_component()
                    .style(ArkUINodeAttributeType::Margin, vec![0.0, 0.0, 0.0, 8.0])
                    .children(vec![text])
                    .into(),
            );
        }
    }

    arkit::row_component()
        .style(ArkUINodeAttributeType::RowAlignItems, FLEX_ALIGN_CENTER)
        .style(ArkUINodeAttributeType::RowJustifyContent, FLEX_ALIGN_CENTER)
        .children(children)
        .into()
}

fn icon_size(size: ButtonSize) -> f32 {
    match size {
        ButtonSize::Icon => 16.0,
        ButtonSize::Sm => 15.0,
        ButtonSize::Lg => 18.0,
        ButtonSize::Default => 16.0,
    }
}

fn apply_interaction_style_if_ready(
    runtime_node: &Rc<RefCell<Option<RuntimeButtonNode>>>,
    style: ButtonInteractionStyle,
) {
    let button_binding = runtime_node.borrow();
    let Some(node) = button_binding.as_ref() else {
        return;
    };
    apply_interaction_style(node, style);
}

fn queue_interaction_style_passes(
    runtime_node: Rc<RefCell<Option<RuntimeButtonNode>>>,
    style: ButtonInteractionStyle,
    remaining: usize,
) {
    if remaining == 0 {
        return;
    }

    arkit::queue_ui_loop(move || {
        apply_interaction_style_if_ready(&runtime_node, style);
        queue_interaction_style_passes(runtime_node.clone(), style, remaining - 1);
    });
}

fn apply_interaction_style(node: &RuntimeButtonNode, style: ButtonInteractionStyle) {
    let _ = node.set_attribute(
        ArkUINodeAttributeType::ButtonType,
        ArkUINodeAttributeItem::from(0_i32),
    );
    let _ = node.set_attribute(
        ArkUINodeAttributeType::Clip,
        ArkUINodeAttributeItem::from(true),
    );
    let _ = node.set_attribute(
        ArkUINodeAttributeType::BorderStyle,
        ArkUINodeAttributeItem::from(0_i32),
    );
    let _ = node.background_color(style.background);
    let _ = node.set_attribute(
        ArkUINodeAttributeType::BorderWidth,
        ArkUINodeAttributeItem::from(edge_all(style.border_width)),
    );
    let _ = node.set_attribute(
        ArkUINodeAttributeType::BorderColor,
        ArkUINodeAttributeItem::from(color_all(style.border_color)),
    );
    let _ = node.set_attribute(
        ArkUINodeAttributeType::BorderRadius,
        ArkUINodeAttributeItem::from(edge_all(radius::MD)),
    );
    let _ = node.opacity(style.opacity);
    let _ = node.font_color(style.foreground);
    let _ = node.set_attribute(
        ArkUINodeAttributeType::TextDecoration,
        text_decoration(style.text_decoration, style.foreground),
    );
}

fn alpha_blend(foreground: u32, background: u32, alpha: f32) -> u32 {
    let alpha = alpha.clamp(0.0, 1.0);
    let [_, fg_r, fg_g, fg_b] = foreground.to_be_bytes();
    let [_, bg_r, bg_g, bg_b] = background.to_be_bytes();

    let mix = |fg: u8, bg: u8| -> u8 {
        ((fg as f32 * alpha) + (bg as f32 * (1.0 - alpha))).round() as u8
    };

    u32::from_be_bytes([0xFF, mix(fg_r, bg_r), mix(fg_g, bg_g), mix(fg_b, bg_b)])
}

fn subtle_button_shadow(element: ButtonElement) -> ButtonElement {
    element.style(
        ArkUINodeAttributeType::Shadow,
        vec![SHADOW_OUTER_DEFAULT_SM],
    )
}

fn clear_button_shadow(element: ButtonElement) -> ButtonElement {
    element
}

fn apply_border_style(
    element: ButtonElement,
    border_width: f32,
    border_color: u32,
) -> ButtonElement {
    element
        .style(ArkUINodeAttributeType::BorderStyle, 0_i32)
        .style(ArkUINodeAttributeType::BorderWidth, edge_all(border_width))
        .style(ArkUINodeAttributeType::BorderColor, color_all(border_color))
        .style(ArkUINodeAttributeType::BorderRadius, edge_all(radius::MD))
}

fn text_decoration(decoration_type: i32, color_value: u32) -> ArkUINodeAttributeItem {
    ArkUINodeAttributeItem::NumberValue(vec![
        ArkUINodeAttributeNumber::Int(decoration_type),
        ArkUINodeAttributeNumber::Uint(color_value),
        ArkUINodeAttributeNumber::Int(TEXT_DECORATION_STYLE_SOLID),
    ])
}
