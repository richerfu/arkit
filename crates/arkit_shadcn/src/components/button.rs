use super::*;
use arkit::ohos_arkui_binding::types::alignment::Alignment;
use arkit::ohos_arkui_binding::common::attribute::{
    ArkUINodeAttributeItem, ArkUINodeAttributeNumber,
};
use arkit::ohos_arkui_binding::common::node::ArkUINode;
use arkit::ohos_arkui_binding::component::attribute::{
    ArkUIAttributeBasic, ArkUICommonAttribute, ArkUICommonFontAttribute,
};
use arkit::prelude::NodeEventType;
use arkit::ohos_arkui_binding::types::text_alignment::TextAlignment;
use arkit_icon as lucide;
use std::cell::RefCell;
use std::rc::Rc;

const TRANSPARENT: u32 = 0x00000000;
const WHITE: u32 = 0xFFFFFFFF;
const BUTTON_TYPE_NORMAL: i32 = 0;
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

#[derive(Default)]
struct RuntimeButtonContentNodes {
    texts: Vec<ArkUINode>,
    images: Vec<ArkUINode>,
}

fn with_runtime_button_content(
    runtime_content: &Option<Rc<RefCell<Option<RuntimeButtonContentNodes>>>>,
    apply: impl FnOnce(&RuntimeButtonContentNodes),
) {
    let Some(runtime_content) = runtime_content.as_ref() else {
        return;
    };
    let borrow = runtime_content.borrow();
    let Some(content) = borrow.as_ref() else {
        return;
    };
    apply(content);
}

fn collect_button_content_nodes(node: &ArkUINode, content: &mut RuntimeButtonContentNodes) {
    for child in node.children() {
        let child = child.borrow();
        if child
            .get_attribute(ArkUINodeAttributeType::TextContent)
            .is_ok()
        {
            content.texts.push(child.clone());
        }
        if child.get_attribute(ArkUINodeAttributeType::ImageAlt).is_ok() {
            content.images.push(child.clone());
        }
        collect_button_content_nodes(&child, content);
    }
}

fn apply_content_interaction_style(
    content: &RuntimeButtonContentNodes,
    style: ButtonInteractionStyle,
) {
    for text in &content.texts {
        let _ = text.set_attribute(
            ArkUINodeAttributeType::FontColor,
            ArkUINodeAttributeItem::from(style.foreground),
        );
    }

    for image in &content.images {
        let _ = image.set_attribute(
            ArkUINodeAttributeType::ColorBlend,
            ArkUINodeAttributeItem::from(style.foreground),
        );
        let _ = image.set_attribute(
            ArkUINodeAttributeType::ForegroundColor,
            ArkUINodeAttributeItem::from(style.foreground),
        );
    }
}

fn restore_content_interaction_style(
    content: &RuntimeButtonContentNodes,
    normal_foreground: u32,
) {
    for text in &content.texts {
        let _ = text.set_attribute(
            ArkUINodeAttributeType::FontColor,
            ArkUINodeAttributeItem::from(normal_foreground),
        );
    }

    for image in &content.images {
        let _ = image.reset_attribute(ArkUINodeAttributeType::ColorBlend);
        let _ = image.reset_attribute(ArkUINodeAttributeType::ForegroundColor);
    }
}

pub fn button<Message: Send + 'static>(
    label: impl Into<String>,
    variant: ButtonVariant,
) -> ButtonElement<Message> {
    button_with_options(label, variant, ButtonSize::Default, false)
}

pub fn button_with_size<Message: Send + 'static>(
    label: impl Into<String>,
    variant: ButtonVariant,
    size: ButtonSize,
) -> ButtonElement<Message> {
    button_with_options(label, variant, size, false)
}

pub fn disabled_button<Message: Send + 'static>(
    label: impl Into<String>,
    variant: ButtonVariant,
) -> ButtonElement<Message> {
    button_with_options(label, variant, ButtonSize::Default, true)
}

