use super::*;
use arkit::ohos_arkui_binding::common::attribute::{
    ArkUINodeAttributeItem, ArkUINodeAttributeNumber,
};
use arkit::ohos_arkui_binding::common::node::ArkUINode;
use arkit::ohos_arkui_binding::component::attribute::{
    ArkUIAttributeBasic, ArkUICommonAttribute, ArkUICommonFontAttribute,
};
use arkit::ohos_arkui_binding::types::alignment::Alignment;
use arkit::prelude::Padding;
use arkit::{ShadowStyle, TextAlignment, UiState};
use arkit_icon as lucide;
use std::cell::Cell;
use std::rc::Rc;

const TRANSPARENT: u32 = 0x00000000;
const WHITE: u32 = 0xFFFFFFFF;
const FONT_WEIGHT_MEDIUM: i32 = 4;
const TEXT_DECORATION_NONE: i32 = 0;
const TEXT_DECORATION_UNDERLINE: i32 = 1;
const TEXT_DECORATION_STYLE_SOLID: i32 = 0;

fn edge_all(value: f32) -> Vec<f32> {
    vec![value, value, value, value]
}

fn color_all(value: u32) -> Vec<u32> {
    vec![value, value, value, value]
}

fn disabled_opacity(disabled: bool) -> f32 {
    if disabled {
        0.5
    } else {
        1.0
    }
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
    node: &RuntimeButtonNode,
    apply: impl FnOnce(&RuntimeButtonContentNodes),
) {
    let mut content = RuntimeButtonContentNodes::default();
    collect_button_content_nodes(&node.0, &mut content);
    apply(&content);
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
        if child
            .get_attribute(ArkUINodeAttributeType::ImageAlt)
            .is_ok()
        {
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

fn restore_content_interaction_style(content: &RuntimeButtonContentNodes, normal_foreground: u32) {
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

fn apply_content_interaction_style_if_changed(
    content: &RuntimeButtonContentNodes,
    style: ButtonInteractionStyle,
    current_foreground: u32,
) {
    if style.foreground != current_foreground {
        apply_content_interaction_style(content, style);
    }
}

fn retheme_button_content<Message: Send + 'static, AppTheme: 'static>(
    element: ButtonElement<Message, AppTheme>,
    foreground: u32,
) -> ButtonElement<Message, AppTheme> {
    element.map_descendants(move |node| {
        if let Some(name) = node
            .attr_string(ArkUINodeAttributeType::ImageAlt)
            .map(str::to_owned)
        {
            let size = node.attr_f32(ArkUINodeAttributeType::Width).unwrap_or(16.0);
            return lucide::icon_node::<Message, AppTheme>(name, size, foreground).unwrap_or(node);
        }

        if node
            .attr_string(ArkUINodeAttributeType::TextContent)
            .is_some()
        {
            node.font_color(foreground)
        } else {
            node
        }
    })
}

pub fn button<Message: Send + 'static>(label: impl Into<String>) -> ButtonElement<Message> {
    apply_button_size(
        button_host(normal_button(label)),
        size_style(ButtonSize::Default),
    )
}

pub fn button_with_icon<Message: Send + 'static>(
    label: impl Into<String>,
    icon_name: impl Into<String>,
) -> ButtonElement<Message> {
    icon_label_button(label, icon_name, ButtonSize::Default)
}

fn icon_label_button<Message: Send + 'static>(
    label: impl Into<String>,
    icon_name: impl Into<String>,
    size: ButtonSize,
) -> ButtonElement<Message> {
    let label = label.into();
    let icon_name = icon_name.into();
    button_host(normal_button_component())
        .children(vec![button_content_row(
            Some(label),
            Some(icon_name),
            color::FOREGROUND,
            icon_size(size),
        )])
        .size(size)
}

pub fn icon_button<Message: Send + 'static>(icon: impl Into<String>) -> ButtonElement<Message> {
    button_host(normal_button_component())
        .children(vec![button_content_row(
            None,
            Some(icon.into()),
            color::FOREGROUND,
            icon_size(ButtonSize::Icon),
        )])
        .size(ButtonSize::Icon)
}

pub fn normal_button_component<Message, AppTheme>() -> ButtonElement<Message, AppTheme> {
    arkit::button_component().attr(
        ArkUINodeAttributeType::ButtonType,
        i32::from(ButtonType::Normal),
    )
}

pub fn normal_button<Message, AppTheme>(
    label: impl Into<String>,
) -> ButtonElement<Message, AppTheme> {
    normal_button_component().label(label)
}

