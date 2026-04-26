use super::*;
use arkit::ohos_arkui_binding::common::attribute::{
    ArkUINodeAttributeItem, ArkUINodeAttributeNumber,
};
use arkit::ohos_arkui_binding::types::alignment::Alignment;
use arkit::prelude::Padding;
use arkit::{ShadowStyle, TextAlignment};
use arkit_icon as lucide;

const TRANSPARENT: u32 = 0x00000000;
const TEXT_DECORATION_NONE: i32 = 0;
const TEXT_DECORATION_STYLE_SOLID: i32 = 0;

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

pub(super) fn button<Message: Send + 'static>(label: impl Into<String>) -> ButtonElement<Message> {
    apply_button_size(
        apply_button_theme(button_host(normal_button(label)), ButtonVariant::Default),
        size_style(ButtonSize::Default),
    )
}

fn button_with_icon<Message: Send + 'static>(
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
    let button = button_host(normal_button_component())
        .children(vec![button_content_row(
            Some(label),
            Some(icon_name),
            colors().foreground,
            icon_size(size),
        )])
        .size(size);

    apply_button_theme(button, ButtonVariant::Default)
}

pub(super) fn icon_button<Message: Send + 'static>(
    icon: impl Into<String>,
) -> ButtonElement<Message> {
    let button = button_host(normal_button_component())
        .children(vec![button_content_row(
            None,
            Some(icon.into()),
            colors().foreground,
            icon_size(ButtonSize::Icon),
        )])
        .size(ButtonSize::Icon);

    apply_button_theme(button, ButtonVariant::Default)
}

fn normal_button_component<Message, AppTheme>() -> ButtonElement<Message, AppTheme> {
    arkit::button_component::<Message, AppTheme>()
}

fn normal_button<Message: 'static, AppTheme: 'static>(
    label: impl Into<String>,
) -> ButtonElement<Message, AppTheme> {
    normal_button_component().children(vec![arkit::text(label).into()])
}

fn button_host<Message, AppTheme>(
    element: ButtonElement<Message, AppTheme>,
) -> ButtonElement<Message, AppTheme> {
    element
        .focusable(false)
        .focus_on_touch(false)
        .background_color(TRANSPARENT)
        .border_style(BorderStyle::Solid)
        .border_radius(radii().md)
        .clip(true)
        .alignment(Alignment::Center)
        .padding(Padding::ZERO)
        .border_width(0.0)
        .border_color(TRANSPARENT)
        .align_self(ItemAlignment::Start)
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
    let initial_text_decoration = TEXT_DECORATION_NONE;

    let button = element
        .border_radius(radii().md)
        .clip(true)
        .border_width(variant_style.border_width)
        .border_color(variant_style.border_color)
        .background_color(variant_style.background)
        .font_weight(FontWeight::W500)
        .font_color(variant_style.foreground)
        .text_align(TextAlignment::Center)
        .text_decoration(text_decoration(
            initial_text_decoration,
            variant_style.foreground,
        ));

    finalize_button(button, variant_style)
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
            background: colors().primary,
            foreground: colors().primary_foreground,
            border_width: 0.0,
            border_color: TRANSPARENT,
            shadow: true,
        },
        // bg-secondary, shadow-sm
        ButtonVariant::Secondary => ButtonVariantStyle {
            background: colors().secondary,
            foreground: colors().secondary_foreground,
            border_width: 0.0,
            border_color: TRANSPARENT,
            shadow: true,
        },
        // border-border bg-background, shadow-sm
        ButtonVariant::Outline => ButtonVariantStyle {
            background: colors().background,
            foreground: colors().foreground,
            border_width: 1.0,
            border_color: colors().border,
            shadow: true,
        },
        // no bg, no shadow
        ButtonVariant::Ghost => ButtonVariantStyle {
            background: TRANSPARENT,
            foreground: colors().foreground,
            border_width: 0.0,
            border_color: TRANSPARENT,
            shadow: false,
        },
        // bg-destructive, shadow-sm
        ButtonVariant::Destructive => ButtonVariantStyle {
            background: colors().destructive,
            foreground: colors().destructive_foreground,
            border_width: 0.0,
            border_color: TRANSPARENT,
            shadow: true,
        },
        // no bg, no shadow
        ButtonVariant::Link => ButtonVariantStyle {
            background: TRANSPARENT,
            foreground: colors().primary,
            border_width: 0.0,
            border_color: TRANSPARENT,
            shadow: false,
        },
    }
}

