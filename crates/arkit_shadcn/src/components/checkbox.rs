use super::*;
use arkit::ohos_arkui_binding::component::attribute::ArkUICommonAttribute;
use arkit::ohos_arkui_binding::types::alignment::Alignment;
use arkit_icon as lucide;
use std::rc::Rc;

const CHECKBOX_SIZE: f32 = 16.0;
const CHECKBOX_BORDER_WIDTH: f32 = 1.0;
const TRANSPARENT: u32 = 0x00000000;
const CHECKBOX_ICON_SIZE: f32 = 12.0;
const CHECKBOX_ICON_STROKE_WIDTH: f32 = 3.5;
const HIT_TEST_TRANSPARENT: i32 = 2;

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

fn checkbox_border_width(selected: bool) -> f32 {
    if selected {
        CHECKBOX_BORDER_WIDTH
    } else {
        0.0
    }
}

fn unchecked_shell<Message: 'static>() -> RowElement<Message> {
    shadow_sm(
        arkit::row_component::<Message, arkit::Theme>()
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

fn checked_shell<Message: 'static>(checked_color: u32) -> RowElement<Message> {
    shadow_sm(
        arkit::row_component::<Message, arkit::Theme>()
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
                .render::<Message, arkit::Theme>()]),
    )
}

fn checkbox_impl(
    label_text: String,
    checked: bool,
    on_toggle: Option<Rc<dyn Fn(bool)>>,
    style: CheckboxStyle,
) -> Element {
    checkbox_impl_with_message::<()>(label_text, checked, on_toggle, style)
}

fn checkbox_impl_with_message<Message: 'static>(
    label_text: String,
    checked: bool,
    on_toggle: Option<Rc<dyn Fn(bool)>>,
    style: CheckboxStyle,
) -> Element<Message> {
    let checked_color = style.checked_color;
    let disabled = style.disabled;
    let has_label = !label_text.is_empty();

    let mut checkbox = arkit::checkbox_component::<Message, arkit::Theme>()
        .checked(checked)
        .patch_attr(
            ArkUINodeAttributeType::BackgroundColor,
            checkbox_background_color(checked, checked_color),
        )
        .patch_attr(
            ArkUINodeAttributeType::BorderColor,
            vec![checkbox_border_color(checked, checked_color)],
        )
        .patch_attr(
            ArkUINodeAttributeType::BorderWidth,
            vec![
                checkbox_border_width(checked),
                checkbox_border_width(checked),
                checkbox_border_width(checked),
                checkbox_border_width(checked),
            ],
        )
        .style(ArkUINodeAttributeType::CheckboxSelectColor, checked_color)
        .style(
            ArkUINodeAttributeType::BorderRadius,
            vec![radius::SM, radius::SM, radius::SM, radius::SM],
        )
        .style(ArkUINodeAttributeType::Opacity, 0.0_f32)
        .style(ArkUINodeAttributeType::Clip, true);

    if let Some(on_toggle) = on_toggle {
        checkbox = checkbox.on_toggle_local(move |value| on_toggle(value));
    }

    if disabled {
        checkbox = checkbox.style(ArkUINodeAttributeType::Enabled, false);
    }

    let visual_indicator = if checked {
        checked_shell::<Message>(style.checked_color).into()
    } else {
        unchecked_shell::<Message>().into()
    };

    let indicator = arkit::stack_component::<Message, arkit::Theme>()
        .width(CHECKBOX_SIZE)
        .height(CHECKBOX_SIZE)
        .style(
            ArkUINodeAttributeType::HitTestBehavior,
            HIT_TEST_TRANSPARENT,
        )
        .children(vec![visual_indicator]);

    let mut children = vec![indicator.into()];

    if has_label {
        let text = label(label_text);
        children.push(
            arkit::row_component::<Message, arkit::Theme>()
                .style(
                    ArkUINodeAttributeType::Margin,
                    vec![0.0, 0.0, 0.0, spacing::SM],
                )
                .children(vec![text.into()])
                .into(),
        );
    }

    let mut visual_row = arkit::row_component::<Message, arkit::Theme>()
        .align_items_center()
        .style(
            ArkUINodeAttributeType::HitTestBehavior,
            HIT_TEST_TRANSPARENT,
        )
        .children(children);

    if disabled {
        visual_row = visual_row.style(ArkUINodeAttributeType::Opacity, 0.5_f32);
    }

    let checkbox = if has_label {
        checkbox.percent_width(1.0).percent_height(1.0)
    } else {
        checkbox.width(CHECKBOX_SIZE).height(CHECKBOX_SIZE)
    };

    arkit::stack_component::<Message, arkit::Theme>()
        .native(move |node| node.set_alignment(i32::from(Alignment::TopStart)))
        .children(vec![checkbox.into(), visual_row.into()])
        .into()
}

pub fn checkbox(
    label: impl Into<String>,
    checked: bool,
    on_toggle: impl Fn(bool) + 'static,
) -> Element {
    checkbox_impl(
        label.into(),
        checked,
        Some(Rc::new(on_toggle)),
        CheckboxStyle {
            checked_color: color::PRIMARY,
            disabled: false,
        },
    )
}

pub fn checkbox_message<Message>(
    label: impl Into<String>,
    checked: bool,
    on_toggle: impl Fn(bool) -> Message + 'static,
) -> Element<Message>
where
    Message: Send + 'static,
{
    checkbox_impl_with_message(
        label.into(),
        checked,
        Some(Rc::new(move |value| dispatch_message(on_toggle(value)))),
        CheckboxStyle {
            checked_color: color::PRIMARY,
            disabled: false,
        },
    )
}

pub fn checkbox_with_checked_color(
    label: impl Into<String>,
    checked: bool,
    on_toggle: impl Fn(bool) + 'static,
    checked_color: u32,
) -> Element {
    checkbox_impl(
        label.into(),
        checked,
        Some(Rc::new(on_toggle)),
        CheckboxStyle {
            checked_color,
            disabled: false,
        },
    )
}

pub fn checkbox_with_checked_color_message<Message>(
    label: impl Into<String>,
    checked: bool,
    on_toggle: impl Fn(bool) -> Message + 'static,
    checked_color: u32,
) -> Element<Message>
where
    Message: Send + 'static,
{
    checkbox_impl_with_message(
        label.into(),
        checked,
        Some(Rc::new(move |value| dispatch_message(on_toggle(value)))),
        CheckboxStyle {
            checked_color,
            disabled: false,
        },
    )
}

pub fn disabled_checkbox<Message: 'static>(
    label: impl Into<String>,
    checked: bool,
) -> Element<Message> {
    checkbox_impl_with_message(
        label.into(),
        checked,
        None,
        CheckboxStyle {
            checked_color: color::PRIMARY,
            disabled: true,
        },
    )
}
