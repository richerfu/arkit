use super::*;
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

fn toggle_group_shell(children: Vec<Element>) -> Element {
    shadow_sm(
        arkit::row_component()
            .style(ArkUINodeAttributeType::RowAlignItems, FLEX_ALIGN_CENTER)
            .style(ArkUINodeAttributeType::Clip, true)
            .style(
                ArkUINodeAttributeType::BorderRadius,
                vec![radius::MD, radius::MD, radius::MD, radius::MD],
            )
            .children(children),
    )
    .into()
}

fn toggle_group_item_surface(
    element: ButtonElement,
    active: bool,
    border_width: [f32; 4],
    border_radius: [f32; 4],
) -> ButtonElement {
    element
        .style(ArkUINodeAttributeType::ButtonType, 0_i32)
        .style(ArkUINodeAttributeType::Clip, true)
        .style(ArkUINodeAttributeType::AlignSelf, 1_i32)
        .style(ArkUINodeAttributeType::BorderStyle, 0_i32)
        .style(ArkUINodeAttributeType::BorderRadius, border_radius.to_vec())
        .style(ArkUINodeAttributeType::BorderWidth, border_width.to_vec())
        .style(ArkUINodeAttributeType::BorderColor, vec![color::INPUT])
        .style(ArkUINodeAttributeType::RowAlignItems, FLEX_ALIGN_CENTER)
        .style(ArkUINodeAttributeType::RowJustifyContent, FLEX_ALIGN_CENTER)
        .background_color(if active {
            color::ACCENT
        } else {
            color::BACKGROUND
        })
}

pub fn toggle_group(options: Vec<String>, selected: Signal<String>) -> Element {
    let total = options.len();
    let children = options
        .into_iter()
        .enumerate()
        .map(|(index, item)| {
            let active = selected.get() == item;
            let value = selected.clone();
            let text = item.clone();

            toggle_group_item_surface(
                arkit::button(item)
                    .height(40.0)
                    .style(
                        ArkUINodeAttributeType::Padding,
                        vec![8.0, 10.0, 8.0, 10.0],
                    )
                    .font_size(typography::SM)
                    .style(ArkUINodeAttributeType::FontWeight, 4_i32)
                    .style(
                        ArkUINodeAttributeType::TextAlign,
                        i32::from(TextAlignment::Center),
                    )
                    .style(
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
            .on_click(move || value.set(text.clone()))
            .into()
        })
        .collect::<Vec<_>>();

    toggle_group_shell(children)
}

pub fn toggle_group_icons(options: Vec<String>, selected: Signal<String>) -> Element {
    let total = options.len();
    let children = options
        .into_iter()
        .enumerate()
        .map(|(index, item)| {
            let active = selected.get() == item;
            let value = selected.clone();
            let icon_name = item.clone();

            toggle_group_item_surface(
                arkit::button_component()
                    .width(40.0)
                    .height(40.0)
                    .style(ArkUINodeAttributeType::Padding, vec![0.0, 0.0, 0.0, 0.0])
                    .children(vec![lucide::icon(item)
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
            .on_click(move || value.set(icon_name.clone()))
            .into()
        })
        .collect::<Vec<_>>();

    toggle_group_shell(children)
}

pub fn toggle_group_multi(options: Vec<String>, selected: Signal<Vec<String>>) -> Element {
    let total = options.len();
    let children = options
        .into_iter()
        .enumerate()
        .map(|(index, item)| {
            let active = selected.get().contains(&item);
            let values = selected.clone();
            let text = item.clone();

            toggle_group_item_surface(
                arkit::button(item)
                    .height(40.0)
                    .style(
                        ArkUINodeAttributeType::Padding,
                        vec![8.0, 10.0, 8.0, 10.0],
                    )
                    .font_size(typography::SM)
                    .style(ArkUINodeAttributeType::FontWeight, 4_i32)
                    .style(
                        ArkUINodeAttributeType::TextAlign,
                        i32::from(TextAlignment::Center),
                    )
                    .style(
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
                values.update(|items| {
                    if let Some(pos) = items.iter().position(|value| value == &text) {
                        items.remove(pos);
                    } else {
                        items.push(text.clone());
                    }
                });
            })
            .into()
        })
        .collect::<Vec<_>>();

    toggle_group_shell(children)
}

pub fn toggle_group_icons_multi(options: Vec<String>, selected: Signal<Vec<String>>) -> Element {
    let total = options.len();
    let children = options
        .into_iter()
        .enumerate()
        .map(|(index, item)| {
            let active = selected.get().contains(&item);
            let values = selected.clone();
            let icon_name = item.clone();

            toggle_group_item_surface(
                arkit::button_component()
                    .width(40.0)
                    .height(40.0)
                    .style(ArkUINodeAttributeType::Padding, vec![0.0, 0.0, 0.0, 0.0])
                    .children(vec![lucide::icon(item)
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
            .on_click(move || {
                values.update(|items| {
                    if let Some(pos) = items.iter().position(|value| value == &icon_name) {
                        items.remove(pos);
                    } else {
                        items.push(icon_name.clone());
                    }
                });
            })
            .into()
        })
        .collect::<Vec<_>>();

    toggle_group_shell(children)
}
