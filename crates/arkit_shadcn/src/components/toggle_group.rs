use super::toggle::{
    toggle_content_row, toggle_default_size, toggle_icon_size, toggle_surface, toggle_visual_style,
    ToggleSizeStyle, ToggleVariant,
};
use super::*;

const TOGGLE_GROUP_VARIANT: ToggleVariant = ToggleVariant::Outline;

fn toggle_group_border(index: usize) -> [f32; 4] {
    [1.0, 1.0, 1.0, if index == 0 { 1.0 } else { 0.0 }]
}

fn toggle_group_radius(index: usize, total: usize) -> [f32; 4] {
    let left_radius = if index == 0 { radius::MD } else { 0.0 };
    let right_radius = if index + 1 == total { radius::MD } else { 0.0 };
    [left_radius, right_radius, left_radius, right_radius]
}

fn toggle_group_shell<Message: 'static>(children: Vec<Element<Message>>) -> Element<Message> {
    shadow_sm(
        arkit::row_component::<Message, arkit::Theme>()
            .align_items_center()
            .border_radius([radius::MD, radius::MD, radius::MD, radius::MD])
            .clip(true)
            .children(children),
    )
    .into()
}

fn toggle_group_item<Message: 'static>(
    content: Element<Message>,
    active: bool,
    index: usize,
    total: usize,
    size_style: ToggleSizeStyle,
) -> ButtonElement<Message> {
    let border_width = toggle_group_border(index);
    let border_radius = toggle_group_radius(index, total);

    toggle_surface(
        content,
        active,
        TOGGLE_GROUP_VARIANT,
        size_style,
        border_width,
        border_radius,
        Some(false),
    )
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
            let size_style = toggle_default_size();
            let visual = toggle_visual_style(TOGGLE_GROUP_VARIANT, active);

            toggle_group_item(
                toggle_content_row(
                    Some(text.clone()),
                    None,
                    visual.foreground,
                    size_style.icon_size,
                ),
                active,
                index,
                total,
                size_style,
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
            let size_style = toggle_icon_size();
            let visual = toggle_visual_style(TOGGLE_GROUP_VARIANT, active);

            toggle_group_item(
                toggle_content_row(
                    None,
                    Some(icon_name.clone()),
                    visual.foreground,
                    size_style.icon_size,
                ),
                active,
                index,
                total,
                size_style,
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
            let size_style = toggle_default_size();
            let visual = toggle_visual_style(TOGGLE_GROUP_VARIANT, active);

            toggle_group_item(
                toggle_content_row(
                    Some(text.clone()),
                    None,
                    visual.foreground,
                    size_style.icon_size,
                ),
                active,
                index,
                total,
                size_style,
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
            let size_style = toggle_icon_size();
            let visual = toggle_visual_style(TOGGLE_GROUP_VARIANT, active);

            toggle_group_item(
                toggle_content_row(
                    None,
                    Some(icon_name.clone()),
                    visual.foreground,
                    size_style.icon_size,
                ),
                active,
                index,
                total,
                size_style,
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
