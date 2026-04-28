use super::floating_layer::{floating_panel_with_builder, FloatingAlign, FloatingSide};
use super::*;
use arkit_icon as lucide;
use std::rc::Rc;

const SELECT_PANEL_FALLBACK_WIDTH: f32 = 180.0;

struct SelectTreeState {
    selected: String,
    open: bool,
}

fn select<Message: 'static>(
    options: Vec<String>,
    selected: impl Into<String>,
    open: bool,
    on_open_change: impl Fn(bool) + 'static,
    on_select: impl Fn(String) + 'static,
) -> Element<Message> {
    let selected = selected.into();
    let on_select = Rc::new(on_select);
    let on_open_change = Rc::new(on_open_change);

    let trigger = touch_activate(
        shadow_sm(crate::styles::rounded(
            crate::styles::border(
                arkit::row_component::<Message, arkit::Theme>()
                    .height(40.0)
                    .percent_width(1.0)
                    .background_color(colors().background)
                    .foreground_color(colors().foreground)
                    .padding([8.0, spacing::MD, 8.0, spacing::MD])
                    .align_items_center()
                    .justify_content(JustifyContent::SpaceBetween)
                    .children(vec![
                        {
                            let has_value = !selected.is_empty();
                            let label = if has_value {
                                selected.clone()
                            } else {
                                String::from("Select a fruit")
                            };
                            arkit::text::<Message, arkit::Theme>(label)
                                .font_size(typography::SM)
                                .font_color(if has_value {
                                    colors().foreground
                                } else {
                                    colors().muted_foreground
                                })
                                .line_height(20.0)
                                .into()
                        },
                        lucide::icon("chevron-down")
                            .size(16.0)
                            .color(colors().muted_foreground)
                            .render(),
                    ]),
            ),
            radii().md,
        )),
        {
            let on_open_change = on_open_change.clone();
            move || on_open_change(!open)
        },
    )
    .into();

    let panel_builder: Rc<dyn Fn(Option<f32>) -> Element<Message>> = Rc::new({
        let options = options.clone();
        let selected = selected.clone();
        let on_select = on_select.clone();
        let on_open_change = on_open_change.clone();
        move |trigger_width| {
            let count = options.len();
            let items = options
                .iter()
                .cloned()
                .map(|option| {
                    let sel = selected.clone();
                    let opt = option.clone();
                    let on_select = on_select.clone();
                    let on_open_change = on_open_change.clone();
                    let active = sel == opt;
                    let opt_click = opt.clone();

                    touch_activate(
                        arkit::row_component::<Message, arkit::Theme>()
                            .percent_width(1.0)
                            .height(36.0)
                            .align_items_center()
                            .justify_content(JustifyContent::SpaceBetween)
                            .padding([8.0, spacing::SM, 8.0, spacing::SM])
                            .border_radius([radii().sm, radii().sm, radii().sm, radii().sm])
                            .background_color(if active { colors().accent } else { 0x00000000 })
                            .children(vec![
                                arkit::text::<Message, arkit::Theme>(opt.clone())
                                    .font_size(typography::SM)
                                    .font_color(if active {
                                        colors().accent_foreground
                                    } else {
                                        colors().foreground
                                    })
                                    .line_height(20.0)
                                    .into(),
                                if active {
                                    lucide::icon("check")
                                        .size(16.0)
                                        .color(colors().muted_foreground)
                                        .render::<Message, arkit::Theme>()
                                } else {
                                    arkit::row_component::<Message, arkit::Theme>()
                                        .width(16.0)
                                        .height(16.0)
                                        .into()
                                },
                            ]),
                        move || {
                            on_select(opt_click.clone());
                            on_open_change(false);
                        },
                    )
                    .into()
                })
                .collect::<Vec<_>>();

            let list = if count > 8 {
                arkit::scroll_component::<Message, arkit::Theme>()
                    .height(208.0)
                    .children(vec![arkit::column_component::<Message, arkit::Theme>()
                        .percent_width(1.0)
                        .children(items)
                        .into()])
                    .into()
            } else {
                arkit::column_component::<Message, arkit::Theme>()
                    .percent_width(1.0)
                    .children(items)
                    .into()
            };

            let mut panel = arkit::column_component::<Message, arkit::Theme>()
                .padding([spacing::XXS, spacing::XXS, spacing::XXS, spacing::XXS])
                .children(vec![
                    arkit::row_component::<Message, arkit::Theme>()
                        .padding([8.0, spacing::SM, 8.0, spacing::SM])
                        .children(vec![arkit::text::<Message, arkit::Theme>("Fruits")
                            .font_size(typography::XS)
                            .font_color(colors().muted_foreground)
                            .line_height(16.0)
                            .into()])
                        .into(),
                    list,
                ]);

            panel = panel.width(trigger_width.unwrap_or(SELECT_PANEL_FALLBACK_WIDTH));

            panel_surface(panel).into()
        }
    });

    let dismiss = {
        let on_open_change = on_open_change.clone();
        Rc::new(move || on_open_change(false))
    };
    floating_panel_with_builder(
        trigger,
        open,
        FloatingSide::Bottom,
        FloatingAlign::Start,
        panel_builder,
        Some(dismiss),
    )
}

