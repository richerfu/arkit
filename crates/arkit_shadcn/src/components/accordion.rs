use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use super::*;
use arkit::ohos_arkui_binding::animate::options::Animation;
use arkit::ohos_arkui_binding::common::attribute::{
    ArkUINodeAttributeItem, ArkUINodeAttributeNumber,
};
use arkit::ohos_arkui_binding::common::error::ArkUIResult;
use arkit::ohos_arkui_binding::common::node::ArkUINode;
use arkit::ohos_arkui_binding::component::attribute::{
    ArkUIAttributeBasic, ArkUICommonAttribute, ArkUIEvent,
};
use arkit::ohos_arkui_binding::types::animation_finish_type::AnimationFinishCallbackType;
use arkit::ohos_arkui_binding::types::animation_mode::AnimationMode;
use arkit::ohos_arkui_binding::types::curve::Curve;
use arkit::{component, create_effect, create_signal, queue_after_mount, queue_ui_loop};
use arkit_icon as lucide;

const ACCORDION_TRIGGER_GAP: f32 = spacing::LG;
const ACCORDION_TRIGGER_RADIUS: f32 = radius::MD;
const ACCORDION_ICON_SIZE: f32 = 16.0;
const ACCORDION_CHEVRON_ROTATION: f32 = 180.0;
const ACCORDION_CONTENT_TRANSITION_MS: i32 = 200;
const ACCORDION_CHEVRON_OPEN_MS: i32 = 250;
const ACCORDION_CHEVRON_CLOSE_MS: i32 = 200;
const ACCORDION_LAYOUTPOLICY_WRAPCONTENT: i32 = 1;
const ACCORDION_LAYOUTPOLICY_FIXATIDEALSIZE: i32 = 2;

fn accordion_panel_animation() -> Animation {
    let animation = Animation::new();
    animation.duration(ACCORDION_CONTENT_TRANSITION_MS);
    animation.iterations(1);
    animation.tempo(1.0);
    animation.curve(Curve::EaseOut);
    animation.mode(AnimationMode::Normal);
    animation
}

fn accordion_chevron_animation(is_open: bool) -> Animation {
    let animation = Animation::new();
    animation.duration(if is_open {
        ACCORDION_CHEVRON_OPEN_MS
    } else {
        ACCORDION_CHEVRON_CLOSE_MS
    });
    animation.iterations(1);
    animation.tempo(1.0);
    animation.curve(Curve::EaseOut);
    animation.mode(AnimationMode::Normal);
    animation
}

fn attribute_number_to_f32(value: ArkUINodeAttributeNumber) -> f32 {
    match value {
        ArkUINodeAttributeNumber::Float(value) => value,
        ArkUINodeAttributeNumber::Int(value) => value as f32,
        ArkUINodeAttributeNumber::Uint(value) => value as f32,
    }
}

fn accordion_layout_height(node: &ArkUINode) -> Option<f32> {
    match node.get_layout_rect().ok()? {
        ArkUINodeAttributeItem::NumberValue(values) => values
            .get(3)
            .map(|value| attribute_number_to_f32(*value).max(0.0)),
        _ => None,
    }
}

fn same_height_target(previous: Option<f32>, current: Option<f32>) -> bool {
    match (previous, current) {
        (Some(previous), Some(current)) => (previous - current).abs() < 0.5,
        (None, None) => true,
        _ => false,
    }
}

fn apply_panel_height<T>(node: &mut T, height: Option<f32>) -> ArkUIResult<()>
where
    T: ArkUICommonAttribute + 'static,
{
    if let Some(height) = height {
        node.set_i32_attribute(
            ArkUINodeAttributeType::HeightLayoutpolicy,
            ACCORDION_LAYOUTPOLICY_FIXATIDEALSIZE,
        )?;
        node.height(height.max(0.0))?;
    } else {
        node.set_i32_attribute(
            ArkUINodeAttributeType::HeightLayoutpolicy,
            ACCORDION_LAYOUTPOLICY_WRAPCONTENT,
        )?;
    }
    Ok(())
}

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
        Self::from_parts(
            value,
            accordion_trigger_text(title),
            accordion_content(content),
        )
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

