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
    let left_radius = if index == 0 { radii().md } else { 0.0 };
    let right_radius = if index + 1 == total { radii().md } else { 0.0 };
    [left_radius, right_radius, left_radius, right_radius]
}

fn toggle_group_shell<Message: 'static>(children: Vec<Element<Message>>) -> Element<Message> {
    shadow_sm(
        arkit::row_component::<Message, arkit::Theme>()
            .align_items_center()
            .border_radius([radii().md, radii().md, radii().md, radii().md])
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

fn toggle_group(
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

fn toggle_group_message<Message>(
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

fn toggle_group_icons(
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

fn toggle_group_icons_message<Message>(
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

fn toggle_group_multi(
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

fn toggle_group_multi_message<Message>(
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

fn toggle_group_icons_multi<Message: Send + 'static>(
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

fn toggle_group_icons_multi_message<Message>(
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

// Struct component API
pub struct ToggleGroup<Message = ()> {
    options: Vec<String>,
    selected: Option<Vec<String>>,
    default_selected: Vec<String>,
    icons: bool,
    multi: bool,
    on_change: Option<std::rc::Rc<dyn Fn(Vec<String>) -> Message>>,
}

impl<Message> ToggleGroup<Message> {
    pub fn new(options: Vec<String>) -> Self {
        Self {
            options,
            selected: None,
            default_selected: Vec::new(),
            icons: false,
            multi: false,
            on_change: None,
        }
    }

    pub fn icons(mut self, icons: bool) -> Self {
        self.icons = icons;
        self
    }

    pub fn multi(mut self, multi: bool) -> Self {
        self.multi = multi;
        self
    }

    pub fn selected(mut self, selected: Vec<String>) -> Self {
        self.selected = Some(selected);
        self
    }

    pub fn default_selected(mut self, selected: Vec<String>) -> Self {
        self.default_selected = selected;
        self
    }

    pub fn on_change(mut self, handler: impl Fn(Vec<String>) -> Message + 'static) -> Self {
        self.on_change = Some(std::rc::Rc::new(handler));
        self
    }
}

impl<Message: Send + 'static> arkit::advanced::Widget<Message, arkit::Theme, arkit::Renderer>
    for ToggleGroup<Message>
{
    fn body(
        &self,
        tree: &mut arkit::advanced::widget::Tree,
        _renderer: &arkit::Renderer,
    ) -> Option<Element<Message>> {
        let state = super::widget_state(tree, || self.default_selected.clone());
        let is_controlled = self.selected.is_some();
        let selected = self
            .selected
            .clone()
            .unwrap_or_else(|| state.borrow().clone());
        let on_change = self.on_change.clone();
        Some(toggle_group_icons_multi(
            self.options.clone(),
            selected,
            move |value| {
                if !is_controlled {
                    *state.borrow_mut() = value.clone();
                    super::request_widget_rerender();
                }
                if let Some(on_change) = on_change.as_ref() {
                    dispatch_message(on_change(value));
                }
            },
        ))
    }
}

impl<Message: Send + 'static> From<ToggleGroup<Message>> for Element<Message> {
    fn from(value: ToggleGroup<Message>) -> Self {
        Element::new(value)
    }
}
