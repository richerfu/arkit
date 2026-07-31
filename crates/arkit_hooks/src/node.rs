//! Exact RSX element references.

use std::cell::RefCell;
use std::rc::Rc;

use arkit_arkui::{
    MountedNodeLease, NativeElementEvent, NativeElementRef, NativeElementSubscription,
};
use arkit_prelude::{use_drop, use_effect, use_hook};

/// Allocate a stable handle that must be attached to an exact RSX element via
/// its `native_ref` attribute.
#[track_caller]
pub fn use_native_element_ref() -> NativeElementRef {
    use_hook(NativeElementRef::new)
}

type MountedNodeCallback = Rc<dyn Fn(Option<MountedNodeLease>)>;
type SharedMountedNodeCallback = Rc<RefCell<MountedNodeCallback>>;

struct NativeNodeHookState {
    callback: SharedMountedNodeCallback,
    subscription: Rc<RefCell<Option<NativeElementSubscription>>>,
}

impl Clone for NativeNodeHookState {
    fn clone(&self) -> Self {
        Self {
            callback: self.callback.clone(),
            subscription: self.subscription.clone(),
        }
    }
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
    let next = Rc::new(callback) as MountedNodeCallback;
    let initial = next.clone();
    let state = use_hook(move || NativeNodeHookState {
        callback: Rc::new(RefCell::new(initial)),
        subscription: Rc::new(RefCell::new(None)),
    });
    *state.callback.borrow_mut() = next;

    let effect_state = state.clone();
    use_effect(move || {
        let callback = effect_state.callback.clone();
        let subscription = reference.subscribe(move |event| match event {
            NativeElementEvent::Mounted(lease) => callback.borrow().clone()(Some(lease)),
            NativeElementEvent::Unmounted { .. } => callback.borrow().clone()(None),
            NativeElementEvent::Layout { .. } | NativeElementEvent::Visibility { .. } => {}
        });
        effect_state.subscription.replace(Some(subscription));
    });

    let cleanup = state.subscription.clone();
    use_drop(move || {
        cleanup.borrow_mut().take();
    });
}
