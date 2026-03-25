use super::*;
use arkit::ohos_arkui_binding::common::attribute::ArkUINodeAttributeNumber;
use arkit::ohos_arkui_binding::component::attribute::ArkUICommonAttribute;
use arkit_icon as lucide;

const CHECKBOX_SIZE: f32 = 16.0;
const CHECKBOX_BORDER_WIDTH: f32 = 1.0;
const CHECKBOX_SHAPE_ROUNDED_SQUARE: i32 = 1;
const TRANSPARENT: u32 = 0x00000000;
const CHECKBOX_ICON_SIZE: f32 = 12.0;
const CHECKBOX_ICON_STROKE_WIDTH: f32 = 3.5;

#[derive(Debug, Clone, Copy)]
struct CheckboxStyle {
    checked_color: u32,
    disabled: bool,
}

fn checkbox_border_color(selected: bool, checked_color: u32) -> u32 {
    if selected {
        checked_color
    } else {
        TRANSPARENT
    }
}

fn checkbox_background_color(selected: bool, checked_color: u32) -> u32 {
    if selected {
        checked_color
    } else {
        color::BACKGROUND
    }
}

fn checkbox_unselect_color(selected: bool) -> u32 {
    if selected {
        TRANSPARENT
    } else {
        color::INPUT
    }
}

fn checkbox_border_width(selected: bool) -> f32 {
    if selected {
        CHECKBOX_BORDER_WIDTH
    } else {
        0.0
    }
}

fn unchecked_shell() -> RowElement {
    shadow_sm(
        arkit::row_component()
            .width(CHECKBOX_SIZE)
            .height(CHECKBOX_SIZE)
            .style(
                ArkUINodeAttributeType::BorderRadius,
                vec![radius::SM, radius::SM, radius::SM, radius::SM],
            )
            .style(
                ArkUINodeAttributeType::BorderWidth,
                vec![
                    CHECKBOX_BORDER_WIDTH,
                    CHECKBOX_BORDER_WIDTH,
                    CHECKBOX_BORDER_WIDTH,
                    CHECKBOX_BORDER_WIDTH,
                ],
            )
            .style(ArkUINodeAttributeType::BorderColor, vec![color::INPUT])
            .style(ArkUINodeAttributeType::Clip, true)
            .background_color(color::BACKGROUND),
    )
}

fn checked_shell(checked_color: u32) -> RowElement {
    shadow_sm(
        arkit::row_component()
            .width(CHECKBOX_SIZE)
            .height(CHECKBOX_SIZE)
            .align_items_center()
            .style(ArkUINodeAttributeType::RowJustifyContent, FLEX_ALIGN_CENTER)
            .style(
                ArkUINodeAttributeType::BorderRadius,
                vec![radius::SM, radius::SM, radius::SM, radius::SM],
            )
            .style(
                ArkUINodeAttributeType::BorderWidth,
                vec![
                    CHECKBOX_BORDER_WIDTH,
                    CHECKBOX_BORDER_WIDTH,
                    CHECKBOX_BORDER_WIDTH,
                    CHECKBOX_BORDER_WIDTH,
                ],
            )
            .style(ArkUINodeAttributeType::BorderColor, vec![checked_color])
            .style(ArkUINodeAttributeType::Clip, true)
            .background_color(checked_color)
            .children(vec![lucide::icon("check")
                .size(CHECKBOX_ICON_SIZE)
                .stroke_width(CHECKBOX_ICON_STROKE_WIDTH)
                .color(color::PRIMARY_FOREGROUND)
                .render()]),
    )
}

fn checkbox_mark_style() -> Vec<ArkUINodeAttributeNumber> {
    vec![
        ArkUINodeAttributeNumber::Uint(color::PRIMARY_FOREGROUND),
        ArkUINodeAttributeNumber::Float(10.0),
        ArkUINodeAttributeNumber::Float(2.0),
    ]
}

