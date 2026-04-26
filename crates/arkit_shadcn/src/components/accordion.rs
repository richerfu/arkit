use std::rc::Rc;

use super::text::text_sm_medium;
use super::*;
use arkit::ohos_arkui_binding::component::attribute::ArkUICommonAttribute;
use arkit_icon as lucide;

const ACCORDION_TRIGGER_GAP: f32 = spacing::LG;
const ACCORDION_ICON_SIZE: f32 = 16.0;
const ACCORDION_CHEVRON_ROTATION: f32 = 180.0;

type AccordionSingleChangeHandler = Rc<dyn Fn(Option<String>)>;
type AccordionToggleHandler = Rc<dyn Fn()>;

pub struct AccordionTriggerSpec<Message = ()> {
    child: Element<Message>,
}

pub struct AccordionContentSpec<Message = ()> {
    children: Vec<Element<Message>>,
}

pub struct AccordionItemSpec<Message = ()> {
    trigger: Element<Message>,
    value: String,
    content: Vec<Element<Message>>,
    disabled: bool,
}

impl<Message: 'static> AccordionItemSpec<Message> {
    pub fn new(
        title: impl Into<String>,
        value: impl Into<String>,
        content: Vec<Element<Message>>,
    ) -> Self {
        Self::from_parts(
            value,
            accordion_trigger_text(title),
            accordion_content(content),
        )
    }

    pub fn from_parts(
        value: impl Into<String>,
        trigger: AccordionTriggerSpec<Message>,
        content: AccordionContentSpec<Message>,
    ) -> Self {
        Self {
            trigger: trigger.child,
            value: value.into(),
            content: content.children,
            disabled: false,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn trigger(mut self, trigger: Element<Message>) -> Self {
        self.trigger = trigger;
        self
    }

    pub fn content(mut self, content: Vec<Element<Message>>) -> Self {
        self.content = content;
        self
    }
}

fn accordion_trigger<Message: 'static>(child: Element<Message>) -> AccordionTriggerSpec<Message> {
    AccordionTriggerSpec { child }
}

fn accordion_trigger_text<Message: 'static>(
    title: impl Into<String>,
) -> AccordionTriggerSpec<Message> {
    accordion_trigger(text_sm_medium(title))
}

fn accordion_content<Message: 'static>(
    children: Vec<Element<Message>>,
) -> AccordionContentSpec<Message> {
    AccordionContentSpec { children }
}

fn accordion_item_spec<Message: 'static>(
    title: impl Into<String>,
    value: impl Into<String>,
    content: Vec<Element<Message>>,
) -> AccordionItemSpec<Message> {
    AccordionItemSpec::new(title, value, content)
}

fn accordion_item_parts<Message: 'static>(
    value: impl Into<String>,
    trigger: AccordionTriggerSpec<Message>,
    content: AccordionContentSpec<Message>,
) -> AccordionItemSpec<Message> {
    AccordionItemSpec::from_parts(value, trigger, content)
}

fn accordion_chevron<Message: 'static>(is_open: bool) -> Element<Message> {
    let angle = if is_open {
        ACCORDION_CHEVRON_ROTATION
    } else {
        0.0
    };

    arkit::row_component::<Message, arkit::Theme>()
        .width(ACCORDION_ICON_SIZE)
        .height(ACCORDION_ICON_SIZE)
        .align_items_center()
        .justify_content_center()
        .native(move |node| {
            node.set_transform_center(vec![0.0, 0.0, 0.0, 0.5, 0.5, 0.0])?;
            node.set_rotate(vec![0.0, 0.0, 1.0, angle, 0.0])?;
            Ok(())
        })
        .children(vec![lucide::icon("chevron-down")
            .size(ACCORDION_ICON_SIZE)
            .color(colors().muted_foreground)
            .render::<Message, arkit::Theme>()])
        .into()
}

fn accordion_container<Message: 'static>(children: Vec<Element<Message>>) -> Element<Message> {
    arkit::column_component::<Message, arkit::Theme>()
        .percent_width(1.0)
        .align_items_start()
        .children(children)
        .into()
}

fn accordion_content_panel<Message: 'static>(
    content: Vec<Element<Message>>,
    is_open: bool,
) -> Element<Message> {
    if is_open {
        arkit::column_component::<Message, arkit::Theme>()
            .percent_width(1.0)
            .align_items_start()
            .padding([0.0, 0.0, spacing::LG, 0.0])
            .children(content)
            .into()
    } else {
        arkit::column_component::<Message, arkit::Theme>()
            .percent_width(1.0)
            .height(0.0)
            .opacity(0.0_f32)
            .hit_test_behavior(HitTestBehavior::Transparent)
            .into()
    }
}

