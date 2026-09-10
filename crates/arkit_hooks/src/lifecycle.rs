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

/// Mount and visibility observation for one exact native element.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum ComponentLifecycleState {
    /// The ref is not currently bound to a native node.
    #[default]
    Unmounted,
    /// Mounted, but ArkUI has not reported actual visibility yet.
    Unknown,
    /// ArkUI reported that the mounted node has no visible area.
    Hidden,
    /// ArkUI reported a positive visible fraction.
    Visible { fraction: f32 },
}

impl ComponentLifecycleState {
    /// Whether ArkUI has positively observed this mounted element as visible.
    ///
    /// `Unknown` is deliberately false: callers deciding whether to run
    /// finite or continuous work must choose their own unknown-state policy.
    pub const fn is_known_visible(self) -> bool {
        matches!(self, Self::Visible { .. })
    }

    pub const fn visible_fraction(self) -> Option<f32> {
        match self {
            Self::Visible { fraction } => Some(fraction),
            Self::Unmounted | Self::Unknown | Self::Hidden => None,
        }
    }
}

impl From<NativeVisibility> for ComponentLifecycleState {
    fn from(value: NativeVisibility) -> Self {
        let visible_fraction = if value.fraction.is_finite() {
            value.fraction.clamp(0.0, 1.0)
        } else {
            0.0
        };
        if value.visible && visible_fraction > f32::EPSILON {
            Self::Visible {
                fraction: visible_fraction,
            }
        } else {
            Self::Hidden
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
            Some(NativeElementEvent::Mounted(_)) => ComponentLifecycleState::Unknown,
            Some(NativeElementEvent::Visibility { visibility, .. }) => visibility.into(),
            None | Some(NativeElementEvent::Unmounted { .. }) => ComponentLifecycleState::Unmounted,
            Some(NativeElementEvent::Layout { .. }) => return,
        };
        let mut lifecycle = callback_signal;
        if lifecycle.peek().ne(&next) {
            lifecycle.set(next);
        }
    });

    lifecycle()
}

/// Whether ArkUI has positively observed the exact element as visible.
///
/// This returns `false` for unmounted, unknown, and observed-hidden states.
/// Use [`use_component_lifecycle`] when the unknown state needs a distinct
/// finite-work policy.
#[track_caller]
pub fn use_component_visibility(reference: NativeElementRef) -> bool {
    use_component_lifecycle(reference).is_known_visible()
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
        assert_eq!(state, ComponentLifecycleState::Hidden);
    }

    #[test]
    fn decreasing_visible_area_remains_visible_above_zero() {
        let state = ComponentLifecycleState::from(NativeVisibility {
            visible: true,
            fraction: 0.5,
        });
        assert!(state.is_known_visible());
        assert_eq!(state.visible_fraction(), Some(0.5));
    }

    #[test]
    fn invalid_visible_fraction_is_hidden() {
        assert_eq!(
            ComponentLifecycleState::from(NativeVisibility {
                visible: true,
                fraction: f32::NAN,
            }),
            ComponentLifecycleState::Hidden
        );
    }

    #[test]
    fn unknown_is_not_treated_as_visible() {
        assert!(!ComponentLifecycleState::Unknown.is_known_visible());
        assert_eq!(ComponentLifecycleState::Unknown.visible_fraction(), None);
    }

    #[test]
    fn foreground_default_supports_hosts_that_mounted_after_show() {
        assert!(ApplicationLifecycleState::default().is_foreground());
    }
}
