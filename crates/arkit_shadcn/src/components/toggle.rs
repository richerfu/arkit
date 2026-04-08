use super::*;
use arkit::ohos_arkui_binding::types::alignment::Alignment;
use arkit::ohos_arkui_binding::types::text_alignment::TextAlignment;
use arkit_icon as lucide;

const TRANSPARENT: u32 = 0x00000000;

fn color_all(value: u32) -> Vec<u32> {
    vec![value, value, value, value]
}

fn toggle_surface<Message>(
    element: ButtonElement<Message>,
    active: bool,
    inactive_background: u32,
    border_width: [f32; 4],
    border_color: u32,
    border_radius: [f32; 4],
    shadow: bool,
) -> ButtonElement<Message> {
    let element = element
        .style(ArkUINodeAttributeType::Clip, true)
        .style(ArkUINodeAttributeType::BorderStyle, 0_i32)
        .style(ArkUINodeAttributeType::AlignSelf, 1_i32)
        .style(ArkUINodeAttributeType::BorderRadius, border_radius.to_vec())
        .style(ArkUINodeAttributeType::BorderWidth, border_width.to_vec())
        .style(ArkUINodeAttributeType::BorderColor, color_all(border_color))
        .style(
            ArkUINodeAttributeType::Alignment,
            i32::from(Alignment::Center),
        )
        .patch_background_color(if active {
            color::ACCENT
        } else {
            inactive_background
        });

    if shadow {
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
    toggle_surface(
        normal_button::<Message, arkit::Theme>(label_text)
            .height(40.0)
            .style(ArkUINodeAttributeType::Padding, vec![8.0, 10.0, 8.0, 10.0])
            .font_size(typography::SM)
            .style(ArkUINodeAttributeType::FontWeight, 4_i32)
            .style(
                ArkUINodeAttributeType::TextAlign,
                i32::from(TextAlignment::Center),
            )
            .patch_attr(
                ArkUINodeAttributeType::FontColor,
                if state {
                    color::ACCENT_FOREGROUND
                } else {
                    color::FOREGROUND
                },
            ),
        state,
        TRANSPARENT,
        [0.0, 0.0, 0.0, 0.0],
        TRANSPARENT,
        [radius::MD, radius::MD, radius::MD, radius::MD],
        false,
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
    toggle_surface(
        normal_button_component::<Message, arkit::Theme>()
            .width(40.0)
            .height(40.0)
            .style(ArkUINodeAttributeType::Padding, vec![0.0, 0.0, 0.0, 0.0])
            .children(vec![lucide::icon(icon)
                .size(16.0)
                .color(if state {
                    color::ACCENT_FOREGROUND
                } else {
                    color::FOREGROUND
                })
                .render::<Message, arkit::Theme>()]),
        state,
        TRANSPARENT,
        [0.0, 0.0, 0.0, 0.0],
        TRANSPARENT,
        [radius::MD, radius::MD, radius::MD, radius::MD],
        false,
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