pub fn button_with_options<Message: Send + 'static>(
    label: impl Into<String>,
    variant: ButtonVariant,
    size: ButtonSize,
    disabled: bool,
) -> ButtonElement<Message> {
    let label = label.into();
    let size_style = size_style(size);
    let variant_style = variant_style(variant);
    let pressed_style = pressed_style(variant);
    let button = if matches!(variant, ButtonVariant::Link) {
        let initial_text_decoration = TEXT_DECORATION_NONE;
        button_surface(normal_button(label), size_style, variant_style)
            .patch_font_size(size_style.text_size)
            .patch_attr(ArkUINodeAttributeType::FontWeight, FONT_WEIGHT_MEDIUM)
            .patch_attr(ArkUINodeAttributeType::FontColor, variant_style.foreground)
            .patch_attr(
                ArkUINodeAttributeType::TextAlign,
                i32::from(TextAlignment::Center),
            )
            .patch_attr(
                ArkUINodeAttributeType::TextDecoration,
                text_decoration(initial_text_decoration, variant_style.foreground),
            )
    } else {
        let initial_text_decoration = TEXT_DECORATION_NONE;
        button_surface(normal_button(label), size_style, variant_style)
            .patch_font_size(size_style.text_size)
            .patch_attr(ArkUINodeAttributeType::FontWeight, FONT_WEIGHT_MEDIUM)
            .patch_attr(ArkUINodeAttributeType::FontColor, variant_style.foreground)
            .patch_attr(
                ArkUINodeAttributeType::TextAlign,
                i32::from(TextAlignment::Center),
            )
            .patch_attr(
                ArkUINodeAttributeType::TextDecoration,
                text_decoration(initial_text_decoration, variant_style.foreground),
            )
    };

    finalize_button(button, size_style, variant_style, pressed_style, disabled, None)
}

pub fn button_with_icon<Message: Send + 'static>(
    label: impl Into<String>,
    icon_name: impl Into<String>,
    variant: ButtonVariant,
) -> ButtonElement<Message> {
    button_with_icon_size(label, icon_name, variant, ButtonSize::Default)
}

pub fn button_with_icon_size<Message: Send + 'static>(
    label: impl Into<String>,
    icon_name: impl Into<String>,
    variant: ButtonVariant,
    size: ButtonSize,
) -> ButtonElement<Message> {
    let label = label.into();
    let icon_name = icon_name.into();
    let size_style = size_style(size);
    let variant_style = variant_style(variant);
    let pressed_style = pressed_style(variant);
    let runtime_content = Rc::new(RefCell::new(None::<RuntimeButtonContentNodes>));
    let button = button_surface(normal_button_component(), size_style, variant_style).children(
        vec![button_content_row(
            Some(label),
            Some(icon_name),
            variant_style.foreground,
            icon_size(size),
            runtime_content.clone(),
        )],
    );

    finalize_button(button, size_style, variant_style, pressed_style, false, Some(runtime_content))
}

pub fn icon_button<Message: Send + 'static>(icon: impl Into<String>) -> ButtonElement<Message> {
    icon_button_with_variant(icon, ButtonVariant::Outline)
}

pub fn icon_button_with_variant<Message: Send + 'static>(
    icon: impl Into<String>,
    variant: ButtonVariant,
) -> ButtonElement<Message> {
    let size_style = size_style(ButtonSize::Icon);
    let variant_style = variant_style(variant);
    let pressed_style = pressed_style(variant);
    let runtime_content = Rc::new(RefCell::new(None::<RuntimeButtonContentNodes>));
    let button = button_surface(normal_button_component(), size_style, variant_style).children(
        vec![button_content_row(
            None,
            Some(icon.into()),
            variant_style.foreground,
            icon_size(ButtonSize::Icon),
            runtime_content.clone(),
        )],
    );

    finalize_button(button, size_style, variant_style, pressed_style, false, Some(runtime_content))
}

