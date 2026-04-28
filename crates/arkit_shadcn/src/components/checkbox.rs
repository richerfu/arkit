use super::label::label;
use super::*;
use arkit_icon as lucide;
use std::rc::Rc;

const CHECKBOX_SIZE: f32 = 16.0;
const CHECKBOX_BORDER_WIDTH: f32 = 1.0;
const CHECKBOX_ICON_SIZE: f32 = 12.0;
const CHECKBOX_ICON_STROKE_WIDTH: f32 = 3.5;

#[derive(Debug, Clone, Copy)]
struct CheckboxStyle {
    checked_color: u32,
    disabled: bool,
}

fn checkbox_indicator<Message: 'static>(checked: bool, style: CheckboxStyle) -> Element<Message> {
    let mut indicator = shadow_sm(
        arkit::row_component::<Message, arkit::Theme>()
            .width(CHECKBOX_SIZE)
            .height(CHECKBOX_SIZE)
            .align_items_center()
            .justify_content_center()
            .border_radius([radii().sm, radii().sm, radii().sm, radii().sm])
            .border_width([
                CHECKBOX_BORDER_WIDTH,
                CHECKBOX_BORDER_WIDTH,
                CHECKBOX_BORDER_WIDTH,
                CHECKBOX_BORDER_WIDTH,
            ])
            .border_color(if checked {
                style.checked_color
            } else {
                colors().input
            })
            .clip(true)
            .background_color(if checked {
                style.checked_color
            } else {
                colors().background
            }),
    );

    if checked {
        indicator = indicator.children(vec![lucide::icon("check")
            .size(CHECKBOX_ICON_SIZE)
            .stroke_width(CHECKBOX_ICON_STROKE_WIDTH)
            .color(colors().primary_foreground)
            .render::<Message, arkit::Theme>()]);
    }

    indicator.into()
}

fn checkbox_impl<Message: 'static>(
    label_text: String,
    checked: bool,
    on_toggle: Option<Rc<dyn Fn(bool)>>,
    style: CheckboxStyle,
) -> Element<Message> {
    let mut children = vec![checkbox_indicator::<Message>(checked, style)];

    if !label_text.is_empty() {
        children.push(
            arkit::row_component::<Message, arkit::Theme>()
                .align_items_center()
                .margin([0.0, 0.0, 0.0, spacing::SM])
                .children(vec![label(label_text).into()])
                .into(),
        );
    }

    let mut root = arkit::row_component::<Message, arkit::Theme>()
        .align_items_center()
        .children(children);

    if let Some(on_toggle) = on_toggle {
        if !style.disabled {
            root = root.on_click(move || on_toggle(!checked));
        }
    }

    if style.disabled {
        root = root.opacity(0.5_f32);
    }

    root.into()
}

fn checkbox_message<Message>(
    label: impl Into<String>,
    checked: bool,
    on_toggle: impl Fn(bool) -> Message + 'static,
) -> Element<Message>
where
    Message: Send + 'static,
{
    checkbox_impl(
        label.into(),
        checked,
        Some(Rc::new(move |value| dispatch_message(on_toggle(value)))),
        CheckboxStyle {
            checked_color: colors().primary,
            disabled: false,
        },
    )
}

fn checkbox_with_checked_color_message<Message>(
    label: impl Into<String>,
    checked: bool,
    on_toggle: impl Fn(bool) -> Message + 'static,
    checked_color: u32,
) -> Element<Message>
where
    Message: Send + 'static,
{
    checkbox_impl(
        label.into(),
        checked,
        Some(Rc::new(move |value| dispatch_message(on_toggle(value)))),
        CheckboxStyle {
            checked_color,
            disabled: false,
        },
    )
}

fn disabled_checkbox<Message: 'static>(
    label: impl Into<String>,
    checked: bool,
) -> Element<Message> {
    checkbox_impl(
        label.into(),
        checked,
        None,
        CheckboxStyle {
            checked_color: colors().primary,
            disabled: true,
        },
    )
}

// Struct component API
pub struct Checkbox<Message = ()> {
    label: String,
    checked: Option<bool>,
    default_checked: bool,
    checked_color: Option<u32>,
    disabled: bool,
    on_change: Option<std::rc::Rc<dyn Fn(bool) -> Message>>,
}

impl<Message> Checkbox<Message> {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            checked: None,
            default_checked: false,
            checked_color: None,
            disabled: false,
            on_change: None,
        }
    }

    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = Some(checked);
        self
    }

    pub fn default_checked(mut self, checked: bool) -> Self {
        self.default_checked = checked;
        self
    }

    pub fn checked_color(mut self, color: u32) -> Self {
        self.checked_color = Some(color);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
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
    for Checkbox<Message>
{
    fn body(
        &self,
        tree: &mut arkit::advanced::widget::Tree,
        _renderer: &arkit::Renderer,
    ) -> Option<Element<Message>> {
        let state = super::widget_state(tree, || self.default_checked);
        let is_controlled = self.checked.is_some();
        let checked = self.checked.unwrap_or_else(|| *state.borrow());
        let style = CheckboxStyle {
            checked_color: self.checked_color.unwrap_or_else(|| colors().primary),
            disabled: self.disabled,
        };

        if self.disabled {
            return Some(checkbox_impl(self.label.clone(), checked, None, style));
        }

        let handler = self.on_change.clone();
        Some(checkbox_impl(
            self.label.clone(),
            checked,
            Some(std::rc::Rc::new(move |value| {
                if !is_controlled {
                    *state.borrow_mut() = value;
                    super::request_widget_rerender();
                }
                if let Some(handler) = handler.as_ref() {
                    dispatch_message(handler(value));
                }
            })),
            style,
        ))
    }
}

impl<Message: Send + 'static> From<Checkbox<Message>> for Element<Message> {
    fn from(value: Checkbox<Message>) -> Self {
        Element::new(value)
    }
}
