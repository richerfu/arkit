use super::*;

fn switch<Message: 'static>(state: bool) -> ToggleElement<Message> {
    shadow_sm(
        arkit::toggle_component::<Message, arkit::Theme>()
            .checked(state)
            .toggle_selected_color(colors().primary)
            .toggle_unselected_color(colors().input)
            .toggle_switch_point_color(colors().background)
            .border_style(BorderStyle::Solid)
            // RN: `border border-transparent shadow-sm`.
            .border_width([1.0, 1.0, 1.0, 1.0])
            .border_color(0x00000000)
            .border_radius([radii().full, radii().full, radii().full, radii().full])
            .clip(true)
            .width(32.0)
            .height(18.4),
    )
}

// Struct component API
pub struct Switch<Message = ()> {
    checked: Option<bool>,
    default_checked: bool,
    on_change: Option<std::rc::Rc<dyn Fn(bool) -> Message>>,
}

impl<Message> Switch<Message> {
    pub fn new(checked: bool) -> Self {
        Self {
            checked: Some(checked),
            default_checked: checked,
            on_change: None,
        }
    }

    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = Some(checked);
        self
    }

    pub fn default_checked(mut self, checked: bool) -> Self {
        self.checked = None;
        self.default_checked = checked;
        self
    }

    pub fn on_change(mut self, handler: impl Fn(bool) -> Message + 'static) -> Self {
        self.on_change = Some(std::rc::Rc::new(handler));
        self
    }

    pub fn on_toggle(self, handler: impl Fn(bool) -> Message + 'static) -> Self {
        self.on_change(handler)
    }
}

impl<Message: Send + 'static> arkit::advanced::Widget<Message, arkit::Theme, arkit::Renderer>
    for Switch<Message>
{
    fn body(
        &self,
        tree: &mut arkit::advanced::widget::Tree,
        _renderer: &arkit::Renderer,
    ) -> Option<Element<Message>> {
        let state = super::widget_state(tree, || self.default_checked);
        let is_controlled = self.checked.is_some();
        let checked = self.checked.unwrap_or_else(|| *state.borrow());
        let handler = self.on_change.clone();

        let element = switch::<Message>(checked).on_click(move || {
            let next = !checked;
            if !is_controlled {
                *state.borrow_mut() = next;
                super::request_widget_rerender();
            }
            if let Some(handler) = handler.as_ref() {
                dispatch_message(handler(next));
            }
        });
        Some(element.into())
    }
}

impl<Message: Send + 'static> From<Switch<Message>> for Element<Message> {
    fn from(value: Switch<Message>) -> Self {
        Element::new(value)
    }
}