pub fn normal_button_component<Message, AppTheme>() -> ButtonElement<Message, AppTheme> {
    arkit::button_component().style(
        ArkUINodeAttributeType::ButtonType,
        BUTTON_TYPE_NORMAL,
    )
}

pub fn normal_button<Message, AppTheme>(
    label: impl Into<String>,
) -> ButtonElement<Message, AppTheme> {
    normal_button_component().label(label)
}

fn button_host<Message, AppTheme>(
    element: ButtonElement<Message, AppTheme>,
    size_style: ButtonSizeStyle,
) -> ButtonElement<Message, AppTheme> {
    let mut button = element
        .style(ArkUINodeAttributeType::Focusable, false)
        .style(ArkUINodeAttributeType::FocusOnTouch, false)
        .background_color(TRANSPARENT)
        .style(ArkUINodeAttributeType::Clip, true)
        .style(ArkUINodeAttributeType::BorderStyle, 0_i32)
        .style(
            ArkUINodeAttributeType::Alignment,
            i32::from(Alignment::Center),
        )
        .style(ArkUINodeAttributeType::BorderRadius, edge_all(radius::MD))
        .style(ArkUINodeAttributeType::Padding, vec![0.0, 0.0, 0.0, 0.0])
        .height(size_style.height)
        .style(ArkUINodeAttributeType::BorderWidth, edge_all(0.0))
        .style(ArkUINodeAttributeType::BorderColor, color_all(TRANSPARENT))
        .style(ArkUINodeAttributeType::AlignSelf, 1_i32);

    if let Some(width) = size_style.width {
        button = button.width(width);
    }

    button
}

fn button_surface<Message, AppTheme>(
    element: ButtonElement<Message, AppTheme>,
    size_style: ButtonSizeStyle,
    variant_style: ButtonVariantStyle,
) -> ButtonElement<Message, AppTheme> {
    button_host(element, size_style)
        .style(ArkUINodeAttributeType::Clip, true)
        .style(ArkUINodeAttributeType::BorderRadius, edge_all(radius::MD))
        .style(
            ArkUINodeAttributeType::BorderWidth,
            edge_all(variant_style.border_width),
        )
        .style(
            ArkUINodeAttributeType::BorderColor,
            color_all(variant_style.border_color),
        )
        .style(ArkUINodeAttributeType::Padding, size_style.padding.to_vec())
        .background_color(variant_style.background)
        .patch_attr(
            ArkUINodeAttributeType::BorderWidth,
            edge_all(variant_style.border_width),
        )
        .patch_attr(
            ArkUINodeAttributeType::BorderColor,
            color_all(variant_style.border_color),
        )
        .patch_background_color(variant_style.background)
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
        // bg-primary, shadow-sm
        ButtonVariant::Default => ButtonVariantStyle {
            background: color::PRIMARY,
            foreground: color::PRIMARY_FOREGROUND,
            border_width: 0.0,
            border_color: TRANSPARENT,
            shadow: true,
        },
        // bg-secondary, shadow-sm
        ButtonVariant::Secondary => ButtonVariantStyle {
            background: color::SECONDARY,
            foreground: color::SECONDARY_FOREGROUND,
            border_width: 0.0,
            border_color: TRANSPARENT,
            shadow: true,
        },
        // border-border bg-background, shadow-sm
        ButtonVariant::Outline => ButtonVariantStyle {
            background: color::BACKGROUND,
            foreground: color::FOREGROUND,
            border_width: 1.0,
            border_color: color::BORDER,
            shadow: true,
        },
        // no bg, no shadow
        ButtonVariant::Ghost => ButtonVariantStyle {
            background: TRANSPARENT,
            foreground: color::FOREGROUND,
            border_width: 0.0,
            border_color: TRANSPARENT,
            shadow: false,
        },
        // bg-destructive, shadow-sm
        ButtonVariant::Destructive => ButtonVariantStyle {
            background: color::DESTRUCTIVE,
            foreground: WHITE,
            border_width: 0.0,
            border_color: TRANSPARENT,
            shadow: true,
        },
        // no bg, no shadow
        ButtonVariant::Link => ButtonVariantStyle {
            background: TRANSPARENT,
            foreground: color::PRIMARY,
            border_width: 0.0,
            border_color: TRANSPARENT,
            shadow: false,
        },
    }
}

