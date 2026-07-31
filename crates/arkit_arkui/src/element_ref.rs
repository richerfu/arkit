//! Exact element-to-native binding with generation-checked leases.

use std::cell::RefCell;
use std::rc::{Rc, Weak};

use dioxus_core::{AttributeValue, IntoAttributeValue};
use ohos_arkui_binding::api::node_custom_event::{IntOffset, IntSize};
use ohos_arkui_binding::common::node::ArkUINode;
use rustc_hash::FxHashMap;

pub(crate) type SharedNativeNode = Rc<RefCell<ArkUINode>>;

/// A measured frame in physical pixels, relative to the window.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LayoutFramePx {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl LayoutFramePx {
    pub fn is_measured(self) -> bool {
        self.width > 0.0 && self.height > 0.0
    }
}

/// Current native presentation state for an exact element.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct NativeVisibility {
    pub visible: bool,
    pub fraction: f32,
}

/// A generation-checked, non-owning lease for one mounted ArkUI node.
///
/// The lease does not expose `dispose` and becomes inert when the renderer
/// unmounts or rebinds the element.
#[derive(Clone)]
pub struct MountedNodeLease {
    reference: NativeElementRef,
    epoch: u64,
}

impl std::fmt::Debug for MountedNodeLease {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("MountedNodeLease")
            .field("epoch", &self.epoch)
            .field("mounted", &self.is_mounted())
            .finish()
    }
}

impl PartialEq for MountedNodeLease {
    fn eq(&self, other: &Self) -> bool {
        self.epoch == other.epoch && self.reference == other.reference
    }
}

impl MountedNodeLease {
    pub const fn epoch(&self) -> u64 {
        self.epoch
    }

    pub fn is_mounted(&self) -> bool {
        self.reference.node_for_epoch(self.epoch).is_some()
    }

    pub fn layout_frame_px(&self) -> Option<LayoutFramePx> {
        let node = self.reference.node_for_epoch(self.epoch)?;
        let node = node.borrow();
        let IntSize { width, height } = node.layout_size().ok()?;
        let IntOffset { x, y } = node
            .layout_position_in_window()
            .or_else(|_| node.position_with_translate_in_window())
            .ok()?;
        Some(LayoutFramePx {
            x: x as f32,
            y: y as f32,
            width: width as f32,
            height: height as f32,
        })
    }

    /// Execute an advanced native operation after validating the mount epoch.
    ///
    /// # Safety
    ///
    /// The callback must not clone, dispose, retain, reparent, or replace the
    /// renderer's normal node-event route. Integration-specific callbacks are
    /// allowed only when the native node itself owns their lifetime (for
    /// example custom draw), or when their separate owner is tied to this lease
    /// with [`Self::install_native_teardown`].
    pub unsafe fn with_native<R>(&self, operation: impl FnOnce(&ArkUINode) -> R) -> Option<R> {
        let node = self.reference.node_for_epoch(self.epoch)?;
        let borrowed = node.borrow();
        Some(operation(&borrowed))
    }

    /// Execute a mutating advanced native operation after validating the epoch.
    ///
    /// # Safety
    ///
    /// In addition to [`Self::with_native`] requirements, the callback must not
    /// call `dispose`, alter parentage, or replace renderer-routed node events.
    pub unsafe fn with_native_mut<R>(
        &self,
        operation: impl FnOnce(&mut ArkUINode) -> R,
    ) -> Option<R> {
        let node = self.reference.node_for_epoch(self.epoch)?;
        let mut borrowed = node.borrow_mut();
        Some(operation(&mut borrowed))
    }

    /// Install native-resource cleanup that runs synchronously before this
    /// lease is invalidated and before the renderer disposes the node.
    ///
    /// Multiple integration owners may register independent cleanups. Returns
    /// `false` when this lease is already stale.
    ///
    /// # Safety
    ///
    /// The cleanup must only release resources attached to this native node.
    /// It must not update Dioxus state, mutate the host tree, invoke this ref,
    /// or retain/use the node after returning.
    pub unsafe fn install_native_teardown(&self, cleanup: impl FnOnce() + 'static) -> bool {
        let mut state = self.reference.state.borrow_mut();
        if state.epoch != self.epoch || state.node.as_ref().and_then(Weak::upgrade).is_none() {
            return false;
        }
        state.native_teardowns.push((self.epoch, Box::new(cleanup)));
        true
    }
}

/// Event stream emitted for one exact mounted element.
#[derive(Clone, Debug, PartialEq)]
pub enum NativeElementEvent {
    Mounted(MountedNodeLease),
    Unmounted {
        epoch: u64,
    },
    Layout {
        epoch: u64,
        frame: LayoutFramePx,
    },
    Visibility {
        epoch: u64,
        visibility: NativeVisibility,
    },
}

/// Opaque renderer-to-runtime delivery capability.
///
/// Only the renderer can construct this value. Event sinks may queue it and
/// later consume it at a non-reentrant runtime boundary.
#[doc(hidden)]
pub struct NativeElementDelivery {
    reference: NativeElementRef,
    event: NativeElementEvent,
}

