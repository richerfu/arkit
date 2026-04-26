use super::floating_layer::floating_panel;
use super::*;
use std::rc::Rc;

fn tooltip<Message: 'static>(
    trigger: Element<Message>,
    content: impl Into<String> + 'static,
    open: bool,
    on_open_change: impl Fn(bool) + 'static,
) -> Element<Message> {
    let content = content.into();
    let dismiss = { Rc::new(move || on_open_change(false)) };

    floating_panel(
        trigger,
        arkit::row_component::<Message, arkit::Theme>()
            .padding([8.0, 12.0, 8.0, 12.0])
            .border_radius([radii().md, radii().md, radii().md, radii().md])
            .background_color(colors().primary)
            .children(vec![arkit::text::<Message, arkit::Theme>(content)
                .font_size(typography::XS)
                .font_color(colors().primary_foreground)
                .line_height(16.0)
                .into()])
            .into(),
        open,
        super::floating_layer::FloatingSide::Top,
        Some(dismiss),
    )
}

fn tooltip_message<Message>(
    trigger: Element<Message>,
    content: impl Into<String> + 'static,
    open: bool,
    on_open_change: impl Fn(bool) -> Message + 'static,
) -> Element<Message>
where
    Message: Send + 'static,
{
    tooltip(trigger, content, open, move |value| {
        dispatch_message(on_open_change(value))
    })
}

// Struct component API
pub struct Tooltip<Message = ()> {
    trigger: std::cell::RefCell<Option<Element<Message>>>,
    content: String,
    open: Option<bool>,
    default_open: bool,
    on_open_change: Option<std::rc::Rc<dyn Fn(bool) -> Message>>,
}

impl<Message> Tooltip<Message> {
    pub fn new(trigger: Element<Message>, content: impl Into<String>) -> Self {
        Self {
            trigger: std::cell::RefCell::new(Some(trigger)),
            content: content.into(),
            open: None,
            default_open: false,
            on_open_change: None,
        }
    }

    pub fn open(mut self, open: bool) -> Self {
        self.open = Some(open);
        self
    }

    pub fn default_open(mut self, open: bool) -> Self {
        self.default_open = open;
        self
    }

    pub fn on_open_change(mut self, handler: impl Fn(bool) -> Message + 'static) -> Self {
        self.on_open_change = Some(std::rc::Rc::new(handler));
        self
    }
}

impl<Message: Send + 'static> arkit::advanced::Widget<Message, arkit::Theme, arkit::Renderer>
    for Tooltip<Message>
{
    fn body(
        &self,
        tree: &mut arkit::advanced::widget::Tree,
        _renderer: &arkit::Renderer,
    ) -> Option<Element<Message>> {
        let state = super::widget_state(tree, || self.default_open);
        let is_controlled = self.open.is_some();
        let open = self.open.unwrap_or_else(|| *state.borrow());
        let on_open_change = self.on_open_change.clone();
        let mut trigger = super::take_component_slot(&self.trigger, "tooltip trigger");
        if !is_controlled {
            let trigger_state = state.clone();
            let trigger_handler = on_open_change.clone();
            trigger = arkit::row_component::<Message, arkit::Theme>()
                .on_click(move || {
                    let next = !open;
                    *trigger_state.borrow_mut() = next;
                    super::request_widget_rerender();
                    if let Some(handler) = trigger_handler.as_ref() {
                        dispatch_message(handler(next));
                    }
                })
                .children(vec![trigger])
                .into();
        }

        Some(tooltip(trigger, self.content.clone(), open, move |value| {
            if !is_controlled {
                *state.borrow_mut() = value;
                super::request_widget_rerender();
            }
            if let Some(on_open_change) = on_open_change.as_ref() {
                dispatch_message(on_open_change(value));
            }
        }))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn into_any(self: Box<Self>) -> Box<dyn std::any::Any> {
        self
    }
}

impl<Message: Send + 'static> From<Tooltip<Message>> for Element<Message> {
    fn from(value: Tooltip<Message>) -> Self {
        Element::new(value)
    }
}
