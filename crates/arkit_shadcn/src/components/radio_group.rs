use super::label::label;
use super::*;
use std::rc::Rc;

const RADIO_SIZE: f32 = 16.0;
const RADIO_DOT_SIZE: f32 = 8.0;
const RADIO_BORDER_WIDTH: f32 = 1.0;

fn radio_indicator<Message: 'static>(checked: bool) -> Element<Message> {
    let mut indicator = shadow_sm(
        arkit::row_component::<Message, arkit::Theme>()
            .width(RADIO_SIZE)
            .height(RADIO_SIZE)
            .align_items_center()
            .justify_content_center()
            .border_radius([radii().full, radii().full, radii().full, radii().full])
            .border_width([
                RADIO_BORDER_WIDTH,
                RADIO_BORDER_WIDTH,
                RADIO_BORDER_WIDTH,
                RADIO_BORDER_WIDTH,
            ])
            .border_color(if checked {
                colors().primary
            } else {
                colors().input
            })
            .background_color(colors().background),
    );

    if checked {
        indicator = indicator.children(vec![arkit::row_component::<Message, arkit::Theme>()
            .width(RADIO_DOT_SIZE)
            .height(RADIO_DOT_SIZE)
            .border_radius([radii().full, radii().full, radii().full, radii().full])
            .background_color(colors().primary)
            .into()]);
    }

    indicator.into()
}

fn radio_group_impl<Message: 'static>(
    options: Vec<String>,
    selected: impl Into<String>,
    on_select: impl Fn(String) + 'static,
) -> Element<Message> {
    let selected = selected.into();
    let on_select = Rc::new(on_select);
    let children = options
        .into_iter()
        .enumerate()
        .map(|(index, option)| {
            let selected_value = selected.clone();
            let click_value = option.clone();
            let on_select = on_select.clone();
            let row = arkit::row_component::<Message, arkit::Theme>()
                .percent_width(1.0)
                .align_items_center()
                .on_click(move || on_select(click_value.clone()))
                .children(vec![
                    radio_indicator::<Message>(selected_value == option),
                    arkit::row_component::<Message, arkit::Theme>()
                        .margin([0.0, 0.0, 0.0, spacing::MD])
                        .children(vec![label::<Message>(option).into()])
                        .into(),
                ]);

            if index == 0 {
                row.into()
            } else {
                margin_top(row, spacing::MD).into()
            }
        })
        .collect::<Vec<Element<Message>>>();

    arkit::column_component::<Message, arkit::Theme>()
        .percent_width(1.0)
        .children(children)
        .into()
}

fn radio_group_message<Message>(
    options: Vec<String>,
    selected: impl Into<String>,
    on_select: impl Fn(String) -> Message + 'static,
) -> Element<Message>
where
    Message: Send + 'static,
{
    radio_group_impl(options, selected, move |value| {
        dispatch_message(on_select(value))
    })
}

// Struct component API
pub struct RadioGroup<Message = ()> {
    options: Vec<String>,
    selected: Option<String>,
    default_selected: String,
    on_select: Option<std::rc::Rc<dyn Fn(String) -> Message>>,
}

impl<Message> RadioGroup<Message> {
    pub fn new(options: Vec<String>) -> Self {
        Self {
            options,
            selected: None,
            default_selected: String::new(),
            on_select: None,
        }
    }

    pub fn selected(mut self, selected: impl Into<String>) -> Self {
        self.selected = Some(selected.into());
        self
    }

    pub fn default_selected(mut self, selected: impl Into<String>) -> Self {
        self.default_selected = selected.into();
        self
    }

    pub fn on_select(mut self, handler: impl Fn(String) -> Message + 'static) -> Self {
        self.on_select = Some(std::rc::Rc::new(handler));
        self
    }
}

impl<Message: Send + 'static> arkit::advanced::Widget<Message, arkit::Theme, arkit::Renderer>
    for RadioGroup<Message>
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
        let on_select = self.on_select.clone();
        Some(radio_group_impl(
            self.options.clone(),
            selected,
            move |value| {
                if !is_controlled {
                    *state.borrow_mut() = value.clone();
                    super::request_widget_rerender();
                }
                if let Some(on_select) = on_select.as_ref() {
                    dispatch_message(on_select(value));
                }
            },
        ))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn into_any(self: Box<Self>) -> Box<dyn std::any::Any> {
        self
    }
}

impl<Message: Send + 'static> From<RadioGroup<Message>> for Element<Message> {
    fn from(value: RadioGroup<Message>) -> Self {
        Element::new(value)
    }
}
