use super::*;
use arkit::ohos_arkui_binding::types::alignment::Alignment;
use arkit::ohos_arkui_binding::types::text_alignment::TextAlignment;
use arkit_icon as lucide;

const TRANSPARENT: u32 = 0x00000000;

fn color_all(value: u32) -> Vec<u32> {
    vec![value, value, value, value]
}

fn toggle_surface(
    element: ButtonElement,
    active: bool,
    inactive_background: u32,
    border_width: [f32; 4],
    border_color: u32,
    border_radius: [f32; 4],
    shadow: bool,
) -> ButtonElement {
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

pub fn toggle(label: impl Into<String>, state: Signal<bool>) -> Element {
    let label_text = label.into();
    let next = state.clone();

    arkit::dynamic(move || {
        let active = state.get();
        let next = next.clone();

        toggle_surface(
            normal_button(label_text.clone())
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
                    if active {
                        color::ACCENT_FOREGROUND
                    } else {
                        color::FOREGROUND
                    },
                ),
            active,
            TRANSPARENT,
            [0.0, 0.0, 0.0, 0.0],
            TRANSPARENT,
            [radius::MD, radius::MD, radius::MD, radius::MD],
            false,
        )
        .on_click(move || next.update(|value| *value = !*value))
        .into()
    })
}

pub fn toggle_icon(icon_name: impl Into<String>, state: Signal<bool>) -> Element {
    let icon = icon_name.into();
    let next = state.clone();

    arkit::dynamic(move || {
        let active = state.get();
        let next = next.clone();

        toggle_surface(
            normal_button_component()
                .width(40.0)
                .height(40.0)
                .style(ArkUINodeAttributeType::Padding, vec![0.0, 0.0, 0.0, 0.0])
                .children(vec![lucide::icon(icon.clone())
                    .size(16.0)
                    .color(if active {
                        color::ACCENT_FOREGROUND
                    } else {
                        color::FOREGROUND
                    })
                    .render()]),
            active,
            TRANSPARENT,
            [0.0, 0.0, 0.0, 0.0],
            TRANSPARENT,
            [radius::MD, radius::MD, radius::MD, radius::MD],
            false,
        )
        .on_click(move || next.update(|value| *value = !*value))
        .into()
    })
}
