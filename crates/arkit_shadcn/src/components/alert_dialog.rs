use super::*;
use std::rc::Rc;

fn alert_dialog(
    title: impl Into<String>,
    description: impl Into<String>,
    actions: Vec<Element>,
) -> Element {
    alert_dialog_with_message::<()>(title, description, actions)
}

fn alert_dialog_with_message<Message: 'static>(
    title: impl Into<String>,
    description: impl Into<String>,
    actions: Vec<Element<Message>>,
) -> Element<Message> {
    shadow_sm(
        arkit::column_component::<Message, arkit::Theme>()
            .percent_width(1.0)
            .max_width_constraint(super::dialog::DIALOG_MAX_WIDTH)
            .padding([spacing::XXL, spacing::XXL, spacing::XXL, spacing::XXL])
            .border_radius([radii().lg, radii().lg, radii().lg, radii().lg])
            .border_width([1.0, 1.0, 1.0, 1.0])
            .border_color(colors().border)
            .background_color(colors().background)
            .children(vec![stack(
                vec![
                    super::dialog::dialog_header(title, description),
                    super::dialog::dialog_footer(actions),
                ],
                spacing::LG,
            )]),
    )
    .into()
}

fn alert_dialog_modal_message<Message>(
    open: bool,
    on_open_change: impl Fn(bool) -> Message + 'static,
    title: impl Into<String>,
    description: impl Into<String>,
    actions: Vec<Element<Message>>,
) -> Element<Message>
where
    Message: Send + 'static,
{
    super::dialog::modal_overlay(
        open,
        alert_dialog_with_message(title, description, actions),
        Some(Rc::new(move || dispatch_message(on_open_change(false)))),
    )
}

fn alert_dialog_actions<Message: 'static>(actions: Vec<Element<Message>>) -> Element<Message> {
    super::dialog::dialog_footer(actions)
}

// Struct component API
pub struct AlertDialog<Message = ()> {
    title: String,
    description: String,
    open: Option<bool>,
    default_open: bool,
    on_open_change: Option<std::rc::Rc<dyn Fn(bool) -> Message>>,
    actions: std::cell::RefCell<Option<Vec<Element<Message>>>>,
}

impl<Message> AlertDialog<Message> {
    pub fn new(
        title: impl Into<String>,
        description: impl Into<String>,
        actions: Vec<Element<Message>>,
    ) -> Self {
        Self {
            title: title.into(),
            description: description.into(),
            open: None,
            default_open: false,
            on_open_change: None,
            actions: std::cell::RefCell::new(Some(actions)),
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
    for AlertDialog<Message>
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
        Some(super::dialog::modal_overlay(
            open,
            alert_dialog_with_message(
                self.title.clone(),
                self.description.clone(),
                super::take_component_slot(&self.actions, "alert dialog actions"),
            ),
            Some(Rc::new(move || {
                if !is_controlled {
                    *state.borrow_mut() = false;
                    super::request_widget_rerender();
                }
                if let Some(handler) = handler.as_ref() {
                    dispatch_message(handler(false));
                }
            })),
        ))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn into_any(self: Box<Self>) -> Box<dyn std::any::Any> {
        self
    }
}

impl<Message: Send + 'static> From<AlertDialog<Message>> for Element<Message> {
    fn from(value: AlertDialog<Message>) -> Self {
        Element::new(value)
    }
}