impl NativeElementDelivery {
    pub(crate) fn new(reference: NativeElementRef, event: NativeElementEvent) -> Self {
        Self { reference, event }
    }

    /// Deliver this renderer-created notification exactly once.
    pub fn deliver(self) {
        self.reference.deliver(self.event);
    }
}

type NativeElementCallback = Rc<dyn Fn(NativeElementEvent)>;

struct NativeElementSubscriber {
    callback: NativeElementCallback,
    epoch: u64,
    mounted: bool,
}

#[derive(Default)]
struct NativeElementState {
    epoch: u64,
    node: Option<Weak<RefCell<ArkUINode>>>,
    observe_layout: bool,
    observe_visibility: bool,
    layout: Option<LayoutFramePx>,
    visibility: NativeVisibility,
    native_teardowns: Vec<(u64, Box<dyn FnOnce()>)>,
    next_subscription: u64,
    subscribers: FxHashMap<u64, NativeElementSubscriber>,
}

/// Stable handle bound by the renderer to one exact RSX element.
#[derive(Clone, Default)]
pub struct NativeElementRef {
    state: Rc<RefCell<NativeElementState>>,
}

impl std::fmt::Debug for NativeElementRef {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("NativeElementRef")
            .field("mounted", &self.current().is_some())
            .finish_non_exhaustive()
    }
}

impl PartialEq for NativeElementRef {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.state, &other.state)
    }
}

impl Eq for NativeElementRef {}

impl IntoAttributeValue for NativeElementRef {
    fn into_value(self) -> AttributeValue {
        AttributeValue::any_value(self)
    }
}

impl NativeElementRef {
    pub fn new() -> Self {
        Self::default()
    }

    /// Declare that the hook using this ref needs native layout notifications.
    ///
    /// This is renderer metadata rather than a subscription. It must be set
    /// during render, before the ref is assigned to an element, so the
    /// renderer can install only the native events that are actually needed.
    #[doc(hidden)]
    pub fn request_layout_observation(&self) {
        self.state.borrow_mut().observe_layout = true;
    }

    /// Declare that the hook using this ref needs native visibility events.
    ///
    /// See [`Self::request_layout_observation`] for the timing contract.
    #[doc(hidden)]
    pub fn request_visibility_observation(&self) {
        self.state.borrow_mut().observe_visibility = true;
    }

    pub(crate) fn observes_layout(&self) -> bool {
        self.state.borrow().observe_layout
    }

    pub(crate) fn observes_visibility(&self) -> bool {
        self.state.borrow().observe_visibility
    }

    pub fn current(&self) -> Option<MountedNodeLease> {
        let state = self.state.borrow();
        state.node.as_ref()?.upgrade()?;
        Some(MountedNodeLease {
            reference: self.clone(),
            epoch: state.epoch,
        })
    }

    pub fn subscribe(
        &self,
        callback: impl Fn(NativeElementEvent) + 'static,
    ) -> NativeElementSubscription {
        let callback = Rc::new(callback) as NativeElementCallback;
        let (id, initial) = {
            let mut state = self.state.borrow_mut();
            state.next_subscription = state
                .next_subscription
                .checked_add(1)
                .expect("arkit_arkui: native element subscription space exhausted");
            let id = state.next_subscription;
            let lease = state
                .node
                .as_ref()
                .and_then(Weak::upgrade)
                .map(|_| MountedNodeLease {
                    reference: self.clone(),
                    epoch: state.epoch,
                });
            let epoch = state.epoch;
            state.subscribers.insert(
                id,
                NativeElementSubscriber {
                    callback: callback.clone(),
                    epoch,
                    mounted: lease.is_some(),
                },
            );
            let mut initial = Vec::with_capacity(3);
            if let Some(lease) = lease {
                initial.push(NativeElementEvent::Mounted(lease));
            }
            if let Some(layout) = state.layout {
                initial.push(NativeElementEvent::Layout {
                    epoch: state.epoch,
                    frame: layout,
                });
            }
            initial.push(NativeElementEvent::Visibility {
                epoch: state.epoch,
                visibility: state.visibility,
            });
            (id, initial)
        };
        for event in initial {
            callback(event);
        }
        NativeElementSubscription {
            reference: Rc::downgrade(&self.state),
            id,
        }
    }

