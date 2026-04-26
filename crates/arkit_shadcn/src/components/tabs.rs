use super::*;
use std::rc::Rc;

const TRANSPARENT: u32 = 0x00000000;

fn tabs<Message: 'static>(
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

fn tabs_message<Message>(
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
                .border_radius([radii().md, radii().md, radii().md, radii().md])
                .border_width([1.0, 1.0, 1.0, 1.0])
                .border_color(TRANSPARENT)
                .clear_shadow()
                .background_color(if is_active {
                    colors().background
                } else {
                    TRANSPARENT
                })
                .on_click(move || on_change(index))
                .children(vec![body_text::<Message>(label)
                    .font_color(colors().foreground)
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

// Struct component API
pub struct Tabs<Message = ()> {
    labels: Vec<String>,
    panels: std::cell::RefCell<Option<Vec<Element<Message>>>>,
    active: Option<usize>,
    default_active: usize,
    on_change: Option<std::rc::Rc<dyn Fn(usize) -> Message>>,
}

impl<Message> Tabs<Message> {
    pub fn new(labels: Vec<String>, panels: Vec<Element<Message>>) -> Self {
        Self {
            labels,
            panels: std::cell::RefCell::new(Some(panels)),
            active: None,
            default_active: 0,
            on_change: None,
        }
    }

    pub fn active(mut self, active: usize) -> Self {
        self.active = Some(active);
        self
    }

    pub fn default_active(mut self, active: usize) -> Self {
        self.default_active = active;
        self
    }

    pub fn on_change(mut self, handler: impl Fn(usize) -> Message + 'static) -> Self {
        self.on_change = Some(std::rc::Rc::new(handler));
        self
    }
}

impl<Message: Send + 'static> arkit::advanced::Widget<Message, arkit::Theme, arkit::Renderer>
    for Tabs<Message>
{
    fn body(
        &self,
        tree: &mut arkit::advanced::widget::Tree,
        _renderer: &arkit::Renderer,
    ) -> Option<Element<Message>> {
        let state = super::widget_state(tree, || self.default_active);
        let is_controlled = self.active.is_some();
        let active = self.active.unwrap_or_else(|| *state.borrow());
        let on_change = self.on_change.clone();
        Some(tabs(
            self.labels.clone(),
            active,
            move |value| {
                if !is_controlled {
                    *state.borrow_mut() = value;
                    super::request_widget_rerender();
                }
                if let Some(on_change) = on_change.as_ref() {
                    dispatch_message(on_change(value));
                }
            },
            super::take_component_slot(&self.panels, "tabs panels"),
        ))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn into_any(self: Box<Self>) -> Box<dyn std::any::Any> {
        self
    }
}

impl<Message: Send + 'static> From<Tabs<Message>> for Element<Message> {
    fn from(value: Tabs<Message>) -> Self {
        Element::new(value)
    }
}
