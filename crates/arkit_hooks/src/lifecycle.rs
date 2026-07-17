//! Application and mounted-component lifecycle hooks.

use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

use arkit_prelude::*;
use arkit_runtime::{
    ApplicationLifecycleEvent, ApplicationLifecycleHandle, ApplicationLifecycleState,
    ApplicationLifecycleSubscription,
};
use ohos_arkui_binding::common::attribute::{ArkUINodeAttributeItem, ArkUINodeAttributeNumber};
use ohos_arkui_binding::common::node::ArkUINode;
use ohos_arkui_binding::component::attribute::{
    ArkUIAttributeBasic, ArkUICommonAttribute, ArkUIEvent,
};
use rustc_hash::FxHashMap;

use crate::node::{use_ark_node, HostNode};

#[derive(Clone, Copy)]
struct ApplicationLifecycleSignal(Signal<ApplicationLifecycleState>);

/// Install one reactive lifecycle signal for the entire Arkit tree.
pub(crate) fn use_application_lifecycle_provider() -> ApplicationLifecycleState {
    let handle = dioxus_core::try_consume_context::<ApplicationLifecycleHandle>();
    let signal = use_signal(|| {
        handle
            .as_ref()
            .map(ApplicationLifecycleHandle::get)
            .unwrap_or_default()
    });
    let _subscription = use_hook(|| {
        let callback_signal = signal;
        handle.clone().map(|handle| {
            handle.subscribe(move |_, state| {
                let mut signal = callback_signal;
                if signal.peek().ne(&state) {
                    signal.set(state);
                }
            })
        })
    });
    use_context_provider(|| ApplicationLifecycleSignal(signal));
    signal()
}

/// Read the current ability/window lifecycle snapshot reactively.
#[track_caller]
pub fn use_application_lifecycle() -> ApplicationLifecycleState {
    if let Some(signal) = dioxus_core::try_consume_context::<ApplicationLifecycleSignal>() {
        return (signal.0)();
    }
    dioxus_core::try_consume_context::<ApplicationLifecycleHandle>()
        .map(|handle| handle.get())
        .unwrap_or_default()
}

/// Whether foreground-only component work may run.
#[track_caller]
pub fn use_app_foreground() -> bool {
    use_application_lifecycle().is_foreground()
}

type ApplicationLifecycleCallback =
    Rc<dyn Fn(ApplicationLifecycleEvent, ApplicationLifecycleState)>;

#[derive(Clone)]
struct ApplicationLifecycleCallbackState {
    callback: Rc<RefCell<ApplicationLifecycleCallback>>,
    _subscription: Option<ApplicationLifecycleSubscription>,
}

/// Subscribe to normalized application lifecycle events for the current scope.
///
/// The subscription is removed automatically when the component unmounts.
#[track_caller]
pub fn use_application_lifecycle_event(
    callback: impl Fn(ApplicationLifecycleEvent, ApplicationLifecycleState) + 'static,
) {
    let next = Rc::new(callback) as ApplicationLifecycleCallback;
    let initial = next.clone();
    let handle = dioxus_core::try_consume_context::<ApplicationLifecycleHandle>();
    let state = use_hook(move || {
        let callback = Rc::new(RefCell::new(initial));
        let subscription_callback = callback.clone();
        let subscription = handle.map(|handle| {
            handle.subscribe(move |event, state| {
                subscription_callback.borrow().clone()(event, state);
            })
        });
        ApplicationLifecycleCallbackState {
            callback,
            _subscription: subscription,
        }
    });
    *state.callback.borrow_mut() = next;
}

/// Native visibility of a mounted component's root ArkUI node.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ComponentLifecycleState {
    /// Whether any part of the node is currently presented by ArkUI.
    pub visible: bool,
    /// Visible fraction reported by ArkUI, clamped to `0.0..=1.0`.
    pub visible_fraction: f32,
}

impl ComponentLifecycleState {
    pub fn is_visible(self) -> bool {
        self.visible && self.visible_fraction > 0.0
    }
}

type ComponentLifecycleCallback = Rc<dyn Fn(ComponentLifecycleState)>;

struct ComponentLifecycleObservers {
    generation: u64,
    node: Weak<RefCell<ArkUINode>>,
    state: ComponentLifecycleState,
    subscribers: FxHashMap<u64, ComponentLifecycleCallback>,
}

#[derive(Default)]
struct ComponentLifecycleHub {
    nodes: FxHashMap<usize, ComponentLifecycleObservers>,
    next_generation: u64,
    next_subscription: u64,
}