fn blend_over_background(foreground: u32, background: u32, alpha_percent: u32) -> u32 {
    let alpha = (alpha_percent.min(100) as f32) / 100.0;
    let [_, fg_r, fg_g, fg_b] = foreground.to_be_bytes();
    let [_, bg_r, bg_g, bg_b] = background.to_be_bytes();

    let mix = |fg: u8, bg: u8| -> u8 {
        ((fg as f32 * alpha) + (bg as f32 * (1.0 - alpha))).round() as u8
    };

    u32::from_be_bytes([
        0xFF,
        mix(fg_r, bg_r),
        mix(fg_g, bg_g),
        mix(fg_b, bg_b),
    ])
}

fn pressed_style(variant: ButtonVariant) -> Option<ButtonInteractionStyle> {
    match variant {
        // active:bg-primary/90
        ButtonVariant::Default => Some(ButtonInteractionStyle {
            background: blend_over_background(color::PRIMARY, color::BACKGROUND, 90),
            foreground: color::PRIMARY_FOREGROUND,
            border_width: 0.0,
            border_color: TRANSPARENT,
            opacity: 1.0,
            text_decoration: TEXT_DECORATION_NONE,
        }),
        // active:bg-secondary/80
        ButtonVariant::Secondary => Some(ButtonInteractionStyle {
            background: blend_over_background(color::SECONDARY, color::BACKGROUND, 80),
            foreground: color::SECONDARY_FOREGROUND,
            border_width: 0.0,
            border_color: TRANSPARENT,
            opacity: 1.0,
            text_decoration: TEXT_DECORATION_NONE,
        }),
        // active:bg-accent, group-active:text-accent-foreground
        ButtonVariant::Outline => Some(ButtonInteractionStyle {
            background: color::ACCENT,
            foreground: color::ACCENT_FOREGROUND,
            border_width: 1.0,
            border_color: color::BORDER,
            opacity: 1.0,
            text_decoration: TEXT_DECORATION_NONE,
        }),
        // active:bg-accent, group-active:text-accent-foreground
        ButtonVariant::Ghost => Some(ButtonInteractionStyle {
            background: color::ACCENT,
            foreground: color::ACCENT_FOREGROUND,
            border_width: 0.0,
            border_color: TRANSPARENT,
            opacity: 1.0,
            text_decoration: TEXT_DECORATION_NONE,
        }),
        // active:bg-destructive/90
        ButtonVariant::Destructive => Some(ButtonInteractionStyle {
            background: blend_over_background(color::DESTRUCTIVE, color::BACKGROUND, 90),
            foreground: WHITE,
            border_width: 0.0,
            border_color: TRANSPARENT,
            opacity: 1.0,
            text_decoration: TEXT_DECORATION_NONE,
        }),
        // group-active:underline
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

fn finalize_button<Message: Send + 'static>(
    mut button: ButtonElement<Message>,
    size_style: ButtonSizeStyle,
    variant_style: ButtonVariantStyle,
    pressed_style: Option<ButtonInteractionStyle>,
    disabled: bool,
    runtime_content: Option<Rc<RefCell<Option<RuntimeButtonContentNodes>>>>,
) -> ButtonElement<Message> {
    let runtime_node = Rc::new(RefCell::new(None::<RuntimeButtonNode>));
    let normal_style = ButtonInteractionStyle {
        text_decoration: TEXT_DECORATION_NONE,
        ..interaction_style(variant_style, if disabled { 0.5 } else { 1.0 })
    };

    {
        let runtime_node = runtime_node.clone();
        button = button.with_patch(move |node| {
            let runtime = RuntimeButtonNode(node.clone());
            apply_interaction_style(&runtime, normal_style);
            runtime_node.replace(Some(runtime));
            Ok(())
        });
    }

    {
        let runtime_node = runtime_node.clone();
        button = button.on_event_no_param(NodeEventType::EventOnAreaChange, move || {
            let binding = runtime_node.borrow();
            let Some(node) = binding.as_ref() else {
                return;
            };
            apply_interaction_style(node, normal_style);
        });
    }

    button = button
        .patch_attr(
            ArkUINodeAttributeType::BorderWidth,
            edge_all(variant_style.border_width),
        )
        .patch_attr(
            ArkUINodeAttributeType::BorderColor,
            color_all(variant_style.border_color),
        );

    if !disabled {
        if let Some(pressed_style) = pressed_style {
            let runtime_content = runtime_content.clone();
            button = button.on_supported_ui_states(1, true, move |node, current| {
                let runtime = RuntimeButtonNode(node.clone());
                if current & 1 == 1 {
                    apply_interaction_style(&runtime, pressed_style);
                    with_runtime_button_content(&runtime_content, |content| {
                        apply_content_interaction_style(content, pressed_style);
                    });
                } else {
                    apply_interaction_style(&runtime, normal_style);
                    with_runtime_button_content(&runtime_content, |content| {
                        restore_content_interaction_style(content, normal_style.foreground);
                    });
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
            .patch_attr(ArkUINodeAttributeType::Enabled, false)
            .patch_attr(ArkUINodeAttributeType::Opacity, 0.5_f32)
    } else {
        button
            .patch_attr(ArkUINodeAttributeType::Enabled, true)
            .patch_attr(ArkUINodeAttributeType::Opacity, 1.0_f32)
    };

    button.patch_attr(ArkUINodeAttributeType::Height, size_style.height)
}

fn button_content_row<Message: 'static>(
    label: Option<String>,
    icon_name: Option<String>,
    foreground: u32,
    icon_size: f32,
    runtime_content: Rc<RefCell<Option<RuntimeButtonContentNodes>>>,
) -> Element<Message> {
    let mut children = Vec::new();

    if let Some(icon_name) = icon_name {
        children.push(
            lucide::icon(icon_name)
                .size(icon_size)
                .color(foreground)
                .render::<Message, arkit::Theme>(),
        );
    }

    if let Some(label) = label {
        let text = arkit::text::<Message, arkit::Theme>(label)
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
        .align_items_center()
        .style(ArkUINodeAttributeType::RowJustifyContent, FLEX_ALIGN_CENTER)
        .with_patch(move |node| {
            let mut content = RuntimeButtonContentNodes::default();
            collect_button_content_nodes(node, &mut content);
            runtime_content.replace(Some(content));
            Ok(())
        })
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

fn apply_interaction_style(node: &RuntimeButtonNode, style: ButtonInteractionStyle) {
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

fn subtle_button_shadow<Message, AppTheme>(
    element: ButtonElement<Message, AppTheme>,
) -> ButtonElement<Message, AppTheme> {
    element.patch_attr(
        ArkUINodeAttributeType::Shadow,
        vec![SHADOW_OUTER_DEFAULT_SM],
    )
}

fn clear_button_shadow<Message, AppTheme>(
    element: ButtonElement<Message, AppTheme>,
) -> ButtonElement<Message, AppTheme> {
    element.patch_attr(ArkUINodeAttributeType::Shadow, vec![0_i32])
}

fn text_decoration(decoration_type: i32, color_value: u32) -> ArkUINodeAttributeItem {
    ArkUINodeAttributeItem::NumberValue(vec![
        ArkUINodeAttributeNumber::Int(decoration_type),
        ArkUINodeAttributeNumber::Uint(color_value),
        ArkUINodeAttributeNumber::Int(TEXT_DECORATION_STYLE_SOLID),
    ])
}
