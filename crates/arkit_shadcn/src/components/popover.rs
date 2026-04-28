use super::card::card;
use super::floating_layer::{floating_panel, FloatingSide};
use super::*;
use std::rc::Rc;

const POPOVER_DEFAULT_WIDTH: f32 = 288.0; // Tailwind `w-72`

fn popover<Message: 'static>(
    trigger: Element<Message>,
    content: Vec<Element<Message>>,
    open: bool,
    on_open_change: impl Fn(bool) + 'static,
) -> Element<Message> {
    popover_with_width(
        trigger,
        content,
        open,
        on_open_change,
        POPOVER_DEFAULT_WIDTH,
    )
}

fn popover_message<Message>(
    trigger: Element<Message>,
    content: Vec<Element<Message>>,
    open: bool,
    on_open_change: impl Fn(bool) -> Message + 'static,
) -> Element<Message>
where
    Message: Send + 'static,
{
    popover(trigger, content, open, move |value| {
        dispatch_message(on_open_change(value))
    })
}

fn popover_with_width<Message: 'static>(
    trigger: Element<Message>,
    content: Vec<Element<Message>>,
    open: bool,
    on_open_change: impl Fn(bool) + 'static,
    width: f32,
) -> Element<Message> {
    let dismiss = Rc::new(move || {
        on_open_change(false);
    });
    floating_panel(
        trigger,
        panel_surface(
            arkit::column_component::<Message, arkit::Theme>()
                .width(width)
                .align_items_start()
                .padding([spacing::LG, spacing::LG, spacing::LG, spacing::LG])
                .children(vec![stack(content, spacing::LG)]),
        )
        .into(),
        open,
        FloatingSide::Bottom,
        Some(dismiss),
    )
}

fn popover_with_width_message<Message>(
    trigger: Element<Message>,
    content: Vec<Element<Message>>,
    open: bool,
    on_open_change: impl Fn(bool) -> Message + 'static,
    width: f32,
) -> Element<Message>
where
    Message: Send + 'static,
{
    popover_with_width(
        trigger,
        content,
        open,
        move |value| dispatch_message(on_open_change(value)),
        width,
    )
}

fn popover_card<Message: 'static>(
    title: impl Into<String>,
    body: impl Into<String>,
) -> Element<Message> {
    card(vec![title_text(title).into(), muted_text(body).into()])
}

// Struct component API
pub struct Popover<Message = ()> {
    trigger: std::cell::RefCell<Option<Element<Message>>>,
    content: std::cell::RefCell<Option<Vec<Element<Message>>>>,
    open: Option<bool>,
    default_open: bool,
    width: Option<f32>,
    on_open_change: Option<std::rc::Rc<dyn Fn(bool) -> Message>>,
}

impl<Message> Popover<Message> {
    pub fn new(trigger: Element<Message>, content: Vec<Element<Message>>) -> Self {
        Self {
            trigger: std::cell::RefCell::new(Some(trigger)),
            content: std::cell::RefCell::new(Some(content)),
            open: None,
            default_open: false,
            width: None,
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

    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn on_open_change(mut self, handler: impl Fn(bool) -> Message + 'static) -> Self {
        self.on_open_change = Some(std::rc::Rc::new(handler));
        self
    }
}

impl<Message: Send + 'static> arkit::advanced::Widget<Message, arkit::Theme, arkit::Renderer>
    for Popover<Message>
{
    fn body(
        &self,
        tree: &mut arkit::advanced::widget::Tree,
        _renderer: &arkit::Renderer,
    ) -> Option<Element<Message>> {
        let state = super::widget_state(tree, || self.default_open);
        let is_controlled = self.open.is_some();
        let open = self.open.unwrap_or_else(|| *state.borrow());
        let handler = self.on_open_change.clone();
        let mut trigger = super::take_component_slot(&self.trigger, "popover trigger");
        if !is_controlled {
            let trigger_state = state.clone();
            let trigger_handler = handler.clone();
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

        Some(popover_with_width(
            trigger,
            super::take_component_slot(&self.content, "popover content"),
            open,
            move |value| {
                if !is_controlled {
                    *state.borrow_mut() = value;
                    super::request_widget_rerender();
                }
                if let Some(handler) = handler.as_ref() {
                    dispatch_message(handler(value));
                }
            },
            self.width.unwrap_or(POPOVER_DEFAULT_WIDTH),
        ))
    }
}

impl<Message: Send + 'static> From<Popover<Message>> for Element<Message> {
    fn from(value: Popover<Message>) -> Self {
        Element::new(value)
    }
}