fn button_host<Message, AppTheme>(
    element: ButtonElement<Message, AppTheme>,
) -> ButtonElement<Message, AppTheme> {
    element
        .attr(ArkUINodeAttributeType::Focusable, false)
        .attr(ArkUINodeAttributeType::FocusOnTouch, false)
        .attr(ArkUINodeAttributeType::BackgroundColor, TRANSPARENT)
        .attr(ArkUINodeAttributeType::Clip, true)
        .attr(
            ArkUINodeAttributeType::BorderStyle,
            i32::from(BorderStyle::Solid),
        )
        .attr(
            ArkUINodeAttributeType::Alignment,
            i32::from(Alignment::Center),
        )
        .attr(ArkUINodeAttributeType::BorderRadius, edge_all(radius::MD))
        .attr(
            ArkUINodeAttributeType::Padding,
            vec![
                Padding::ZERO.top,
                Padding::ZERO.right,
                Padding::ZERO.bottom,
                Padding::ZERO.left,
            ],
        )
        .attr(ArkUINodeAttributeType::BorderWidth, edge_all(0.0))
        .attr(ArkUINodeAttributeType::BorderColor, color_all(TRANSPARENT))
        .attr(
            ArkUINodeAttributeType::AlignSelf,
            i32::from(ItemAlignment::Start),
        )
}

fn apply_button_size<Message, AppTheme>(
    element: ButtonElement<Message, AppTheme>,
    size_style: ButtonSizeStyle,
) -> ButtonElement<Message, AppTheme> {
    let mut button = element
        .height(size_style.height)
        .padding(size_style.padding)
        .font_size(size_style.text_size);

    if let Some(width) = size_style.width {
        button = button.width(width);
    }

    button
}

fn resize_button_content<Message: Send + 'static, AppTheme: 'static>(
    element: ButtonElement<Message, AppTheme>,
    size: ButtonSize,
) -> ButtonElement<Message, AppTheme> {
    let size_style = size_style(size);
    let icon_size = icon_size(size);

    element.map_descendants(move |node| {
        if node.attr_string(ArkUINodeAttributeType::ImageAlt).is_some() {
            return node.width(icon_size).height(icon_size);
        }

        if node
            .attr_string(ArkUINodeAttributeType::TextContent)
            .is_some()
        {
            node.font_size(size_style.text_size)
        } else {
            node
        }
    })
}

fn apply_button_theme<Message: Send + 'static, AppTheme: 'static>(
    element: ButtonElement<Message, AppTheme>,
    variant: ButtonVariant,
) -> ButtonElement<Message, AppTheme> {
    let variant_style = variant_style(variant);
    let element = retheme_button_content(element, variant_style.foreground);

    apply_button_theme_with_content(element, variant)
}

fn apply_button_theme_with_content<Message: Send + 'static, AppTheme>(
    element: ButtonElement<Message, AppTheme>,
    variant: ButtonVariant,
) -> ButtonElement<Message, AppTheme> {
    let variant_style = variant_style(variant);
    let pressed_style = pressed_style(variant);
    let initial_text_decoration = TEXT_DECORATION_NONE;

    let button = element
        .attr(ArkUINodeAttributeType::Clip, true)
        .attr(ArkUINodeAttributeType::BorderRadius, edge_all(radius::MD))
        .attr(
            ArkUINodeAttributeType::BorderWidth,
            edge_all(variant_style.border_width),
        )
        .patch_attr(
            ArkUINodeAttributeType::BorderWidth,
            edge_all(variant_style.border_width),
        )
        .attr(
            ArkUINodeAttributeType::BorderColor,
            color_all(variant_style.border_color),
        )
        .patch_attr(
            ArkUINodeAttributeType::BorderColor,
            color_all(variant_style.border_color),
        )
        .attr(
            ArkUINodeAttributeType::BackgroundColor,
            variant_style.background,
        )
        .patch_attr(
            ArkUINodeAttributeType::BackgroundColor,
            variant_style.background,
        )
        .attr(ArkUINodeAttributeType::FontWeight, FONT_WEIGHT_MEDIUM)
        .patch_attr(ArkUINodeAttributeType::FontWeight, FONT_WEIGHT_MEDIUM)
        .attr(ArkUINodeAttributeType::FontColor, variant_style.foreground)
        .patch_attr(ArkUINodeAttributeType::FontColor, variant_style.foreground)
        .attr(
            ArkUINodeAttributeType::TextAlign,
            i32::from(TextAlignment::Center),
        )
        .patch_attr(
            ArkUINodeAttributeType::TextAlign,
            i32::from(TextAlignment::Center),
        )
        .attr(
            ArkUINodeAttributeType::TextDecoration,
            text_decoration(initial_text_decoration, variant_style.foreground),
        )
        .patch_attr(
            ArkUINodeAttributeType::TextDecoration,
            text_decoration(initial_text_decoration, variant_style.foreground),
        );

    finalize_button(button, variant_style, pressed_style)
}