fn select_message<Message>(
    options: Vec<String>,
    selected: impl Into<String>,
    open: bool,
    on_open_change: impl Fn(bool) -> Message + 'static,
    on_select: impl Fn(String) -> Message + 'static,
) -> Element<Message>
where
    Message: Send + 'static,
{
    select(
        options,
        selected,
        open,
        move |value| dispatch_message(on_open_change(value)),
        move |value| dispatch_message(on_select(value)),
    )
}

// Struct component API
pub struct Select<Message = ()> {
    options: Vec<String>,
    selected: Option<String>,
    default_selected: String,
    open: Option<bool>,
    default_open: bool,
    on_open_change: Option<std::rc::Rc<dyn Fn(bool) -> Message>>,
    on_select: Option<std::rc::Rc<dyn Fn(String) -> Message>>,
}

impl<Message> Select<Message> {
    pub fn new(options: Vec<String>) -> Self {
        Self {
            options,
            selected: None,
            default_selected: String::new(),
            open: None,
            default_open: false,
            on_open_change: None,
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

    pub fn on_select(mut self, handler: impl Fn(String) -> Message + 'static) -> Self {
        self.on_select = Some(std::rc::Rc::new(handler));
        self
    }
}

impl<Message: Send + 'static> arkit::advanced::Widget<Message, arkit::Theme, arkit::Renderer>
    for Select<Message>
{
    fn body(
        &self,
        tree: &mut arkit::advanced::widget::Tree,
        _renderer: &arkit::Renderer,
    ) -> Option<Element<Message>> {
        let state = super::widget_state(tree, || SelectTreeState {
            selected: self.default_selected.clone(),
            open: self.default_open,
        });
        let selected_controlled = self.selected.is_some();
        let open_controlled = self.open.is_some();
        let selected = self.selected.clone().unwrap_or_else(|| {
            let state = state.borrow();
            state.selected.clone()
        });
        let open = self.open.unwrap_or_else(|| state.borrow().open);
        let on_open_change = self.on_open_change.clone();
        let on_select = self.on_select.clone();
        let state_for_open = state.clone();
        let state_for_select = state.clone();

        Some(select(
            self.options.clone(),
            selected,
            open,
            move |value| {
                if !open_controlled {
                    state_for_open.borrow_mut().open = value;
                    super::request_widget_rerender();
                }
                if let Some(on_open_change) = on_open_change.as_ref() {
                    dispatch_message(on_open_change(value));
                }
            },
            move |value| {
                if !selected_controlled {
                    state_for_select.borrow_mut().selected = value.clone();
                    super::request_widget_rerender();
                }
                if let Some(on_select) = on_select.as_ref() {
                    dispatch_message(on_select(value));
                }
            },
        ))
    }
}

impl<Message: Send + 'static> From<Select<Message>> for Element<Message> {
    fn from(value: Select<Message>) -> Self {
        Element::new(value)
    }
}
