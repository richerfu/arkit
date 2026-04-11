use super::*;
use arkit::ohos_arkui_binding::types::alignment::Alignment;
use arkit_icon as lucide;

pub(crate) const TOGGLE_TRANSPARENT: u32 = 0x00000000;
const FONT_WEIGHT_MEDIUM: i32 = 4;
const FLEX_ALIGN_CENTER: i32 = 2;

pub(crate) fn color_all(value: u32) -> Vec<u32> {
    vec![value, value, value, value]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ToggleVariant {
    Default,
    Outline,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ToggleSizeStyle {
    pub(crate) height: f32,
    pub(crate) width: Option<f32>,
    pub(crate) padding: [f32; 4],
    pub(crate) icon_size: f32,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ToggleVisualStyle {
    pub(crate) background: u32,
    pub(crate) foreground: u32,
    pub(crate) border_color: u32,
    shadow: bool,
}

pub(crate) fn toggle_default_size() -> ToggleSizeStyle {
    ToggleSizeStyle {
        height: 40.0,
        width: None,
        padding: [8.0, 10.0, 8.0, 10.0],
        icon_size: 16.0,
    }
}

pub(crate) fn toggle_icon_size() -> ToggleSizeStyle {
    ToggleSizeStyle {
        height: 40.0,
        width: Some(40.0),
        padding: [0.0, 0.0, 0.0, 0.0],
        icon_size: 16.0,
    }
}

pub(crate) fn toggle_visual_style(variant: ToggleVariant, active: bool) -> ToggleVisualStyle {
    match variant {
        ToggleVariant::Default => ToggleVisualStyle {
            background: if active {
                color::ACCENT
            } else {
                TOGGLE_TRANSPARENT
            },
            foreground: if active {
                color::ACCENT_FOREGROUND
            } else {
                color::FOREGROUND
            },
            border_color: TOGGLE_TRANSPARENT,
            shadow: false,
        },
        ToggleVariant::Outline => ToggleVisualStyle {
            background: if active {
                color::ACCENT
            } else {
                TOGGLE_TRANSPARENT
            },
            foreground: if active {
                color::ACCENT_FOREGROUND
            } else {
                color::FOREGROUND
            },
            border_color: color::INPUT,
            shadow: true,
        },
    }
}

pub(crate) fn toggle_content_row<Message: 'static>(
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
            .style(ArkUINodeAttributeType::FontColor, foreground)
            .style(ArkUINodeAttributeType::FontWeight, FONT_WEIGHT_MEDIUM)
            .style(ArkUINodeAttributeType::TextLineHeight, 20.0)
            .into();

        if children.is_empty() {
            children.push(text);
        } else {
            children.push(
                arkit::row_component::<Message, arkit::Theme>()
                    .style(ArkUINodeAttributeType::Margin, vec![0.0, 0.0, 0.0, 8.0])
                    .children(vec![text])
                    .into(),
            );
        }
    }

    arkit::row_component::<Message, arkit::Theme>()
        .align_items_center()
        .style(ArkUINodeAttributeType::RowJustifyContent, FLEX_ALIGN_CENTER)
        .children(children)
        .into()
}

pub(crate) fn toggle_surface<Message>(
    content: Element<Message>,
    active: bool,
    variant: ToggleVariant,
    size_style: ToggleSizeStyle,
    border_width: [f32; 4],
    border_radius: [f32; 4],
    shadow_override: Option<bool>,
) -> ButtonElement<Message> {
    let visual = toggle_visual_style(variant, active);
    let mut element = normal_button_component::<Message, arkit::Theme>()
        .style(ArkUINodeAttributeType::Clip, true)
        .style(ArkUINodeAttributeType::BorderStyle, 0_i32)
        .style(ArkUINodeAttributeType::AlignSelf, 1_i32)
        .style(ArkUINodeAttributeType::BorderRadius, border_radius.to_vec())
        .style(ArkUINodeAttributeType::BorderWidth, border_width.to_vec())
        .style(
            ArkUINodeAttributeType::BorderColor,
            color_all(visual.border_color),
        )
        .style(
            ArkUINodeAttributeType::Alignment,
            i32::from(Alignment::Center),
        )
        .style(ArkUINodeAttributeType::Padding, size_style.padding.to_vec())
        .background_color(visual.background)
        .patch_background_color(visual.background)
        .patch_attr(ArkUINodeAttributeType::BorderWidth, border_width.to_vec())
        .patch_attr(
            ArkUINodeAttributeType::BorderColor,
            color_all(visual.border_color),
        )
        .height(size_style.height)
        .children(vec![content]);

    if let Some(width) = size_style.width {
        element = element.width(width);
    }

    if shadow_override.unwrap_or(visual.shadow) {
        shadow_sm(element)
    } else {
        element
    }
}

pub fn toggle<Message: Send + 'static>(
    label: impl Into<String>,
    state: bool,
    on_toggle: impl Fn(bool) + 'static,
) -> Element<Message> {
    let label_text = label.into();
    let size_style = toggle_default_size();
    let visual = toggle_visual_style(ToggleVariant::Default, state);

    toggle_surface(
        toggle_content_row(
            Some(label_text),
            None,
            visual.foreground,
            size_style.icon_size,
        ),
        state,
        ToggleVariant::Default,
        size_style,
        [0.0, 0.0, 0.0, 0.0],
        [radius::MD, radius::MD, radius::MD, radius::MD],
        None,
    )
    .on_click(move || on_toggle(!state))
    .into()
}

pub fn toggle_message<Message>(
    label: impl Into<String>,
    state: bool,
    on_toggle: impl Fn(bool) -> Message + 'static,
) -> Element<Message>
where
    Message: Send + 'static,
{
    toggle(label, state, move |value| {
        dispatch_message(on_toggle(value))
    })
}

pub fn toggle_icon<Message: Send + 'static>(
    icon_name: impl Into<String>,
    state: bool,
    on_toggle: impl Fn(bool) + 'static,
) -> Element<Message> {
    let icon = icon_name.into();
    let size_style = toggle_icon_size();
    let visual = toggle_visual_style(ToggleVariant::Default, state);

    toggle_surface(
        toggle_content_row(None, Some(icon), visual.foreground, size_style.icon_size),
        state,
        ToggleVariant::Default,
        size_style,
        [0.0, 0.0, 0.0, 0.0],
        [radius::MD, radius::MD, radius::MD, radius::MD],
        None,
    )
    .on_click(move || on_toggle(!state))
    .into()
}

pub fn toggle_icon_message<Message>(
    icon_name: impl Into<String>,
    state: bool,
    on_toggle: impl Fn(bool) -> Message + 'static,
) -> Element<Message>
where
    Message: Send + 'static,
{
    toggle_icon(icon_name, state, move |value| {
        dispatch_message(on_toggle(value))
    })
}
