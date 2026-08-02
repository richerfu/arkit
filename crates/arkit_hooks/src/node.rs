//! Exact RSX element references.

use std::cell::RefCell;
use std::rc::Rc;

use arkit_arkui::{
    MountedNodeLease, NativeElementEvent, NativeElementRef, NativeElementSubscription,
};
use arkit_prelude::{use_drop, use_effect, use_hook, use_reactive};

/// Allocate a stable handle that must be attached to an exact RSX element via
/// its `native_ref` attribute.
#[track_caller]
pub fn use_native_element_ref() -> NativeElementRef {
    use_hook(NativeElementRef::new)
}

type NativeElementHookCallback = Rc<dyn Fn(Option<NativeElementEvent>)>;
type SharedNativeElementHookCallback = Rc<RefCell<NativeElementHookCallback>>;

struct NativeElementHookState {
    callback: SharedNativeElementHookCallback,
    subscription: Rc<RefCell<Option<NativeElementSubscription>>>,
}

impl Clone for NativeElementHookState {
    fn clone(&self) -> Self {
        Self {
            callback: self.callback.clone(),
            subscription: self.subscription.clone(),
        }
    }
}

/// Subscribe once per exact ref while always dispatching to the latest render
/// callback. `None` marks a non-reactive ref replacement boundary.
pub(crate) fn use_native_element_events(
    reference: NativeElementRef,
    callback: impl Fn(Option<NativeElementEvent>) + 'static,
) {
    let next = Rc::new(callback) as NativeElementHookCallback;
    let initial = next.clone();
    let state = use_hook(move || NativeElementHookState {
        callback: Rc::new(RefCell::new(initial)),
        subscription: Rc::new(RefCell::new(None)),
    });
    *state.callback.borrow_mut() = next;

    let effect_state = state.clone();
    use_effect(use_reactive(&reference, move |reference| {
        if effect_state.subscription.borrow_mut().take().is_some() {
            let callback = effect_state.callback.borrow().clone();
            callback(None);
        }

        let callback = effect_state.callback.clone();
        let subscription = reference.subscribe(move |event| {
            let callback = callback.borrow().clone();
            callback(Some(event));
        });
        effect_state.subscription.replace(Some(subscription));
    }));

    let cleanup = state.subscription.clone();
    use_drop(move || {
        cleanup.borrow_mut().take();
    });
}

/// Observe mount/rebind/unmount for an exact element reference.
///
/// Advanced native components use the generation-checked lease inside the
/// callback. Normal application code should prefer layout/lifecycle hooks.
#[track_caller]
pub fn use_mounted_node(
    reference: NativeElementRef,
    callback: impl Fn(Option<MountedNodeLease>) + 'static,
) {
    use_native_element_events(reference, move |event| match event {
        Some(NativeElementEvent::Mounted(lease)) => callback(Some(lease)),
        None | Some(NativeElementEvent::Unmounted { .. }) => callback(None),
        Some(NativeElementEvent::Layout { .. } | NativeElementEvent::Visibility { .. }) => {}
    });
}
