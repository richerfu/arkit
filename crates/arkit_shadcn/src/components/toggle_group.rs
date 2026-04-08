use super::*;
use arkit::ohos_arkui_binding::types::alignment::Alignment;
use arkit::ohos_arkui_binding::types::text_alignment::TextAlignment;
use arkit_icon as lucide;

fn toggle_group_border(index: usize) -> [f32; 4] {
    [1.0, 1.0, 1.0, if index == 0 { 1.0 } else { 0.0 }]
}

fn toggle_group_radius(index: usize, total: usize) -> [f32; 4] {
    let left_radius = if index == 0 { radius::MD } else { 0.0 };
    let right_radius = if index + 1 == total { radius::MD } else { 0.0 };
    [left_radius, right_radius, right_radius, left_radius]
}

fn toggle_group_shell<Message: 'static>(children: Vec<Element<Message>>) -> Element<Message> {
    shadow_sm(
        arkit::row_component::<Message, arkit::Theme>()
            .align_items_center()
            .style(ArkUINodeAttributeType::Clip, true)
            .style(
                ArkUINodeAttributeType::BorderRadius,
                vec![radius::MD, radius::MD, radius::MD, radius::MD],
            )
            .children(children),
    )
    .into()
}

fn toggle_group_item_surface<Message>(
    element: ButtonElement<Message>,
    active: bool,
    border_width: [f32; 4],
    border_radius: [f32; 4],
) -> ButtonElement<Message> {
    element
        .style(ArkUINodeAttributeType::Clip, true)
        .style(ArkUINodeAttributeType::BorderStyle, 0_i32)
        .style(ArkUINodeAttributeType::AlignSelf, 1_i32)
        .style(ArkUINodeAttributeType::BorderRadius, border_radius.to_vec())
        .style(ArkUINodeAttributeType::BorderWidth, border_width.to_vec())
        .style(ArkUINodeAttributeType::BorderColor, vec![color::INPUT])
        .style(
            ArkUINodeAttributeType::Alignment,
            i32::from(Alignment::Center),
        )
        .patch_background_color(if active {
            color::ACCENT
        } else {
            color::BACKGROUND
        })
}

pub fn toggle_group(
    options: Vec<String>,
    selected: impl Into<String>,
    on_select: impl Fn(String) + 'static,
) -> Element {
    let selected = selected.into();
    let on_select = std::rc::Rc::new(on_select);
    let total = options.len();
    let children = options
        .into_iter()
        .enumerate()
        .map(|(index, item)| {
            let text = item.clone();
            let active = selected == text;
            let on_select = on_select.clone();

            toggle_group_item_surface(
                normal_button(text.clone())
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
                toggle_group_border(index),
                toggle_group_radius(index, total),
            )
            .on_click(move || on_select(text.clone()))
            .into()
        })
        .collect::<Vec<_>>();

    toggle_group_shell(children)
}

pub fn toggle_group_message<Message>(
    options: Vec<String>,
    selected: impl Into<String>,
    on_select: impl Fn(String) -> Message + 'static,
) -> Element
where
    Message: Send + 'static,
{
    toggle_group(options, selected, move |value| {
        dispatch_message(on_select(value))
    })
}

pub fn toggle_group_icons(
    options: Vec<String>,
    selected: impl Into<String>,
    on_select: impl Fn(String) + 'static,
) -> Element {
    let selected = selected.into();
    let on_select = std::rc::Rc::new(on_select);
    let total = options.len();
    let children = options
        .into_iter()
        .enumerate()
        .map(|(index, item)| {
            let icon_name = item.clone();
            let active = selected == icon_name;
            let on_select = on_select.clone();

            toggle_group_item_surface(
                normal_button_component()
                    .width(40.0)
                    .height(40.0)
                    .style(ArkUINodeAttributeType::Padding, vec![0.0, 0.0, 0.0, 0.0])
                    .children(vec![lucide::icon(icon_name.clone())
                        .size(16.0)
                        .color(if active {
                            color::ACCENT_FOREGROUND
                        } else {
                            color::FOREGROUND
                        })
                        .render()]),
                active,
                toggle_group_border(index),
                toggle_group_radius(index, total),
            )
            .on_click(move || on_select(icon_name.clone()))
            .into()
        })
        .collect::<Vec<_>>();

    toggle_group_shell(children)
}

