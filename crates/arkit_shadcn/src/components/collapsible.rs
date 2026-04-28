use super::button::icon_button;
use super::*;

fn collapsible<Message: Send + 'static>(
    title: impl Into<String>,
    open: bool,
    on_open_change: impl Fn(bool) + 'static,
    content: Vec<Element<Message>>,
) -> Element<Message> {
    let mut items = content.into_iter();
    let first = items.next();
    let rest: Vec<Element<Message>> = items
        .map(|child| {
            arkit::row_component::<Message, arkit::Theme>()
                .margin([spacing::SM, 0.0, 0.0, 0.0])
                .children(vec![child])
                .into()
        })
        .collect();

    let mut children: Vec<Element<Message>> = vec![arkit::row_component::<Message, arkit::Theme>()
        .percent_width(1.0)
        .align_items_center()
        .justify_content(JustifyContent::SpaceBetween)
        .padding([0.0, spacing::LG, 0.0, spacing::LG])
        .on_click(move || on_open_change(!open))
        .children(vec![
            body_text(title)
                .font_weight(FontWeight::W600)
                .font_color(colors().foreground)
                .into(),
            icon_button("chevrons-up-down")
                .theme(ButtonVariant::Ghost)
                .width(32.0)
                .height(32.0)
                .padding(arkit::Padding::ZERO)
                .into(),
        ])
        .into()];

    if let Some(first) = first {
        children.push(
            arkit::row_component::<Message, arkit::Theme>()
                .margin([spacing::SM, 0.0, 0.0, 0.0])
                .children(vec![first])
                .into(),
        );
    }

    // Keep the body mounted and let normal patching update visibility so layout
    // and interaction remain stable across explicit runtime rerenders.
    if !rest.is_empty() {
        children.push(
            visibility_gate(
                arkit::column_component::<Message, arkit::Theme>().percent_width(1.0),
                open,
            )
            .children(rest)
            .into(),
        );
    }

    arkit::column_component::<Message, arkit::Theme>()
        .percent_width(1.0)
        .children(children)
        .into()
}

fn collapsible_message<Message>(
    title: impl Into<String>,
    open: bool,
    on_open_change: impl Fn(bool) -> Message + 'static,
    content: Vec<Element<Message>>,
) -> Element<Message>
where
    Message: Send + 'static,
{
    collapsible(
        title,
        open,
        move |value| dispatch_message(on_open_change(value)),
        content,
    )
}

// Struct component API
pub struct Collapsible<Message = ()> {
    title: String,
    content: std::cell::RefCell<Option<Vec<Element<Message>>>>,
    open: Option<bool>,
    default_open: bool,
    on_open_change: Option<std::rc::Rc<dyn Fn(bool) -> Message>>,
}

impl<Message> Collapsible<Message> {
    pub fn new(title: impl Into<String>, content: Vec<Element<Message>>) -> Self {
        Self {
            title: title.into(),
            content: std::cell::RefCell::new(Some(content)),
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
    for Collapsible<Message>
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
        Some(collapsible(
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
            super::take_component_slot(&self.content, "collapsible content"),
        ))
    }
}

impl<Message: Send + 'static> From<Collapsible<Message>> for Element<Message> {
    fn from(value: Collapsible<Message>) -> Self {
        Element::new(value)
    }
}
