use super::*;
use arkit::ohos_arkui_binding::component::attribute::ArkUICommonAttribute;

#[derive(Debug, Clone, Copy)]
struct CheckboxStyle {
    checked_color: u32,
    disabled: bool,
}

fn checkbox_impl(label: String, checked: Signal<bool>, style: CheckboxStyle) -> Element {
    let click = checked.clone();
    let watch_state = checked.clone();
    let checked_color = style.checked_color;
    let disabled = style.disabled;

    let mut checkbox = shadow_sm(
        arkit::checkbox_component()
            .watch_signal(watch_state, move |node, value| {
                node.set_attribute(ArkUINodeAttributeType::CheckboxSelect, value.into())?;
                node.set_attribute(
                    ArkUINodeAttributeType::BorderColor,
                    vec![if value { checked_color } else { color::INPUT }].into(),
                )?;
                node.set_attribute(ArkUINodeAttributeType::CheckboxSelectColor, checked_color.into())
            })
            .style(ArkUINodeAttributeType::CheckboxSelect, checked.get())
            .style(ArkUINodeAttributeType::CheckboxSelectColor, checked_color)
            .style(
                ArkUINodeAttributeType::CheckboxUnselectColor,
                color::BACKGROUND,
            )
            .style(
                ArkUINodeAttributeType::BorderColor,
                vec![if checked.get() {
                    checked_color
                } else {
                    color::INPUT
                }],
            )
            .style(
                ArkUINodeAttributeType::BorderWidth,
                vec![1.0, 1.0, 1.0, 1.0],
            )
            .style(
                ArkUINodeAttributeType::BorderRadius,
                vec![4.0, 4.0, 4.0, 4.0],
            )
            .style(ArkUINodeAttributeType::Clip, true)
            .width(16.0)
            .height(16.0),
    );

    if disabled {
        checkbox = checkbox
            .style(ArkUINodeAttributeType::Enabled, false)
            .style(ArkUINodeAttributeType::Opacity, 0.5_f32);
    } else {
        checkbox = checkbox.on_click(move || click.update(|value| *value = !*value));
    }

    let mut children = vec![checkbox.into()];

    if !label.is_empty() {
        let mut text = arkit::text(label)
            .font_size(typography::SM)
            .style(ArkUINodeAttributeType::FontWeight, 4_i32)
            .style(ArkUINodeAttributeType::FontColor, color::FOREGROUND)
            .style(ArkUINodeAttributeType::TextLineHeight, 20.0);

        if disabled {
            text = text.style(ArkUINodeAttributeType::Opacity, 0.5_f32);
        }

        children.push(
            arkit::row_component()
                .style(
                    ArkUINodeAttributeType::Margin,
                    vec![0.0, 0.0, 0.0, spacing::SM],
                )
                .children(vec![text.into()])
                .into(),
        );
    }

    arkit::row_component()
        .style(ArkUINodeAttributeType::RowAlignItems, FLEX_ALIGN_CENTER)
        .children(children)
        .into()
}

pub fn checkbox(label: impl Into<String>, checked: Signal<bool>) -> Element {
    checkbox_impl(
        label.into(),
        checked,
        CheckboxStyle {
            checked_color: color::PRIMARY,
            disabled: false,
        },
    )
}

pub fn checkbox_with_checked_color(
    label: impl Into<String>,
    checked: Signal<bool>,
    checked_color: u32,
) -> Element {
    checkbox_impl(
        label.into(),
        checked,
        CheckboxStyle {
            checked_color,
            disabled: false,
        },
    )
}

pub fn disabled_checkbox(label: impl Into<String>, checked: Signal<bool>) -> Element {
    checkbox_impl(
        label.into(),
        checked,
        CheckboxStyle {
            checked_color: color::PRIMARY,
            disabled: true,
        },
    )
}