thread_local! {
    static COMPONENT_LIFECYCLE_HUB: RefCell<ComponentLifecycleHub> =
        RefCell::new(ComponentLifecycleHub::default());
}

struct ComponentLifecycleSubscription {
    node_key: usize,
    generation: u64,
    id: u64,
}

impl Drop for ComponentLifecycleSubscription {
    fn drop(&mut self) {
        COMPONENT_LIFECYCLE_HUB.with(|hub| {
            let mut hub = hub.borrow_mut();
            let Some(node) = hub.nodes.get_mut(&self.node_key) else {
                return;
            };
            if node.generation == self.generation {
                node.subscribers.remove(&self.id);
            }
        });
    }
}

fn component_node_key(node: &ArkUINode) -> usize {
    node.raw_handle() as usize
}

fn read_declared_visibility(node: &ArkUINode) -> bool {
    let Ok(ArkUINodeAttributeItem::NumberValue(values)) = node.get_visibility() else {
        return true;
    };
    match values.first() {
        Some(ArkUINodeAttributeNumber::Int(value)) => *value == 0,
        Some(ArkUINodeAttributeNumber::Uint(value)) => *value == 0,
        Some(ArkUINodeAttributeNumber::Float(value)) => value.abs() <= f32::EPSILON,
        None => true,
    }
}

fn subscribe_component_lifecycle(
    node: &HostNode,
    callback: ComponentLifecycleCallback,
) -> ComponentLifecycleSubscription {
    let node_key = component_node_key(&node.borrow());
    let visible = read_declared_visibility(&node.borrow());
    let initial = ComponentLifecycleState {
        visible,
        visible_fraction: if visible { 1.0 } else { 0.0 },
    };
    let (generation, id, install_listener, current) = COMPONENT_LIFECYCLE_HUB.with(|hub| {
        let mut hub = hub.borrow_mut();
        hub.nodes.retain(|_, entry| entry.node.strong_count() != 0);
        let same_node = hub.nodes.get(&node_key).is_some_and(|entry| {
            entry
                .node
                .upgrade()
                .is_some_and(|current| Rc::ptr_eq(&current, node))
        });
        let install_listener = !same_node;
        if install_listener {
            hub.next_generation = hub
                .next_generation
                .checked_add(1)
                .expect("arkit_hooks: component lifecycle generation space exhausted");
            let generation = hub.next_generation;
            hub.nodes.insert(
                node_key,
                ComponentLifecycleObservers {
                    generation,
                    node: Rc::downgrade(node),
                    state: initial,
                    subscribers: FxHashMap::default(),
                },
            );
        }
        hub.next_subscription = hub
            .next_subscription
            .checked_add(1)
            .expect("arkit_hooks: component lifecycle subscription id space exhausted");
        let id = hub.next_subscription;
        let entry = hub
            .nodes
            .get_mut(&node_key)
            .expect("component lifecycle hub entry must exist after insertion");
        entry.subscribers.insert(id, callback.clone());
        (entry.generation, id, install_listener, entry.state)
    });

    if install_listener {
        register_component_lifecycle_events(node, node_key, generation);
    }
    callback(current);

    ComponentLifecycleSubscription {
        node_key,
        generation,
        id,
    }
}

fn dispatch_component_lifecycle(
    node_key: usize,
    generation: u64,
    mut state: ComponentLifecycleState,
) {
    state.visible_fraction = state.visible_fraction.clamp(0.0, 1.0);
    if state.visible_fraction <= f32::EPSILON {
        state.visible = false;
        state.visible_fraction = 0.0;
    }
    let callbacks = COMPONENT_LIFECYCLE_HUB.with(|hub| {
        let mut hub = hub.borrow_mut();
        let entry = hub.nodes.get_mut(&node_key)?;
        if entry.generation != generation || entry.state == state {
            return None;
        }
        entry.state = state;
        Some(entry.subscribers.values().cloned().collect::<Vec<_>>())
    });
    if let Some(callbacks) = callbacks {
        for callback in callbacks {
            callback(state);
        }
    }
}

fn lifecycle_from_visible_fraction(visible_fraction: f32) -> ComponentLifecycleState {
    let visible_fraction = if visible_fraction.is_finite() {
        visible_fraction.clamp(0.0, 1.0)
    } else {
        0.0
    };
    ComponentLifecycleState {
        visible: visible_fraction > f32::EPSILON,
        visible_fraction,
    }
}

fn queue_component_lifecycle(node_key: usize, generation: u64, state: ComponentLifecycleState) {
    arkit_runtime::queue_ui_loop(move || {
        dispatch_component_lifecycle(node_key, generation, state);
    });
}