pub trait ButtonStyleExt: Sized {
    fn theme(self, variant: ButtonVariant) -> Self;
    fn size(self, size: ButtonSize) -> Self;
    fn disabled(self, disabled: bool) -> Self;
}

impl<Message: Send + 'static, AppTheme: 'static> ButtonStyleExt
    for ButtonElement<Message, AppTheme>
{
    fn theme(self, variant: ButtonVariant) -> Self {
        apply_button_theme(self, variant)
    }

    fn size(self, size: ButtonSize) -> Self {
        resize_button_content(apply_button_size(self, size_style(size)), size)
    }

    fn disabled(self, disabled: bool) -> Self {
        let opacity = disabled_opacity(disabled);

        self.enabled(!disabled).opacity(opacity)
    }
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

    u32::from_be_bytes([0xFF, mix(fg_r, bg_r), mix(fg_g, bg_g), mix(fg_b, bg_b)])
}

fn pressed_style(variant: ButtonVariant) -> Option<ButtonInteractionStyle> {
    match variant {
        // active:bg-primary/90
        ButtonVariant::Default => Some(ButtonInteractionStyle {
            background: blend_over_background(color::PRIMARY, color::BACKGROUND, 90),
            foreground: color::PRIMARY_FOREGROUND,
            border_width: 0.0,
            border_color: TRANSPARENT,
            text_decoration: TEXT_DECORATION_NONE,
        }),
        // active:bg-secondary/80
        ButtonVariant::Secondary => Some(ButtonInteractionStyle {
            background: blend_over_background(color::SECONDARY, color::BACKGROUND, 80),
            foreground: color::SECONDARY_FOREGROUND,
            border_width: 0.0,
            border_color: TRANSPARENT,
            text_decoration: TEXT_DECORATION_NONE,
        }),
        // active:bg-accent, group-active:text-accent-foreground
        ButtonVariant::Outline => Some(ButtonInteractionStyle {
            background: color::ACCENT,
            foreground: color::ACCENT_FOREGROUND,
            border_width: 1.0,
            border_color: color::BORDER,
            text_decoration: TEXT_DECORATION_NONE,
        }),
        // active:bg-accent, group-active:text-accent-foreground
        ButtonVariant::Ghost => Some(ButtonInteractionStyle {
            background: color::ACCENT,
            foreground: color::ACCENT_FOREGROUND,
            border_width: 0.0,
            border_color: TRANSPARENT,
            text_decoration: TEXT_DECORATION_NONE,
        }),
        // active:bg-destructive/90
        ButtonVariant::Destructive => Some(ButtonInteractionStyle {
            background: blend_over_background(color::DESTRUCTIVE, color::BACKGROUND, 90),
            foreground: WHITE,
            border_width: 0.0,
            border_color: TRANSPARENT,
            text_decoration: TEXT_DECORATION_NONE,
        }),
        // group-active:underline
        ButtonVariant::Link => Some(ButtonInteractionStyle {
            background: TRANSPARENT,
            foreground: color::PRIMARY,
            border_width: 0.0,
            border_color: TRANSPARENT,
            text_decoration: TEXT_DECORATION_UNDERLINE,
        }),
    }
}

fn interaction_style(style: ButtonVariantStyle) -> ButtonInteractionStyle {
    ButtonInteractionStyle {
        background: style.background,
        foreground: style.foreground,
        border_width: style.border_width,
        border_color: style.border_color,
        text_decoration: TEXT_DECORATION_NONE,
    }
}

