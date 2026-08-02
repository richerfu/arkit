//! Application and exact-element lifecycle hooks.

use std::cell::RefCell;
use std::rc::Rc;

use arkit_arkui::{NativeElementEvent, NativeElementRef, NativeVisibility};
use arkit_prelude::*;
use arkit_runtime::{
    ApplicationLifecycleEvent, ApplicationLifecycleHandle, ApplicationLifecycleState,
    ApplicationLifecycleSubscription,
};

use crate::node::use_native_element_events;

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

/// Native presentation state for one exact mounted element.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ComponentLifecycleState {
    pub visible: bool,
    pub visible_fraction: f32,
}

impl ComponentLifecycleState {
    pub fn is_visible(self) -> bool {
        self.visible && self.visible_fraction > 0.0
    }
}

impl From<NativeVisibility> for ComponentLifecycleState {
    fn from(value: NativeVisibility) -> Self {
        let visible_fraction = if value.fraction.is_finite() {
            value.fraction.clamp(0.0, 1.0)
        } else {
            0.0
        };
        Self {
            visible: value.visible && visible_fraction > f32::EPSILON,
            visible_fraction,
        }
    }
}

/// Observe show/hide state for the element carrying `reference`.
///
/// The same handle must be assigned to that element's `native_ref` attribute.
#[track_caller]
pub fn use_component_lifecycle(reference: NativeElementRef) -> ComponentLifecycleState {
    // Prime renderer metadata before RSX assigns this ref to a native node.
    reference.request_visibility_observation();
    let lifecycle = use_signal(ComponentLifecycleState::default);
    use_native_element_events(reference, move |event| {
        let callback_signal = lifecycle;
        let next = match event {
            Some(NativeElementEvent::Visibility { visibility, .. }) => visibility.into(),
            None | Some(NativeElementEvent::Unmounted { .. }) => ComponentLifecycleState::default(),
            Some(NativeElementEvent::Mounted(_) | NativeElementEvent::Layout { .. }) => return,
        };
        let mut lifecycle = callback_signal;
        if lifecycle.peek().ne(&next) {
            lifecycle.set(next);
        }
    });

    lifecycle()
}

/// Shorthand for [`use_component_lifecycle`]'s visible state.
#[track_caller]
pub fn use_component_visibility(reference: NativeElementRef) -> bool {
    use_component_lifecycle(reference).is_visible()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_visible_fraction_is_hidden() {
        let state = ComponentLifecycleState::from(NativeVisibility {
            visible: true,
            fraction: 0.0,
        });
        assert!(!state.is_visible());
    }

    #[test]
    fn decreasing_visible_area_remains_visible_above_zero() {
        let state = ComponentLifecycleState::from(NativeVisibility {
            visible: true,
            fraction: 0.5,
        });
        assert!(state.is_visible());
        assert_eq!(state.visible_fraction, 0.5);
    }

    #[test]
    fn invalid_visible_fraction_is_hidden() {
        assert_eq!(
            ComponentLifecycleState::from(NativeVisibility {
                visible: true,
                fraction: f32::NAN,
            }),
            ComponentLifecycleState::default()
        );
    }

    #[test]
    fn foreground_default_supports_hosts_that_mounted_after_show() {
        assert!(ApplicationLifecycleState::default().is_foreground());
    }
}