pub fn toggle_group_icons_message<Message>(
    options: Vec<String>,
    selected: impl Into<String>,
    on_select: impl Fn(String) -> Message + 'static,
) -> Element
where
    Message: Send + 'static,
{
    toggle_group_icons(options, selected, move |value| {
        dispatch_message(on_select(value))
    })
}

pub fn toggle_group_multi(
    options: Vec<String>,
    selected: Vec<String>,
    on_change: impl Fn(Vec<String>) + 'static,
) -> Element {
    let on_change = std::rc::Rc::new(on_change);
    let total = options.len();
    let children = options
        .into_iter()
        .enumerate()
        .map(|(index, item)| {
            let text = item.clone();
            let active = selected.contains(&text);
            let on_change = on_change.clone();
            let selected_values = selected.clone();

            toggle_group_item_surface(
                normal_button(text.clone())
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
                toggle_group_border(index),
                toggle_group_radius(index, total),
            )
            .on_click(move || {
                let mut next = selected_values.clone();
                if let Some(pos) = next.iter().position(|value| value == &text) {
                    next.remove(pos);
                } else {
                    next.push(text.clone());
                }
                on_change(next);
            })
            .into()
        })
        .collect::<Vec<_>>();

    toggle_group_shell(children)
}

pub fn toggle_group_multi_message<Message>(
    options: Vec<String>,
    selected: Vec<String>,
    on_change: impl Fn(Vec<String>) -> Message + 'static,
) -> Element
where
    Message: Send + 'static,
{
    toggle_group_multi(options, selected, move |value| {
        dispatch_message(on_change(value))
    })
}

pub fn toggle_group_icons_multi<Message: Send + 'static>(
    options: Vec<String>,
    selected: Vec<String>,
    on_change: impl Fn(Vec<String>) + 'static,
) -> Element<Message> {
    let on_change = std::rc::Rc::new(on_change);
    let total = options.len();
    let children = options
        .into_iter()
        .enumerate()
        .map(|(index, item)| {
            let icon_name = item.clone();
            let active = selected.contains(&icon_name);
            let on_change = on_change.clone();
            let selected_values = selected.clone();

            toggle_group_item_surface(
                normal_button_component::<Message, arkit::Theme>()
                    .width(40.0)
                    .height(40.0)
                    .style(ArkUINodeAttributeType::Padding, vec![0.0, 0.0, 0.0, 0.0])
                    .children(vec![lucide::icon(icon_name.clone())
                        .size(16.0)
                        .color(if active {
                            color::ACCENT_FOREGROUND
                        } else {
                            color::FOREGROUND
                        })
                        .render::<Message, arkit::Theme>()]),
                active,
                toggle_group_border(index),
                toggle_group_radius(index, total),
            )
            .on_click(move || {
                let mut next = selected_values.clone();
                if let Some(pos) = next.iter().position(|value| value == &icon_name) {
                    next.remove(pos);
                } else {
                    next.push(icon_name.clone());
                }
                on_change(next);
            })
            .into()
        })
        .collect::<Vec<_>>();

    toggle_group_shell::<Message>(children)
}

pub fn toggle_group_icons_multi_message<Message>(
    options: Vec<String>,
    selected: Vec<String>,
    on_change: impl Fn(Vec<String>) -> Message + 'static,
) -> Element<Message>
where
    Message: Send + 'static,
{
    toggle_group_icons_multi(options, selected, move |value| {
        dispatch_message(on_change(value))
    })
}