fn accordion_chevron(is_open_signal: Signal<bool>) -> Element {
    let initial = is_open_signal.get();
    let initial_angle = if initial {
        ACCORDION_CHEVRON_ROTATION
    } else {
        0.0
    };

    arkit::row_component()
        .width(ACCORDION_ICON_SIZE)
        .height(ACCORDION_ICON_SIZE)
        .align_items_center()
        .style(ArkUINodeAttributeType::RowJustifyContent, FLEX_ALIGN_CENTER)
        .native_with_cleanup(move |node| {
            node.set_transform_center(vec![0.0, 0.0, 0.0, 0.5, 0.5, 0.0])?;
            node.set_rotate(vec![0.0, 0.0, 1.0, initial_angle, 0.0])?;

            let active = Rc::new(Cell::new(true));
            let cleanup_active = active.clone();
            let animated_node = Rc::new(RefCell::new(node.borrow_mut().clone()));
            let signal = is_open_signal.clone();
            let mut prev = initial;

            create_effect(move || {
                let current = signal.get();
                if current == prev || !active.get() {
                    return;
                }
                prev = current;

                let target_angle = if current {
                    ACCORDION_CHEVRON_ROTATION
                } else {
                    0.0
                };
                let animated_node = animated_node.clone();
                let animation_slot = Rc::new(RefCell::new(None::<Animation>));

                let animation = accordion_chevron_animation(current);
                let update_node = animated_node.clone();
                let update_active = active.clone();

                animation.update(move || {
                    if !update_active.get() {
                        return;
                    }
                    let node = update_node.borrow_mut();
                    if let Err(error) = node.set_rotate(vec![0.0, 0.0, 1.0, target_angle, 0.0]) {
                        eprintln!("accordion chevron error: failed to animate rotation: {error}");
                    }
                });

                let finish_slot = animation_slot.clone();
                animation.finish(AnimationFinishCallbackType::Logically, move || {
                    let release_slot = finish_slot.clone();
                    queue_ui_loop(move || {
                        let _ = release_slot.borrow_mut().take();
                    });
                });

                let animation_node = animated_node.borrow().clone();
                if let Err(error) = animation_node.animate_to(&animation) {
                    eprintln!(
                        "accordion chevron error: failed to start rotation animation: {error}"
                    );
                } else {
                    *animation_slot.borrow_mut() = Some(animation);
                }
            });

            Ok(move || {
                cleanup_active.set(false);
            })
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

fn accordion_content_body(
    content: Vec<Element>,
    measured_height: Signal<f32>,
    has_measured_height: Signal<bool>,
    measure_enabled: bool,
) -> Element {
    arkit::column_component()
        .percent_width(1.0)
        .align_items_start()
        .style(
            ArkUINodeAttributeType::Padding,
            vec![0.0, 0.0, spacing::LG, 0.0],
        )
        .native_with_cleanup(move |node| {
            let measured_node = Rc::new(RefCell::new(node.borrow_mut().clone()));
            let active = Rc::new(Cell::new(true));
            let update_measurement: Rc<dyn Fn()> = Rc::new({
                let measured_node = measured_node.clone();
                let active = active.clone();
                let measured_height = measured_height.clone();
                let has_measured_height = has_measured_height.clone();
                move || {
                    if !active.get() || !measure_enabled {
                        return;
                    }

                    let Some(height) = accordion_layout_height(&measured_node.borrow()) else {
                        return;
                    };
                    if height <= 0.5 {
                        return;
                    }

                    let previous_height = measured_height.get();
                    let previous_measured = has_measured_height.get();
                    if !previous_measured || (previous_height - height).abs() >= 0.5 {
                        measured_height.set(height);
                    }
                    if !previous_measured {
                        has_measured_height.set(true);
                    }
                }
            });

            node.on_area_change({
                let update_measurement = update_measurement.clone();
                move |_| update_measurement()
            });

            node.on_size_change({
                let update_measurement = update_measurement.clone();
                move |_| update_measurement()
            });

            queue_after_mount({
                let update_measurement = update_measurement.clone();
                move || {
                    queue_ui_loop(move || {
                        update_measurement();
                    });
                }
            });

            Ok(move || {
                active.set(false);
            })
        })
        .children(content)
        .into()
}

fn accordion_content_panel(
    content: Vec<Element>,
    is_open_signal: Signal<bool>,
    measured_height: Signal<f32>,
    has_measured_height: Signal<bool>,
) -> Element {
    let initial_open = is_open_signal.get();
    let initial_measured = has_measured_height.get();
    let initial_height = measured_height.get();

    let initial_target: Option<f32> = if initial_open {
        if initial_measured {
            Some(initial_height)
        } else {
            None
        }
    } else {
        Some(0.0)
    };

    // Clone signals for the children (before the closure consumes them)
    let body_measured = measured_height.clone();
    let body_has_measured = has_measured_height.clone();

    arkit::column_component()
        .percent_width(1.0)
        .align_items_start()
        .style(ArkUINodeAttributeType::Clip, true)
        .native_with_cleanup(move |node| {
            let active = Rc::new(Cell::new(true));
            let cleanup_active = active.clone();

            // Set initial height
            if let Some(target) = initial_target {
                apply_panel_height(node, Some(target))?;
            }

            let node_ref = Rc::new(RefCell::new(node.borrow_mut().clone()));
            let prev_target: Rc<RefCell<Option<f32>>> = Rc::new(RefCell::new(initial_target));
            let prev_open = Rc::new(Cell::new(initial_open));
            let prev_measured = Rc::new(Cell::new(initial_measured));

            let sig = is_open_signal.clone();
            let m_height = measured_height.clone();
            let has_m = has_measured_height.clone();

            create_effect(move || {
                let current_open = sig.get();
                let measured = has_m.get();
                let content_height = m_height.get();

                let open_changed = current_open != prev_open.get();
                let just_measured = measured && !prev_measured.get();

                if !open_changed && !just_measured {
                    return;
                }
                if !active.get() {
                    return;
                }

                prev_open.set(current_open);
                prev_measured.set(measured);

                let target_height: Option<f32> = if current_open {
                    if measured {
                        Some(content_height)
                    } else {
                        None
                    }
                } else {
                    Some(0.0)
                };

                let previous = *prev_target.borrow();

                // Animate only when we have both previous and target heights
                let should_animate = previous.is_some()
                    && target_height.is_some()
                    && !same_height_target(previous, target_height);

                if should_animate {
                    let from_h = previous.unwrap();
                    let to_h = target_height.unwrap();

                    let node = node_ref.borrow().clone();
                    let _ = node.set_i32_attribute(
                        ArkUINodeAttributeType::HeightLayoutpolicy,
                        ACCORDION_LAYOUTPOLICY_FIXATIDEALSIZE,
                    );
                    let _ = node.height(from_h.max(0.0));

                    let animated_node = Rc::new(RefCell::new(node));
                    let animation_slot = Rc::new(RefCell::new(None::<Animation>));

                    let animation = accordion_panel_animation();
                    let update_node = animated_node.clone();
                    let update_active = active.clone();
                    animation.update(move || {
                        if !update_active.get() {
                            return;
                        }
                        let node = update_node.borrow_mut();
                        if let Err(error) = node.height(to_h) {
                            eprintln!("accordion panel error: failed to animate height: {error}");
                        }
                    });

                    let finish_slot = animation_slot.clone();
                    animation.finish(AnimationFinishCallbackType::Logically, move || {
                        let release_slot = finish_slot.clone();
                        queue_ui_loop(move || {
                            let _ = release_slot.borrow_mut().take();
                        });
                    });

                    let animation_node = animated_node.borrow().clone();
                    if let Err(error) = animation_node.animate_to(&animation) {
                        eprintln!(
                            "accordion panel error: failed to start height animation: {error}"
                        );
                    } else {
                        *animation_slot.borrow_mut() = Some(animation);
                    }
                } else if let Some(h) = target_height {
                    let node = node_ref.borrow().clone();
                    let _ = apply_panel_height_on_node(&node, Some(h));
                } else if current_open {
                    // Not measured yet — use wrap content so body can measure
                    let node = node_ref.borrow().clone();
                    let _ = apply_panel_height_on_node(&node, None);
                }

                if target_height.is_some() && !same_height_target(previous, target_height) {
                    *prev_target.borrow_mut() = target_height;
                }
            });

            Ok(move || {
                cleanup_active.set(false);
            })
        })
        .children(vec![accordion_content_body(
            content,
            body_measured.clone(),
            body_has_measured.clone(),
            true, // always enable measurement
        )])
        .into()
}

fn apply_panel_height_on_node(node: &ArkUINode, height: Option<f32>) -> ArkUIResult<()> {
    if let Some(height) = height {
        node.set_i32_attribute(
            ArkUINodeAttributeType::HeightLayoutpolicy,
            ACCORDION_LAYOUTPOLICY_FIXATIDEALSIZE,
        )?;
        node.height(height.max(0.0))
    } else {
        node.set_i32_attribute(
            ArkUINodeAttributeType::HeightLayoutpolicy,
            ACCORDION_LAYOUTPOLICY_WRAPCONTENT,
        )?;
        Ok(())
    }
}

#[component]
fn accordion_item_view(
    trigger: Element,
    is_open_signal: Signal<bool>,
    disabled: bool,
    on_toggle: AccordionToggleHandler,
    content: Vec<Element>,
) -> Element {
    let measured_height = create_signal(0.0_f32);
    let has_measured_height = create_signal(false);

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
            accordion_chevron(is_open_signal.clone()),
        ]);

    if disabled {
        trigger_row = trigger_row
            .style(ArkUINodeAttributeType::Enabled, false)
            .style(ArkUINodeAttributeType::Opacity, 0.5_f32);
    } else {
        trigger_row = trigger_row.on_click(move || on_toggle());
    }

    let children = vec![
        trigger_row.into(),
        accordion_content_panel(
            content,
            is_open_signal,
            measured_height,
            has_measured_height,
        ),
    ];

    arkit::column_component()
        .percent_width(1.0)
        .style(
            ArkUINodeAttributeType::BorderWidth,
            vec![0.0, 0.0, 1.0, 0.0],
        )
        .style(ArkUINodeAttributeType::BorderColor, vec![color::BORDER])
        .children(children)
        .into()
}

fn render_single_items(
    items: Vec<AccordionItemSpec>,
    open_item: Signal<Option<String>>,
    collapsible: bool,
    on_value_change: Option<AccordionSingleChangeHandler>,
) -> Element {
    let children = items
        .into_iter()
        .map(|item| {
            let next_value = item.value.clone();
            let signal = open_item.clone();
            let callback = on_value_change.clone();

            // Create derived signal for this item's open state
            let is_open_signal =
                create_signal(open_item.get().as_deref() == Some(item.value.as_str()));
            {
                let oi = open_item.clone();
                let v = item.value.clone();
                let ios = is_open_signal.clone();
                create_effect(move || {
                    ios.set(oi.get().as_deref() == Some(v.as_str()));
                });
            }

            accordion_item_view(
                item.trigger,
                is_open_signal,
                item.disabled,
                Rc::new(move || {
                    let next = if signal.get().as_deref() == Some(next_value.as_str()) {
                        if collapsible {
                            None
                        } else {
                            Some(next_value.clone())
                        }
                    } else {
                        Some(next_value.clone())
                    };

                    signal.set(next.clone());
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
    open_items: Signal<Vec<String>>,
    on_value_change: Option<AccordionMultipleChangeHandler>,
) -> Element {
    let children = items
        .into_iter()
        .map(|item| {
            let value = item.value.clone();
            let signal = open_items.clone();
            let callback = on_value_change.clone();

            // Create derived signal for this item's open state
            let is_open_signal = create_signal(signal.get().contains(&item.value));
            {
                let sig = signal.clone();
                let v = value.clone();
                let ios = is_open_signal.clone();
                create_effect(move || {
                    ios.set(sig.get().contains(&v));
                });
            }

            accordion_item_view(
                item.trigger,
                is_open_signal,
                item.disabled,
                Rc::new(move || {
                    let mut next = signal.get();
                    if let Some(index) = next.iter().position(|item| item == &value) {
                        next.remove(index);
                    } else {
                        next.push(value.clone());
                    }

                    signal.set(next.clone());
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
    value: Signal<AccordionValue>,
    spec: AccordionRootSpec,
) -> Element {
    match spec.accordion_type {
        AccordionType::Single => {
            let children = items
                .into_iter()
                .map(|item| {
                    let item_value = item.value.clone();
                    let next_value = item_value.clone();
                    let signal = value.clone();
                    let callback = spec.on_value_change.clone();
                    let collapsible = spec.collapsible;

                    let is_open_signal = create_signal(
                        value.get().as_single().as_deref() == Some(item.value.as_str()),
                    );
                    {
                        let sig = signal.clone();
                        let v = item.value.clone();
                        let ios = is_open_signal.clone();
                        create_effect(move || {
                            ios.set(sig.get().as_single().as_deref() == Some(v.as_str()));
                        });
                    }

                    accordion_item_view(
                        item.trigger,
                        is_open_signal,
                        item.disabled,
                        Rc::new(move || {
                            let current = signal.get().as_single();
                            let next = if current.as_deref() == Some(next_value.as_str()) {
                                if collapsible {
                                    None
                                } else {
                                    Some(next_value.clone())
                                }
                            } else {
                                Some(next_value.clone())
                            };

                            let next_state = AccordionValue::Single(next.clone());
                            signal.set(next_state.clone());
                            if let Some(handler) = callback.as_ref() {
                                handler(next_state);
                            }
                        }),
                        item.content,
                    )
                })
                .collect::<Vec<_>>();

            accordion_container(children)
        }
        AccordionType::Multiple => {
            let children = items
                .into_iter()
                .map(|item| {
                    let item_value = item.value.clone();
                    let signal = value.clone();
                    let callback = spec.on_value_change.clone();

                    let is_open_signal =
                        create_signal(value.get().as_multiple().contains(&item.value));
                    {
                        let sig = signal.clone();
                        let v = item.value.clone();
                        let ios = is_open_signal.clone();
                        create_effect(move || {
                            ios.set(sig.get().as_multiple().contains(&v));
                        });
                    }

                    accordion_item_view(
                        item.trigger,
                        is_open_signal,
                        item.disabled,
                        Rc::new(move || {
                            let mut next = signal.get().as_multiple();
                            if let Some(index) = next.iter().position(|value| value == &item_value)
                            {
                                next.remove(index);
                            } else {
                                next.push(item_value.clone());
                            }

                            let next_state = AccordionValue::Multiple(next);
                            signal.set(next_state.clone());
                            if let Some(handler) = callback.as_ref() {
                                handler(next_state);
                            }
                        }),
                        item.content,
                    )
                })
                .collect::<Vec<_>>();

            accordion_container(children)
        }
    }
}

pub fn accordion(children: Vec<Element>) -> Element {
    accordion_container(children)
}

pub fn accordion_item(
    title: impl Into<String>,
    value: impl Into<String>,
    open_item: Signal<Option<String>>,
    content: Vec<Element>,
) -> Element {
    let value = value.into();
    let click_value = value.clone();
    let click = open_item.clone();

    let is_open_signal = create_signal(open_item.get().as_deref() == Some(value.as_str()));
    {
        let oi = open_item.clone();
        let v = value.clone();
        let ios = is_open_signal.clone();
        create_effect(move || {
            ios.set(oi.get().as_deref() == Some(v.as_str()));
        });
    }

    accordion_item_view(
        accordion_trigger_text(title).child,
        is_open_signal,
        false,
        Rc::new(move || {
            click.update(|current| {
                if current.as_deref() == Some(click_value.as_str()) {
                    *current = None;
                } else {
                    *current = Some(click_value.clone());
                }
            })
        }),
        content,
    )
}

#[component]
pub fn accordion_root(items: Vec<AccordionItemSpec>, spec: AccordionRootSpec) -> Element {
    let default_value = spec.default_value.clone();
    let value = create_signal(default_value);
    render_root_items(items, value, spec)
}

pub fn accordion_root_controlled(
    items: Vec<AccordionItemSpec>,
    value: Signal<AccordionValue>,
    spec: AccordionRootSpec,
) -> Element {
    render_root_items(items, value, spec)
}

#[component]
pub fn accordion_single(
    items: Vec<AccordionItemSpec>,
    collapsible: bool,
    default_value: Option<String>,
) -> Element {
    let open_item = create_signal(default_value);
    render_single_items(items, open_item, collapsible, None)
}

pub fn accordion_single_controlled(
    items: Vec<AccordionItemSpec>,
    value: Signal<Option<String>>,
    collapsible: bool,
    on_value_change: Option<AccordionSingleChangeHandler>,
) -> Element {
    render_single_items(items, value, collapsible, on_value_change)
}

#[component]
pub fn accordion_multiple(items: Vec<AccordionItemSpec>, default_value: Vec<String>) -> Element {
    let open_items = create_signal(default_value);
    render_multiple_items(items, open_items, None)
}

pub fn accordion_multiple_controlled(
    items: Vec<AccordionItemSpec>,
    value: Signal<Vec<String>>,
    on_value_change: Option<AccordionMultipleChangeHandler>,
) -> Element {
    render_multiple_items(items, value, on_value_change)
}