fn checkbox_impl(label_text: String, checked: Signal<bool>, style: CheckboxStyle) -> Element {
    let watch_state = checked.clone();
    let checked_color = style.checked_color;
    let disabled = style.disabled;
    let selected = checked.get();

    let mut checkbox = shadow_sm(
        arkit::checkbox_component()
            .watch_signal(watch_state, move |node, value| {
                node.set_checkbox_select(value)?;
                node.set_checkbox_shape(CHECKBOX_SHAPE_ROUNDED_SQUARE)?;
                node.set_checkbox_mark(checkbox_mark_style())?;
                node.set_checkbox_select_color(checked_color)?;
                node.set_checkbox_unselect_color(checkbox_unselect_color(value))?;
                node.set_attribute(
                    ArkUINodeAttributeType::BackgroundColor,
                    checkbox_background_color(value, checked_color).into(),
                )?;
                node.set_attribute(
                    ArkUINodeAttributeType::BorderColor,
                    vec![checkbox_border_color(value, checked_color)].into(),
                )?;
                node.set_attribute(
                    ArkUINodeAttributeType::BorderWidth,
                    vec![
                        checkbox_border_width(value),
                        checkbox_border_width(value),
                        checkbox_border_width(value),
                        checkbox_border_width(value),
                    ]
                    .into(),
                )?;
                node.set_attribute(ArkUINodeAttributeType::Opacity, 0.0_f32.into())
            })
            .style(ArkUINodeAttributeType::CheckboxSelect, checked.get())
            .style(ArkUINodeAttributeType::CheckboxSelectColor, checked_color)
            .style(
                ArkUINodeAttributeType::CheckboxUnselectColor,
                checkbox_unselect_color(checked.get()),
            )
            .style(
                ArkUINodeAttributeType::CheckboxShape,
                CHECKBOX_SHAPE_ROUNDED_SQUARE,
            )
            .style(ArkUINodeAttributeType::CheckboxMark, checkbox_mark_style())
            .style(
                ArkUINodeAttributeType::BackgroundColor,
                checkbox_background_color(checked.get(), checked_color),
            )
            .style(
                ArkUINodeAttributeType::BorderColor,
                vec![checkbox_border_color(checked.get(), checked_color)],
            )
            .style(
                ArkUINodeAttributeType::BorderWidth,
                vec![
                    checkbox_border_width(checked.get()),
                    checkbox_border_width(checked.get()),
                    checkbox_border_width(checked.get()),
                    checkbox_border_width(checked.get()),
                ],
            )
            .style(
                ArkUINodeAttributeType::BorderRadius,
                vec![radius::SM, radius::SM, radius::SM, radius::SM],
            )
            .style(ArkUINodeAttributeType::Opacity, 0.0_f32)
            .style(ArkUINodeAttributeType::Clip, true)
            .width(CHECKBOX_SIZE)
            .height(CHECKBOX_SIZE),
    );

    if disabled {
        checkbox = checkbox
            .style(ArkUINodeAttributeType::Enabled, false)
            .style(ArkUINodeAttributeType::Opacity, 0.5_f32);
    }

    let visual = if selected {
        checked_shell(checked_color).into()
    } else {
        unchecked_shell().into()
    };

    let mut indicator = arkit::stack_component()
        .width(CHECKBOX_SIZE)
        .height(CHECKBOX_SIZE)
        .children(vec![checkbox.into(), visual]);

    if disabled {
        indicator = indicator.style(ArkUINodeAttributeType::Opacity, 0.5_f32);
    }

    if label_text.is_empty() {
        if !disabled {
            let click = checked.clone();
            indicator = indicator.on_click(move || click.update(|value| *value = !*value));
        }
        return indicator.into();
    }

    let mut children = vec![indicator.into()];

    if !label_text.is_empty() {
        let mut text = label(label_text);
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

    let mut root = arkit::row_component()
        .align_items_center()
        .children(children);
    if !disabled {
        let click = checked.clone();
        root = root.on_click(move || click.update(|value| *value = !*value));
    }

    root.into()
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