fn finalize_button<Message: Send + 'static, AppTheme>(
    mut button: ButtonElement<Message, AppTheme>,
    variant_style: ButtonVariantStyle,
) -> ButtonElement<Message, AppTheme> {
    button = if variant_style.shadow {
        subtle_button_shadow(button)
    } else {
        clear_button_shadow(button)
    };

    button
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
                arkit::row_component::<Message, arkit::Theme>()
                    .margin([0.0, 0.0, 0.0, 8.0])
                    .children(vec![text])
                    .into(),
            );
        }
    }

    arkit::row_component::<Message, arkit::Theme>()
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

// Struct component API
enum ButtonContent {
    Label,
    IconLabel,
    Icon,
}

pub struct Button<Message = ()> {
    content: ButtonContent,
    label: Option<String>,
    icon_name: Option<String>,
    variant: ButtonVariant,
    size: ButtonSize,
    disabled: bool,
    key: Option<String>,
    width: Option<arkit::Length>,
    height: Option<arkit::Length>,
    padding: Option<arkit::Padding>,
    on_press: std::cell::RefCell<Option<Message>>,
    on_click: Option<std::rc::Rc<dyn Fn()>>,
}

impl<Message> Button<Message> {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            content: ButtonContent::Label,
            label: Some(label.into()),
            icon_name: None,
            variant: ButtonVariant::Default,
            size: ButtonSize::Default,
            disabled: false,
            key: None,
            width: None,
            height: None,
            padding: None,
            on_press: std::cell::RefCell::new(None),
            on_click: None,
        }
    }

    pub fn with_icon(label: impl Into<String>, icon_name: impl Into<String>) -> Self {
        Self {
            content: ButtonContent::IconLabel,
            label: Some(label.into()),
            icon_name: Some(icon_name.into()),
            ..Self::new("")
        }
    }

    pub fn icon(icon_name: impl Into<String>) -> Self {
        Self {
            content: ButtonContent::Icon,
            label: None,
            icon_name: Some(icon_name.into()),
            size: ButtonSize::Icon,
            ..Self::new("")
        }
    }

    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn theme(self, variant: ButtonVariant) -> Self {
        self.variant(variant)
    }

    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn key(mut self, key: impl Into<String>) -> Self {
        self.key = Some(key.into());
        self
    }

    pub fn width(mut self, width: impl Into<arkit::Length>) -> Self {
        self.width = Some(width.into());
        self
    }

    pub fn percent_width(mut self, value: f32) -> Self {
        self.width = Some(arkit::Length::FillPortion((value * 1000.0) as u16));
        self
    }

    pub fn height(mut self, height: impl Into<arkit::Length>) -> Self {
        self.height = Some(height.into());
        self
    }

    pub fn padding(mut self, padding: impl Into<arkit::Padding>) -> Self {
        self.padding = Some(padding.into());
        self
    }

    pub fn on_press(self, message: Message) -> Self {
        *self.on_press.borrow_mut() = Some(message);
        self
    }

    pub fn on_click(mut self, callback: impl Fn() + 'static) -> Self {
        self.on_click = Some(std::rc::Rc::new(callback));
        self
    }
}

impl<Message: Clone + Send + 'static> Button<Message> {
    fn render(&self) -> Element<Message> {
        let mut button = match self.content {
            ButtonContent::Label => button(self.label.clone().unwrap_or_default()),
            ButtonContent::IconLabel => button_with_icon(
                self.label.clone().unwrap_or_default(),
                self.icon_name.clone().unwrap_or_default(),
            ),
            ButtonContent::Icon => icon_button(self.icon_name.clone().unwrap_or_default()),
        }
        .theme(self.variant)
        .size(self.size)
        .disabled(self.disabled);

        if let Some(key) = self.key.clone() {
            button = button.key(key);
        }
        if let Some(width) = self.width {
            button = button.width(width);
        }
        if let Some(height) = self.height {
            button = button.height(height);
        }
        if let Some(padding) = self.padding {
            button = button.padding(padding);
        }
        if let Some(message) = self.on_press.borrow_mut().take() {
            button = button.on_press(message);
        }
        if let Some(callback) = self.on_click.clone() {
            button = button.on_click(move || callback());
        }

        button.into()
    }
}

impl<Message: Clone + Send + 'static>
    arkit::advanced::Widget<Message, arkit::Theme, arkit::Renderer> for Button<Message>
{
    fn body(
        &self,
        _tree: &mut arkit::advanced::widget::Tree,
        _renderer: &arkit::Renderer,
    ) -> Option<Element<Message>> {
        Some(self.render())
    }
}

impl<Message: Clone + Send + 'static> From<Button<Message>> for Element<Message> {
    fn from(value: Button<Message>) -> Self {
        Element::new(value)
    }
}
