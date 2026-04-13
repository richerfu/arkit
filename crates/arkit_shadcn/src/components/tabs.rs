use super::*;
use std::rc::Rc;

const TRANSPARENT: u32 = 0x00000000;

pub fn tabs<Message: 'static>(
    tab_labels: Vec<String>,
    active: usize,
    on_change: impl Fn(usize) + 'static,
    panels: Vec<Element<Message>>,
) -> Element<Message> {
    arkit::column_component::<Message, arkit::Theme>()
        .width(arkit::Length::Fill)
        .children(vec![
            tabs_list(tab_labels, active, Rc::new(on_change)),
            arkit::row_component::<Message, arkit::Theme>()
                .margin([spacing::SM, 0.0, 0.0, 0.0])
                .children(vec![tabs_content(panels, active)])
                .into(),
        ])
        .into()
}

pub fn tabs_message<Message>(
    tab_labels: Vec<String>,
    active: usize,
    on_change: impl Fn(usize) -> Message + 'static,
    panels: Vec<Element<Message>>,
) -> Element<Message>
where
    Message: Send + 'static,
{
    tabs(
        tab_labels,
        active,
        move |value| dispatch_message(on_change(value)),
        panels,
    )
}

fn tabs_list<Message: 'static>(
    tab_labels: Vec<String>,
    active: usize,
    on_change: Rc<dyn Fn(usize)>,
) -> Element<Message> {
    let children = tab_labels
        .into_iter()
        .enumerate()
        .map(|(index, label)| {
            let is_active = active == index;
            let on_change = on_change.clone();
            arkit::row_component::<Message, arkit::Theme>()
                .height(TABS_TRIGGER_HEIGHT)
                .align_items_center()
                .justify_content_center()
                .padding([spacing::XXS, spacing::SM, spacing::XXS, spacing::SM])
                .border_radius([radius::MD, radius::MD, radius::MD, radius::MD])
                .border_width([1.0, 1.0, 1.0, 1.0])
                .border_color(TRANSPARENT)
                .clear_shadow()
                .background_color(if is_active {
                    color::BACKGROUND
                } else {
                    TRANSPARENT
                })
                .on_click(move || on_change(index))
                .children(vec![body_text::<Message>(label)
                    .font_color(color::FOREGROUND)
                    .into()])
                .into()
        })
        .collect::<Vec<_>>();

    rounded_tabs_list_surface::<Message>(
        arkit::row_component::<Message, arkit::Theme>()
            .align_items_center()
            .children(children),
    )
    .into()
}

fn tabs_content<Message: 'static>(
    panels: Vec<Element<Message>>,
    active: usize,
) -> Element<Message> {
    let panel_containers: Vec<Element<Message>> = panels
        .into_iter()
        .enumerate()
        .map(|(index, panel)| {
            let is_active = active == index;
            arkit::column_component::<Message, arkit::Theme>()
                .width(arkit::Length::Fill)
                .visibility(if is_active {
                    Visibility::Visible
                } else {
                    Visibility::None
                })
                .children(vec![panel])
                .into()
        })
        .collect();

    arkit::stack_component::<Message, arkit::Theme>()
        .width(arkit::Length::Fill)
        .children(panel_containers)
        .into()
}
