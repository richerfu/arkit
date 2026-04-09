use super::*;
use arkit_icon as lucide;
use std::rc::Rc;

const CHECKBOX_SIZE: f32 = 16.0;
const CHECKBOX_BORDER_WIDTH: f32 = 1.0;
const CHECKBOX_ICON_SIZE: f32 = 12.0;
const CHECKBOX_ICON_STROKE_WIDTH: f32 = 3.5;

#[derive(Debug, Clone, Copy)]
struct CheckboxStyle {
    checked_color: u32,
    disabled: bool,
}

fn checkbox_indicator<Message: 'static>(checked: bool, style: CheckboxStyle) -> Element<Message> {
    let mut indicator = shadow_sm(
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
            .style(
                ArkUINodeAttributeType::BorderColor,
                vec![if checked {
                    style.checked_color
                } else {
                    color::INPUT
                }],
            )
            .style(ArkUINodeAttributeType::Clip, true)
            .background_color(if checked {
                style.checked_color
            } else {
                color::BACKGROUND
            }),
    );

    if checked {
        indicator = indicator.children(vec![lucide::icon("check")
            .size(CHECKBOX_ICON_SIZE)
            .stroke_width(CHECKBOX_ICON_STROKE_WIDTH)
            .color(color::PRIMARY_FOREGROUND)
            .render::<Message, arkit::Theme>()]);
    }

    indicator.into()
}

fn checkbox_impl<Message: 'static>(
    label_text: String,
    checked: bool,
    on_toggle: Option<Rc<dyn Fn(bool)>>,
    style: CheckboxStyle,
) -> Element<Message> {
    let mut children = vec![checkbox_indicator::<Message>(checked, style)];

    if !label_text.is_empty() {
        children.push(
            arkit::row_component::<Message, arkit::Theme>()
                .align_items_center()
                .style(
                    ArkUINodeAttributeType::Margin,
                    vec![0.0, 0.0, 0.0, spacing::SM],
                )
                .children(vec![label(label_text).into()])
                .into(),
        );
    }

    let mut root = arkit::row_component::<Message, arkit::Theme>()
        .align_items_center()
        .children(children);

    if let Some(on_toggle) = on_toggle {
        if !style.disabled {
            root = root.on_click(move || on_toggle(!checked));
        }
    }

    if style.disabled {
        root = root.style(ArkUINodeAttributeType::Opacity, 0.5_f32);
    }

    root.into()
}

pub fn checkbox_message<Message>(
    label: impl Into<String>,
    checked: bool,
    on_toggle: impl Fn(bool) -> Message + 'static,
) -> Element<Message>
where
    Message: Send + 'static,
{
    checkbox_impl(
        label.into(),
        checked,
        Some(Rc::new(move |value| dispatch_message(on_toggle(value)))),
        CheckboxStyle {
            checked_color: color::PRIMARY,
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
    checkbox_impl(
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
    checkbox_impl(
        label.into(),
        checked,
        None,
        CheckboxStyle {
            checked_color: color::PRIMARY,
            disabled: true,
        },
    )
}
