use std::rc::Rc;

use super::*;
use arkit::ohos_arkui_binding::component::attribute::ArkUICommonAttribute;
use arkit_icon as lucide;

const ACCORDION_TRIGGER_GAP: f32 = spacing::LG;
const ACCORDION_TRIGGER_RADIUS: f32 = radius::MD;
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
}

pub fn accordion_trigger<Message: 'static>(
    child: Element<Message>,
) -> AccordionTriggerSpec<Message> {
    AccordionTriggerSpec { child }
}

pub fn accordion_trigger_text<Message: 'static>(
    title: impl Into<String>,
) -> AccordionTriggerSpec<Message> {
    accordion_trigger(text_sm_medium(title))
}

pub fn accordion_content<Message: 'static>(
    children: Vec<Element<Message>>,
) -> AccordionContentSpec<Message> {
    AccordionContentSpec { children }
}

pub fn accordion_item_spec<Message: 'static>(
    title: impl Into<String>,
    value: impl Into<String>,
    content: Vec<Element<Message>>,
) -> AccordionItemSpec<Message> {
    AccordionItemSpec::new(title, value, content)
}

pub fn accordion_item_parts<Message: 'static>(
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
            .color(color::MUTED_FOREGROUND)
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
        .border_radius([
            ACCORDION_TRIGGER_RADIUS,
            ACCORDION_TRIGGER_RADIUS,
            ACCORDION_TRIGGER_RADIUS,
            ACCORDION_TRIGGER_RADIUS,
        ])
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
        .border_color(color::BORDER)
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

pub fn accordion<Message: 'static>(children: Vec<Element<Message>>) -> Element<Message> {
    accordion_container(children)
}

pub fn accordion_single_controlled<Message>(
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