    pub(crate) fn bind(&self, node: &SharedNativeNode) -> Option<NativeElementEvent> {
        let native_handle = node.borrow().raw_handle();
        let (unchanged, previous_teardowns) = {
            let mut state = self.state.borrow_mut();
            let unchanged = state
                .node
                .as_ref()
                .and_then(Weak::upgrade)
                .is_some_and(|current| current.borrow().raw_handle() == native_handle);
            let previous_teardowns = if unchanged {
                Vec::new()
            } else {
                std::mem::take(&mut state.native_teardowns)
            };
            (unchanged, previous_teardowns)
        };
        for (_, cleanup) in previous_teardowns {
            cleanup();
        }

        let mut state = self.state.borrow_mut();
        if !unchanged {
            state.epoch = state
                .epoch
                .checked_add(1)
                .expect("arkit_arkui: native element epoch space exhausted");
            state.node = Some(Rc::downgrade(node));
            state.layout = None;
            state.visibility = NativeVisibility::default();
        }
        // ArkUI child insertion can replace only the Rust `Rc` wrapper while
        // retaining the same native handle. Keep the current mounted wrapper
        // without invalidating leases for that ownership-neutral rewrap.
        state.node = Some(Rc::downgrade(node));
        (!unchanged).then(|| {
            NativeElementEvent::Mounted(MountedNodeLease {
                reference: self.clone(),
                epoch: state.epoch,
            })
        })
    }

    pub(crate) fn unbind(&self) -> Option<NativeElementEvent> {
        let (epoch, teardowns) = {
            let mut state = self.state.borrow_mut();
            state.node.as_ref().and_then(Weak::upgrade)?;
            let epoch = state.epoch;
            let teardowns = std::mem::take(&mut state.native_teardowns);
            (epoch, teardowns)
        };
        for (teardown_epoch, cleanup) in teardowns {
            if teardown_epoch == epoch {
                cleanup();
            }
        }

        let mut state = self.state.borrow_mut();
        if state.epoch != epoch {
            return None;
        }
        state.node = None;
        state.layout = None;
        state.visibility = NativeVisibility::default();
        state.epoch = state
            .epoch
            .checked_add(1)
            .expect("arkit_arkui: native element epoch space exhausted");
        Some(NativeElementEvent::Unmounted { epoch })
    }

    pub(crate) fn node_for_epoch(&self, epoch: u64) -> Option<SharedNativeNode> {
        let state = self.state.borrow();
        (state.epoch == epoch)
            .then(|| state.node.as_ref().and_then(Weak::upgrade))
            .flatten()
    }

    fn deliver(&self, event: NativeElementEvent) {
        let deliveries = {
            let mut state = self.state.borrow_mut();
            match &event {
                NativeElementEvent::Layout { epoch, frame }
                    if *epoch == state.epoch && state.node.is_some() =>
                {
                    state.layout = Some(*frame);
                }
                NativeElementEvent::Visibility { epoch, visibility }
                    if *epoch == state.epoch && state.node.is_some() =>
                {
                    state.visibility = *visibility;
                }
                NativeElementEvent::Mounted(_)
                | NativeElementEvent::Unmounted { .. }
                | NativeElementEvent::Layout { .. }
                | NativeElementEvent::Visibility { .. } => {}
            }
            state
                .subscribers
                .values_mut()
                .flat_map(|subscriber| {
                    let mut events = Vec::with_capacity(2);
                    match &event {
                        NativeElementEvent::Mounted(lease) => {
                            if lease.epoch < subscriber.epoch
                                || (lease.epoch == subscriber.epoch && subscriber.mounted)
                            {
                                // Stale or duplicate mount notification.
                            } else {
                                if subscriber.mounted {
                                    events.push(NativeElementEvent::Unmounted {
                                        epoch: subscriber.epoch,
                                    });
                                }
                                subscriber.epoch = lease.epoch;
                                subscriber.mounted = true;
                                events.push(event.clone());
                            }
                        }
                        NativeElementEvent::Unmounted { epoch } => {
                            if *epoch == subscriber.epoch && subscriber.mounted {
                                subscriber.mounted = false;
                                events.push(event.clone());
                            }
                        }
                        NativeElementEvent::Layout { epoch, .. }
                        | NativeElementEvent::Visibility { epoch, .. } => {
                            if *epoch == subscriber.epoch && subscriber.mounted {
                                events.push(event.clone());
                            }
                        }
                    }
                    events
                        .into_iter()
                        .map(|event| (subscriber.callback.clone(), event))
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>()
        };
        for (callback, event) in deliveries {
            callback(event);
        }
    }
}

/// RAII subscription to a [`NativeElementRef`] event stream.
pub struct NativeElementSubscription {
    reference: Weak<RefCell<NativeElementState>>,
    id: u64,
}

impl Drop for NativeElementSubscription {
    fn drop(&mut self) {
        if let Some(reference) = self.reference.upgrade() {
            reference.borrow_mut().subscribers.remove(&self.id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::NativeElementRef;

    #[test]
    fn native_observation_is_opt_in_and_sticky() {
        let reference = NativeElementRef::new();
        assert!(!reference.observes_layout());
        assert!(!reference.observes_visibility());

        reference.request_layout_observation();
        reference.request_layout_observation();
        assert!(reference.observes_layout());
        assert!(!reference.observes_visibility());

        reference.request_visibility_observation();
        assert!(reference.observes_layout());
        assert!(reference.observes_visibility());
    }
}
