use std::rc::Rc;

use super::*;
use arkit::component;
use arkit::ohos_arkui_binding::component::attribute::ArkUICommonAttribute;
use arkit_icon as lucide;

const ACCORDION_TRIGGER_GAP: f32 = spacing::LG;
const ACCORDION_TRIGGER_RADIUS: f32 = radius::MD;
const ACCORDION_ICON_SIZE: f32 = 16.0;
const ACCORDION_CHEVRON_ROTATION: f32 = 180.0;

pub type AccordionSingleChangeHandler = Rc<dyn Fn(Option<String>)>;
pub type AccordionMultipleChangeHandler = Rc<dyn Fn(Vec<String>)>;
pub type AccordionValueChangeHandler = Rc<dyn Fn(AccordionValue)>;
type AccordionToggleHandler = Rc<dyn Fn()>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AccordionType {
    Single,
    Multiple,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AccordionValue {
    Single(Option<String>),
    Multiple(Vec<String>),
}

impl AccordionValue {
    pub fn single(value: impl Into<String>) -> Self {
        Self::Single(Some(value.into()))
    }

    pub fn single_optional(value: Option<String>) -> Self {
        Self::Single(value)
    }

    pub fn multiple<I, S>(values: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self::Multiple(values.into_iter().map(Into::into).collect())
    }

    fn as_single(&self) -> Option<String> {
        match self {
            Self::Single(value) => value.clone(),
            Self::Multiple(values) => values.first().cloned(),
        }
    }

    fn as_multiple(&self) -> Vec<String> {
        match self {
            Self::Single(value) => value.iter().cloned().collect(),
            Self::Multiple(values) => values.clone(),
        }
    }
}

#[derive(Clone)]
pub struct AccordionRootSpec {
    accordion_type: AccordionType,
    collapsible: bool,
    default_value: AccordionValue,
    on_value_change: Option<AccordionValueChangeHandler>,
}

impl AccordionRootSpec {
    pub fn single() -> Self {
        Self {
            accordion_type: AccordionType::Single,
            collapsible: false,
            default_value: AccordionValue::Single(None),
            on_value_change: None,
        }
    }

    pub fn multiple() -> Self {
        Self {
            accordion_type: AccordionType::Multiple,
            collapsible: false,
            default_value: AccordionValue::Multiple(Vec::new()),
            on_value_change: None,
        }
    }

    pub fn collapsible(mut self, collapsible: bool) -> Self {
        self.collapsible = collapsible;
        self
    }

    pub fn default_single(mut self, value: impl Into<String>) -> Self {
        self.default_value = AccordionValue::single(value);
        self
    }

    pub fn default_single_optional(mut self, value: Option<String>) -> Self {
        self.default_value = AccordionValue::single_optional(value);
        self
    }

    pub fn default_multiple<I, S>(mut self, values: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.default_value = AccordionValue::multiple(values);
        self
    }

    pub fn on_value_change(mut self, handler: AccordionValueChangeHandler) -> Self {
        self.on_value_change = Some(handler);
        self
    }
}

pub struct AccordionTriggerSpec {
    child: Element,
}

pub struct AccordionContentSpec {
    children: Vec<Element>,
}

pub struct AccordionItemSpec {
    trigger: Element,
    value: String,
    content: Vec<Element>,
    disabled: bool,
}

impl AccordionItemSpec {
    pub fn new(title: impl Into<String>, value: impl Into<String>, content: Vec<Element>) -> Self {
        Self::from_parts(value, accordion_trigger_text(title), accordion_content(content))
    }

    pub fn from_parts(
        value: impl Into<String>,
        trigger: AccordionTriggerSpec,
        content: AccordionContentSpec,
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

pub fn accordion_trigger(child: Element) -> AccordionTriggerSpec {
    AccordionTriggerSpec { child }
}

pub fn accordion_trigger_text(title: impl Into<String>) -> AccordionTriggerSpec {
    accordion_trigger(text_sm_medium(title))
}

pub fn accordion_content(children: Vec<Element>) -> AccordionContentSpec {
    AccordionContentSpec { children }
}

pub fn accordion_item_spec(
    title: impl Into<String>,
    value: impl Into<String>,
    content: Vec<Element>,
) -> AccordionItemSpec {
    AccordionItemSpec::new(title, value, content)
}

pub fn accordion_item_parts(
    value: impl Into<String>,
    trigger: AccordionTriggerSpec,
    content: AccordionContentSpec,
) -> AccordionItemSpec {
    AccordionItemSpec::from_parts(value, trigger, content)
}

fn accordion_chevron(is_open: bool) -> Element {
    let angle = if is_open {
        ACCORDION_CHEVRON_ROTATION
    } else {
        0.0
    };

    arkit::row_component()
        .width(ACCORDION_ICON_SIZE)
        .height(ACCORDION_ICON_SIZE)
        .align_items_center()
        .style(ArkUINodeAttributeType::RowJustifyContent, FLEX_ALIGN_CENTER)
        .native(move |node| {
            node.set_transform_center(vec![0.0, 0.0, 0.0, 0.5, 0.5, 0.0])?;
            node.set_rotate(vec![0.0, 0.0, 1.0, angle, 0.0])?;
            Ok(())
        })
        .children(vec![lucide::icon("chevron-down")
            .size(ACCORDION_ICON_SIZE)
            .color(color::MUTED_FOREGROUND)
            .render()])
        .into()
}

fn accordion_container(children: Vec<Element>) -> Element {
    arkit::column_component()
        .percent_width(1.0)
        .align_items_start()
        .children(children)
        .into()
}

fn accordion_content_panel(content: Vec<Element>, is_open: bool) -> Element {
    if is_open {
        arkit::column_component()
            .percent_width(1.0)
            .align_items_start()
            .style(
                ArkUINodeAttributeType::Padding,
                vec![0.0, 0.0, spacing::LG, 0.0],
            )
            .children(content)
            .into()
    } else {
        arkit::column_component()
            .percent_width(1.0)
            .height(0.0)
            .style(ArkUINodeAttributeType::Opacity, 0.0_f32)
            .style(ArkUINodeAttributeType::HitTestBehavior, HIT_TEST_TRANSPARENT)
            .into()
    }
}

#[component]
fn accordion_item_view(
    trigger: Element,
    is_open: bool,
    disabled: bool,
    on_toggle: AccordionToggleHandler,
    content: Vec<Element>,
) -> Element {
    let mut trigger_row = arkit::row_component()
        .percent_width(1.0)
        .align_items_top()
        .style(ArkUINodeAttributeType::RowJustifyContent, FLEX_ALIGN_START)
        .style(
            ArkUINodeAttributeType::Padding,
            vec![spacing::LG, 0.0, spacing::LG, 0.0],
        )
        .style(
            ArkUINodeAttributeType::BorderRadius,
            vec![
                ACCORDION_TRIGGER_RADIUS,
                ACCORDION_TRIGGER_RADIUS,
                ACCORDION_TRIGGER_RADIUS,
                ACCORDION_TRIGGER_RADIUS,
            ],
        )
        .children(vec![
            arkit::column_component()
                .style(ArkUINodeAttributeType::LayoutWeight, 1.0_f32)
                .align_items_start()
                .style(
                    ArkUINodeAttributeType::Margin,
                    vec![0.0, ACCORDION_TRIGGER_GAP, 0.0, 0.0],
                )
                .children(vec![trigger])
                .into(),
            accordion_chevron(is_open),
        ]);

    if disabled {
        trigger_row = trigger_row
            .style(ArkUINodeAttributeType::Enabled, false)
            .style(ArkUINodeAttributeType::Opacity, 0.5_f32);
    } else {
        trigger_row = trigger_row.on_click(move || on_toggle());
    }

    arkit::column_component()
        .percent_width(1.0)
        .style(
            ArkUINodeAttributeType::BorderWidth,
            vec![0.0, 0.0, 1.0, 0.0],
        )
        .style(ArkUINodeAttributeType::BorderColor, vec![color::BORDER])
        .children(vec![
            trigger_row.into(),
            accordion_content_panel(content, is_open),
        ])
        .into()
}

fn render_single_items(
    items: Vec<AccordionItemSpec>,
    open_item: Option<String>,
    collapsible: bool,
    on_value_change: Option<AccordionSingleChangeHandler>,
) -> Element {
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

fn render_multiple_items(
    items: Vec<AccordionItemSpec>,
    open_items: Vec<String>,
    on_value_change: Option<AccordionMultipleChangeHandler>,
) -> Element {
    let children = items
        .into_iter()
        .map(|item| {
            let value = item.value.clone();
            let current_values = open_items.clone();
            let callback = on_value_change.clone();

            accordion_item_view(
                item.trigger,
                open_items.contains(&item.value),
                item.disabled,
                Rc::new(move || {
                    let mut next = current_values.clone();
                    if let Some(index) = next.iter().position(|item| item == &value) {
                        next.remove(index);
                    } else {
                        next.push(value.clone());
                    }

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

fn render_root_items(
    items: Vec<AccordionItemSpec>,
    value: AccordionValue,
    spec: AccordionRootSpec,
    on_value_change: Option<AccordionValueChangeHandler>,
) -> Element {
    match spec.accordion_type {
        AccordionType::Single => {
            let single_handler = on_value_change.map(|handler| {
                Rc::new(move |value: Option<String>| handler(AccordionValue::Single(value)))
                    as AccordionSingleChangeHandler
            });
            render_single_items(items, value.as_single(), spec.collapsible, single_handler)
        }
        AccordionType::Multiple => {
            let multi_handler = on_value_change.map(|handler| {
                Rc::new(move |value: Vec<String>| handler(AccordionValue::Multiple(value)))
                    as AccordionMultipleChangeHandler
            });
            render_multiple_items(items, value.as_multiple(), multi_handler)
        }
    }
}

pub fn accordion(children: Vec<Element>) -> Element {
    accordion_container(children)
}

pub fn accordion_item(
    title: impl Into<String>,
    value: impl Into<String>,
    open_item: Option<String>,
    on_value_change: Option<AccordionSingleChangeHandler>,
    content: Vec<Element>,
) -> Element {
    render_single_items(
        vec![AccordionItemSpec::new(title, value, content)],
        open_item,
        true,
        on_value_change,
    )
}

#[derive(Clone)]
struct AccordionRootMarker;

#[component]
pub fn accordion_root(items: Vec<AccordionItemSpec>, spec: AccordionRootSpec) -> Element {
    let state = local_ref_state(AccordionRootMarker, spec.default_value.clone());
    let current = state.borrow().clone();
    let external_handler = spec.on_value_change.clone();
    let on_value_change: AccordionValueChangeHandler = Rc::new(move |next| {
        state.replace(next.clone());
        request_runtime_rerender();
        if let Some(handler) = external_handler.as_ref() {
            handler(next);
        }
    });
    render_root_items(items, current, spec, Some(on_value_change))
}

pub fn accordion_root_controlled(
    items: Vec<AccordionItemSpec>,
    value: AccordionValue,
    spec: AccordionRootSpec,
) -> Element {
    render_root_items(items, value, spec.clone(), spec.on_value_change.clone())
}

#[derive(Clone)]
struct AccordionSingleMarker;

#[component]
pub fn accordion_single(
    items: Vec<AccordionItemSpec>,
    collapsible: bool,
    default_value: Option<String>,
) -> Element {
    let state = local_ref_state(AccordionSingleMarker, default_value);
    let current = state.borrow().clone();
    let on_value_change: AccordionSingleChangeHandler = Rc::new(move |next| {
        state.replace(next);
        request_runtime_rerender();
    });
    render_single_items(items, current, collapsible, Some(on_value_change))
}

pub fn accordion_single_controlled(
    items: Vec<AccordionItemSpec>,
    value: Option<String>,
    collapsible: bool,
    on_value_change: Option<AccordionSingleChangeHandler>,
) -> Element {
    render_single_items(items, value, collapsible, on_value_change)
}

#[derive(Clone)]
struct AccordionMultipleMarker;

#[component]
pub fn accordion_multiple(items: Vec<AccordionItemSpec>, default_value: Vec<String>) -> Element {
    let state = local_ref_state(AccordionMultipleMarker, default_value);
    let current = state.borrow().clone();
    let on_value_change: AccordionMultipleChangeHandler = Rc::new(move |next| {
        state.replace(next);
        request_runtime_rerender();
    });
    render_multiple_items(items, current, Some(on_value_change))
}

pub fn accordion_multiple_controlled(
    items: Vec<AccordionItemSpec>,
    value: Vec<String>,
    on_value_change: Option<AccordionMultipleChangeHandler>,
) -> Element {
    render_multiple_items(items, value, on_value_change)
}