fn accordion_item_view<Message>(
    trigger: Element<Message>,
    is_open: bool,
    disabled: bool,
    on_toggle: AccordionToggleHandler,
    content: Vec<Element<Message>>,
) -> Element<Message>
where
    Message: 'static,
{
    let mut trigger_row = arkit::row_component::<Message, arkit::Theme>()
        .percent_width(1.0)
        .align_items_top()
        .justify_content_start()
        .padding([spacing::LG, 0.0, spacing::LG, 0.0])
        .border_radius([radii().md, radii().md, radii().md, radii().md])
        .children(vec![
            arkit::column_component::<Message, arkit::Theme>()
                .layout_weight(1.0_f32)
                .align_items_start()
                .margin([0.0, ACCORDION_TRIGGER_GAP, 0.0, 0.0])
                .children(vec![trigger])
                .into(),
            accordion_chevron::<Message>(is_open),
        ]);

    if disabled {
        trigger_row = trigger_row.enabled(false).opacity(0.5_f32);
    } else {
        trigger_row = trigger_row.on_click(move || on_toggle());
    }

    arkit::column_component::<Message, arkit::Theme>()
        .percent_width(1.0)
        .border_width([0.0, 0.0, 1.0, 0.0])
        .border_color(colors().border)
        .children(vec![
            trigger_row.into(),
            accordion_content_panel(content, is_open),
        ])
        .into()
}

fn render_single_items<Message>(
    items: Vec<AccordionItemSpec<Message>>,
    open_item: Option<String>,
    collapsible: bool,
    on_value_change: Option<AccordionSingleChangeHandler>,
) -> Element<Message>
where
    Message: 'static,
{
    let children = items
        .into_iter()
        .map(|item| {
            let next_value = item.value.clone();
            let current_value = open_item.clone();
            let callback = on_value_change.clone();

            accordion_item_view(
                item.trigger,
                open_item.as_deref() == Some(item.value.as_str()),
                item.disabled,
                Rc::new(move || {
                    let next = if current_value.as_deref() == Some(next_value.as_str()) {
                        if collapsible {
                            None
                        } else {
                            Some(next_value.clone())
                        }
                    } else {
                        Some(next_value.clone())
                    };

                    if let Some(handler) = callback.as_ref() {
                        handler(next);
                    }
                }),
                item.content,
            )
        })
        .collect::<Vec<_>>();

    accordion_container(children)
}

fn accordion<Message: 'static>(children: Vec<Element<Message>>) -> Element<Message> {
    accordion_container(children)
}

fn accordion_single_controlled<Message>(
    items: Vec<AccordionItemSpec<Message>>,
    value: Option<String>,
    collapsible: bool,
    on_value_change: impl Fn(Option<String>) -> Message + 'static,
) -> Element<Message>
where
    Message: Send + 'static,
{
    render_single_items(
        items,
        value,
        collapsible,
        Some(dispatch_optional_string(on_value_change)),
    )
}

// Struct component API
pub struct Accordion<Message = ()> {
    items: std::cell::RefCell<Option<Vec<AccordionItemSpec<Message>>>>,
    value: Option<Option<String>>,
    default_value: Option<String>,
    collapsible: bool,
    on_value_change: Option<std::rc::Rc<dyn Fn(Option<String>) -> Message>>,
}

impl<Message> Accordion<Message> {
    pub fn single(items: Vec<AccordionItemSpec<Message>>) -> Self {
        Self {
            items: std::cell::RefCell::new(Some(items)),
            value: None,
            default_value: None,
            collapsible: false,
            on_value_change: None,
        }
    }

    pub fn value(mut self, value: Option<String>) -> Self {
        self.value = Some(value);
        self
    }

    pub fn default_value(mut self, value: Option<String>) -> Self {
        self.default_value = value;
        self
    }

    pub fn collapsible(mut self, collapsible: bool) -> Self {
        self.collapsible = collapsible;
        self
    }

    pub fn on_value_change(
        mut self,
        handler: impl Fn(Option<String>) -> Message + 'static,
    ) -> Self {
        self.on_value_change = Some(std::rc::Rc::new(handler));
        self
    }
}

impl<Message: Send + 'static> arkit::advanced::Widget<Message, arkit::Theme, arkit::Renderer>
    for Accordion<Message>
{
    fn body(
        &self,
        tree: &mut arkit::advanced::widget::Tree,
        _renderer: &arkit::Renderer,
    ) -> Option<Element<Message>> {
        let state = super::widget_state(tree, || self.default_value.clone());
        let is_controlled = self.value.is_some();
        let value = self.value.clone().unwrap_or_else(|| state.borrow().clone());
        let handler = self.on_value_change.clone();
        Some(render_single_items(
            super::take_component_slot(&self.items, "accordion items"),
            value,
            self.collapsible,
            Some(std::rc::Rc::new(move |value| {
                if !is_controlled {
                    *state.borrow_mut() = value.clone();
                    super::request_widget_rerender();
                }
                if let Some(handler) = handler.as_ref() {
                    dispatch_message(handler(value));
                }
            })),
        ))
    }
}

impl<Message: Send + 'static> From<Accordion<Message>> for Element<Message> {
    fn from(value: Accordion<Message>) -> Self {
        Element::new(value)
    }
}