fn register_component_lifecycle_events(node: &HostNode, node_key: usize, generation: u64) {
    // A literal zero threshold can emit the pre-layout 0% snapshot without a
    // later transition when the node becomes visible. Use the smallest
    // practical positive threshold so mounting crosses it in both directions.
    const VISIBILITY_THRESHOLD: f32 = 0.001;
    let _ = node
        .borrow()
        .set_visible_area_change_ratio(vec![VISIBILITY_THRESHOLD]);

    let mut borrowed = node.borrow_mut();
    let mut event_node = LifecycleEventNode(&mut borrowed);
    event_node.on_appear(move || {
        queue_component_lifecycle(
            node_key,
            generation,
            ComponentLifecycleState {
                visible: true,
                visible_fraction: 1.0,
            },
        );
    });
    event_node.on_disappear(move || {
        queue_component_lifecycle(node_key, generation, ComponentLifecycleState::default());
    });
    event_node.on_visible_area_change(move |_increased, fraction| {
        queue_component_lifecycle(
            node_key,
            generation,
            lifecycle_from_visible_fraction(fraction),
        );
    });
}

struct LifecycleEventNode<'a>(&'a mut ArkUINode);

impl ArkUIAttributeBasic for LifecycleEventNode<'_> {
    fn raw(&self) -> &ArkUINode {
        self.0
    }

    fn borrow_mut(&mut self) -> &mut ArkUINode {
        self.0
    }
}

impl ArkUIEvent for LifecycleEventNode<'_> {}

struct ComponentLifecycleHookState {
    subscription: Rc<RefCell<Option<ComponentLifecycleSubscription>>>,
    node_key: Rc<Cell<Option<usize>>>,
}

impl Clone for ComponentLifecycleHookState {
    fn clone(&self) -> Self {
        Self {
            subscription: self.subscription.clone(),
            node_key: self.node_key.clone(),
        }
    }
}

/// Observe the current component root's show/hide lifecycle.
///
/// Mount and unmount remain scope lifecycle operations: use a mount-time
/// `use_effect` for creation and `use_drop` for destruction. This hook adds
/// the native visibility edge that Dioxus scope lifetime alone cannot express.
#[track_caller]
pub fn use_component_lifecycle() -> ComponentLifecycleState {
    let node_ref = use_ark_node();
    let mut lifecycle = use_signal(ComponentLifecycleState::default);
    let state = use_hook(|| ComponentLifecycleHookState {
        subscription: Rc::new(RefCell::new(None)),
        node_key: Rc::new(Cell::new(None)),
    });

    let effect_state = state.clone();
    use_effect(move || {
        let Some(node) = node_ref.get() else {
            effect_state.subscription.borrow_mut().take();
            effect_state.node_key.set(None);
            lifecycle.set(ComponentLifecycleState::default());
            return;
        };
        let key = component_node_key(&node.borrow());
        if effect_state.node_key.get() == Some(key) {
            return;
        }
        let callback_signal = lifecycle;
        let callback = Rc::new(move |state| {
            let mut lifecycle = callback_signal;
            if lifecycle.peek().ne(&state) {
                lifecycle.set(state);
            }
        });
        let subscription = subscribe_component_lifecycle(&node, callback);
        effect_state.subscription.replace(Some(subscription));
        effect_state.node_key.set(Some(key));
    });

    let cleanup = state.subscription.clone();
    use_drop(move || {
        cleanup.borrow_mut().take();
    });

    lifecycle()
}

/// Shorthand for [`use_component_lifecycle`]'s visible state.
#[track_caller]
pub fn use_component_visibility() -> bool {
    use_component_lifecycle().is_visible()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_visible_fraction_is_hidden() {
        let state = lifecycle_from_visible_fraction(0.0);
        assert!(!state.is_visible());
    }

    #[test]
    fn decreasing_visible_area_remains_visible_above_zero() {
        // ArkUI's first event value is the direction of the ratio change, not
        // the current visibility. A decreasing but non-zero ratio must remain
        // active so native components are not spuriously suspended.
        let state = lifecycle_from_visible_fraction(0.5);
        assert!(state.is_visible());
        assert_eq!(state.visible_fraction, 0.5);
    }

    #[test]
    fn invalid_visible_fraction_is_hidden() {
        assert_eq!(
            lifecycle_from_visible_fraction(f32::NAN),
            ComponentLifecycleState::default()
        );
    }

    #[test]
    fn foreground_default_supports_hosts_that_mounted_after_show() {
        assert!(ApplicationLifecycleState::default().is_foreground());
    }
}
