use super::button::icon_button;
use super::*;
use std::rc::Rc;

pub(crate) const DIALOG_MAX_WIDTH: f32 = 512.0;
const DIALOG_VIEWPORT_INSET: f32 = spacing::LG;

pub(crate) fn modal_overlay<Message: 'static>(
    open: bool,
    panel: Element<Message>,
    on_dismiss: Option<Rc<dyn Fn()>>,
) -> Element<Message> {
    arkit::modal_overlay(
        if open { Some(panel) } else { None },
        arkit::ModalOverlaySpec {
            open,
            presentation: arkit::ModalPresentation::CenteredDialog,
            dismiss_on_backdrop: on_dismiss.is_some(),
            backdrop_color: 0x80000000,
            viewport_inset: DIALOG_VIEWPORT_INSET,
        },
        on_dismiss,
    )
}

pub(crate) fn dialog_panel<Message: Send + 'static>(
    content: Vec<Element<Message>>,
    on_close: Option<Rc<dyn Fn()>>,
) -> Element<Message> {
    let close_button = icon_button::<Message>("x")
        .theme(ButtonVariant::Ghost)
        .width(28.0)
        .height(28.0)
        .padding(arkit::Padding::ZERO)
        .opacity(0.7_f32)
        .on_click(move || {
            if let Some(close) = on_close.as_ref() {
                close();
            }
        });

    shadow_sm(
        arkit::column_component::<Message, arkit::Theme>()
            .percent_width(1.0)
            .max_width_constraint(DIALOG_MAX_WIDTH)
            .padding([spacing::XXL, spacing::XXL, spacing::XXL, spacing::XXL])
            .border_radius([radii().lg, radii().lg, radii().lg, radii().lg])
            .border_width([1.0, 1.0, 1.0, 1.0])
            .border_color(colors().border)
            .background_color(colors().background)
            .children(vec![
                arkit::row_component::<Message, arkit::Theme>()
                    .percent_width(1.0)
                    .justify_content_end()
                    .children(vec![close_button.into()])
                    .into(),
                arkit::column_component::<Message, arkit::Theme>()
                    .percent_width(1.0)
                    .children(vec![stack(content, spacing::LG)])
                    .into(),
            ]),
    )
    .into()
}

fn dialog_impl<Message: Send + 'static>(
    _title: impl Into<String>,
    open: bool,
    on_open_change: impl Fn(bool) + 'static,
    content: Vec<Element<Message>>,
) -> Element<Message> {
    let dismiss = Rc::new(move || on_open_change(false));
    modal_overlay(
        open,
        dialog_panel(content, Some(dismiss.clone())),
        Some(dismiss),
    )
}

fn dialog_message<Message>(
    title: impl Into<String>,
    open: bool,
    on_open_change: impl Fn(bool) -> Message + 'static,
    content: Vec<Element<Message>>,
) -> Element<Message>
where
    Message: Send + 'static,
{
    dialog_impl(
        title,
        open,
        move |value| dispatch_message(on_open_change(value)),
        content,
    )
}

pub(super) fn dialog_footer<Message: 'static>(actions: Vec<Element<Message>>) -> Element<Message> {
    arkit::column_component::<Message, arkit::Theme>()
        .percent_width(1.0)
        .children(
            actions
                .into_iter()
                .rev()
                .enumerate()
                .map(|(index, child)| {
                    if index == 0 {
                        child
                    } else {
                        arkit::row_component::<Message, arkit::Theme>()
                            .percent_width(1.0)
                            .margin([spacing::SM, 0.0, 0.0, 0.0])
                            .children(vec![child])
                            .into()
                    }
                })
                .collect::<Vec<_>>(),
        )
        .into()
}

pub(super) fn dialog_header<Message: 'static>(
    title: impl Into<String>,
    description: impl Into<String>,
) -> Element<Message> {
    let title = title.into();
    let description = description.into();
    let mut children = vec![arkit::text::<Message, arkit::Theme>(title)
        .font_size(typography::LG)
        .font_weight(FontWeight::W600)
        .font_color(colors().foreground)
        .line_height(18.0)
        .into()];
    if !description.is_empty() {
        children.push(
            margin_top(
                arkit::text::<Message, arkit::Theme>(description)
                    .font_size(typography::SM)
                    .font_color(colors().muted_foreground)
                    .line_height(20.0),
                spacing::SM,
            )
            .into(),
        );
    }
    arkit::column_component()
        .percent_width(1.0)
        .children(children)
        .into()
}

// Struct component API
pub struct Dialog<Message = ()> {
    title: String,
    open: Option<bool>,
    default_open: bool,
    on_open_change: Option<std::rc::Rc<dyn Fn(bool) -> Message>>,
    content: std::cell::RefCell<Option<Vec<Element<Message>>>>,
}

impl<Message> Dialog<Message> {
    pub fn new(title: impl Into<String>, content: Vec<Element<Message>>) -> Self {
        Self {
            title: title.into(),
            open: None,
            default_open: false,
            on_open_change: None,
            content: std::cell::RefCell::new(Some(content)),
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
    for Dialog<Message>
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
        Some(dialog_impl(
            self.title.clone(),
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
            super::take_component_slot(&self.content, "dialog content"),
        ))
    }
}

impl<Message: Send + 'static> From<Dialog<Message>> for Element<Message> {
    fn from(value: Dialog<Message>) -> Self {
        Element::new(value)
    }
}

pub struct DialogHeader<Message = ()> {
    title: String,
    description: String,
    _marker: std::marker::PhantomData<Message>,
}

impl<Message> DialogHeader<Message> {
    pub fn new(title: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            description: description.into(),
            _marker: std::marker::PhantomData,
        }
    }
}

impl_component_widget!(DialogHeader<Message>, Message, |value: &DialogHeader<
    Message,
>| {
    dialog_header(value.title.clone(), value.description.clone())
});

pub struct DialogFooter<Message = ()> {
    actions: std::cell::RefCell<Option<Vec<Element<Message>>>>,
}

impl<Message> DialogFooter<Message> {
    pub fn new(actions: Vec<Element<Message>>) -> Self {
        Self {
            actions: std::cell::RefCell::new(Some(actions)),
        }
    }
}

impl_component_widget!(DialogFooter<Message>, Message, |value: &DialogFooter<
    Message,
>| {
    dialog_footer(super::take_component_slot(
        &value.actions,
        "dialog footer actions",
    ))
});