fn finalize_button<Message: Send + 'static, AppTheme>(
    mut button: ButtonElement<Message, AppTheme>,
    variant_style: ButtonVariantStyle,
    pressed_style: Option<ButtonInteractionStyle>,
) -> ButtonElement<Message, AppTheme> {
    let pressed_state = Rc::new(Cell::new(false));
    let normal_style = ButtonInteractionStyle {
        text_decoration: TEXT_DECORATION_NONE,
        ..interaction_style(variant_style)
    };
    let normal_border_radius = Rc::new(edge_all(radius::MD));

    {
        let pressed_state = pressed_state.clone();
        let normal_border_radius = normal_border_radius.clone();
        button = button.with_patch(move |node| {
            let runtime = RuntimeButtonNode(node.clone());
            apply_runtime_button_state(
                &runtime,
                normal_style,
                pressed_style,
                pressed_state.get(),
                normal_border_radius.as_slice(),
            );
            Ok(())
        });
    }

    {
        let pressed_state = pressed_state.clone();
        let normal_border_radius = normal_border_radius.clone();
        button = button.with_next_frame(move |node| {
            let runtime = RuntimeButtonNode(node.clone());
            apply_runtime_button_state(
                &runtime,
                normal_style,
                pressed_style,
                pressed_state.get(),
                normal_border_radius.as_slice(),
            );
            Ok(())
        });
    }

    {
        let pressed_state = pressed_state.clone();
        let normal_border_radius = normal_border_radius.clone();
        button = button.with_next_idle(move |node| {
            let runtime = RuntimeButtonNode(node.clone());
            apply_runtime_button_state(
                &runtime,
                normal_style,
                pressed_style,
                pressed_state.get(),
                normal_border_radius.as_slice(),
            );
            Ok(())
        });
    }

    if let Some(pressed_style) = pressed_style {
        let pressed_state = pressed_state.clone();
        let normal_border_radius = normal_border_radius.clone();
        button = button.on_supported_ui_states(UiState::PRESSED, true, move |node, current| {
            let runtime = RuntimeButtonNode(node.clone());
            if current.contains(UiState::PRESSED) {
                pressed_state.set(true);
            } else {
                pressed_state.set(false);
            }
            apply_runtime_button_state(
                &runtime,
                normal_style,
                Some(pressed_style),
                pressed_state.get(),
                normal_border_radius.as_slice(),
            );
        });
    }

    button = if variant_style.shadow {
        subtle_button_shadow(button)
    } else {
        clear_button_shadow(button)
    };

    button
        .patch_attr(ArkUINodeAttributeType::Clip, true)
        .patch_attr(ArkUINodeAttributeType::BorderRadius, edge_all(radius::MD))
}

fn button_content_row<Message: 'static>(
    label: Option<String>,
    icon_name: Option<String>,
    foreground: u32,
    icon_size: f32,
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
            .font_color(foreground)
            .font_weight(FontWeight::W500)
            .line_height(20.0)
            .into();

        if children.is_empty() {
            children.push(text);
        } else {
            children.push(
                arkit::row_component()
                    .margin_left(8.0)
                    .children(vec![text])
                    .into(),
            );
        }
    }

    arkit::row_component()
        .align_items_center()
        .justify_content_center()
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

fn apply_runtime_button_state(
    node: &RuntimeButtonNode,
    normal: ButtonInteractionStyle,
    pressed: Option<ButtonInteractionStyle>,
    is_pressed: bool,
    border_radius: &[f32],
) {
    apply_button_interaction_state(node, normal, pressed, is_pressed, border_radius);
    with_runtime_button_content(node, |content| {
        if is_pressed {
            if let Some(pressed) = pressed {
                apply_content_interaction_style_if_changed(content, pressed, normal.foreground);
                return;
            }
        }
        restore_content_interaction_style(content, normal.foreground);
    });
}

fn apply_button_interaction_state(
    node: &RuntimeButtonNode,
    normal: ButtonInteractionStyle,
    pressed: Option<ButtonInteractionStyle>,
    is_pressed: bool,
    border_radius: &[f32],
) {
    if is_pressed {
        if let Some(pressed) = pressed {
            apply_interaction_style(node, pressed, border_radius);
            return;
        }
    }
    apply_interaction_style(node, normal, border_radius);
}

fn apply_interaction_style(
    node: &RuntimeButtonNode,
    style: ButtonInteractionStyle,
    border_radius: &[f32],
) {
    let _ = node.set_attribute(
        ArkUINodeAttributeType::Clip,
        ArkUINodeAttributeItem::from(true),
    );
    let _ = node.set_attribute(
        ArkUINodeAttributeType::BorderStyle,
        ArkUINodeAttributeItem::from(i32::from(BorderStyle::Solid)),
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
        ArkUINodeAttributeItem::from(border_radius.to_vec()),
    );
    let _ = node.font_color(style.foreground);
    let _ = node.set_attribute(
        ArkUINodeAttributeType::TextDecoration,
        text_decoration(style.text_decoration, style.foreground),
    );
}

fn subtle_button_shadow<Message, AppTheme>(
    element: ButtonElement<Message, AppTheme>,
) -> ButtonElement<Message, AppTheme> {
    element.shadow(ShadowStyle::OuterDefaultSm)
}

fn clear_button_shadow<Message, AppTheme>(
    element: ButtonElement<Message, AppTheme>,
) -> ButtonElement<Message, AppTheme> {
    element.clear_shadow()
}

fn text_decoration(decoration_type: i32, color_value: u32) -> ArkUINodeAttributeItem {
    ArkUINodeAttributeItem::NumberValue(vec![
        ArkUINodeAttributeNumber::Int(decoration_type),
        ArkUINodeAttributeNumber::Uint(color_value),
        ArkUINodeAttributeNumber::Int(TEXT_DECORATION_STYLE_SOLID),
    ])
}
