//! Layout-observer hooks: [`use_layout_size`], [`use_layout_frame`], and
//! [`use_layout_frame_node`].
//!
//! Each hook resolves the native ArkUI node backing the current dioxus element
//! (via [`crate::use_ark_node`]) and registers an ArkUI `onAreaChange` listener
//! on it. The callback is invoked with plain-data [`LayoutSize`] /
//! [`LayoutFrame`] whenever the measured layout changes meaningfully (>= 0.5
//! px). A change-dedup cache (keyed by native handle) suppresses noisy native
//! callbacks when layout hasn't meaningfully changed.
//!
//! The node handle is a shared `Rc<RefCell<ArkUINode>>` (the **same** `Rc`
//! mounted in the ArkUI tree and used as the event-dispatch user-data target).
//! Registering `onAreaChange` on it makes ArkUI's dispatcher find the callback,
//! so no polling fallback is needed — `onAreaChange` fires reliably once the
//! node is laid out.

use std::collections::BTreeMap;
use std::rc::Rc;
use std::sync::{Mutex, OnceLock};

use arkit_prelude::{use_drop, use_effect, use_hook};
use ohos_arkui_binding::api::node_custom_event::{IntOffset, IntSize};
use ohos_arkui_binding::common::node::ArkUINode;
use ohos_arkui_binding::component::attribute::{ArkUIAttributeBasic, ArkUIEvent};

use crate::node::{use_ark_node, HostNode};

/// A measured size in physical pixels.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LayoutSize {
    pub width: f32,
    pub height: f32,
}

impl LayoutSize {
    pub fn is_measured(self) -> bool {
        self.width > 0.0 && self.height > 0.0
    }
}

/// A measured frame (position + size) in physical pixels, relative to the
/// window.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LayoutFrame {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl LayoutFrame {
    pub fn is_measured(self) -> bool {
        self.width > 0.0 && self.height > 0.0
    }
}

// ---------------------------------------------------------------------------
// Change-dedup cache
// ---------------------------------------------------------------------------

static LAYOUT_SIZE_CACHE: OnceLock<Mutex<BTreeMap<usize, LayoutSize>>> = OnceLock::new();
static LAYOUT_FRAME_CACHE: OnceLock<Mutex<BTreeMap<usize, LayoutFrame>>> = OnceLock::new();

fn layout_observer_key(node: &ArkUINode) -> usize {
    node.raw_handle() as usize
}

fn emit_layout_size_if_changed(node: &ArkUINode, next: LayoutSize, on_change: &dyn Fn(LayoutSize)) {
    let key = layout_observer_key(node);
    if let Ok(mut cache) = LAYOUT_SIZE_CACHE
        .get_or_init(|| Mutex::new(BTreeMap::new()))
        .lock()
    {
        if cache
            .get(&key)
            .is_some_and(|prev| layout_size_close(*prev, next))
        {
            return;
        }
        cache.insert(key, next);
    }
    on_change(next);
}

fn emit_layout_frame_if_changed(
    node: &ArkUINode,
    next: LayoutFrame,
    on_change: &dyn Fn(LayoutFrame),
) {
    let key = layout_observer_key(node);
    if let Ok(mut cache) = LAYOUT_FRAME_CACHE
        .get_or_init(|| Mutex::new(BTreeMap::new()))
        .lock()
    {
        if cache
            .get(&key)
            .is_some_and(|prev| layout_frame_close(*prev, next))
        {
            return;
        }
        cache.insert(key, next);
    }
    on_change(next);
}

fn emit_layout_frame_node_if_changed(
    node: &ArkUINode,
    next: LayoutFrame,
    on_change: &dyn Fn(ArkUINode, LayoutFrame),
) {
    let key = layout_observer_key(node);
    if let Ok(mut cache) = LAYOUT_FRAME_CACHE
        .get_or_init(|| Mutex::new(BTreeMap::new()))
        .lock()
    {
        if cache
            .get(&key)
            .is_some_and(|prev| layout_frame_close(*prev, next))
        {
            return;
        }
        cache.insert(key, next);
    }
    on_change(node.clone(), next);
}

fn clear_layout_observer_cache(key: usize) {
    if let Some(cache) = LAYOUT_SIZE_CACHE.get() {
        if let Ok(mut cache) = cache.lock() {
            cache.remove(&key);
        }
    }
    if let Some(cache) = LAYOUT_FRAME_CACHE.get() {
        if let Ok(mut cache) = cache.lock() {
            cache.remove(&key);
        }
    }
}

fn layout_size_close(previous: LayoutSize, next: LayoutSize) -> bool {
    (previous.width - next.width).abs() < 0.5 && (previous.height - next.height).abs() < 0.5
}

fn layout_frame_close(previous: LayoutFrame, next: LayoutFrame) -> bool {
    layout_size_close(
        LayoutSize {
            width: previous.width,
            height: previous.height,
        },
        LayoutSize {
            width: next.width,
            height: next.height,
        },
    ) && (previous.x - next.x).abs() < 0.5
        && (previous.y - next.y).abs() < 0.5
}

// ---------------------------------------------------------------------------
// Node reading helpers
// ---------------------------------------------------------------------------

fn read_layout_size(node: &ArkUINode) -> Option<LayoutSize> {
    let IntSize { width, height } = node.layout_size().ok()?;
    Some(LayoutSize {
        width: width as f32,
        height: height as f32,
    })
}

fn read_layout_frame(node: &ArkUINode) -> Option<LayoutFrame> {
    let size = read_layout_size(node)?;
    let IntOffset { x, y } = node
        .position_with_translate_in_window()
        .or_else(|_| node.layout_position_in_window())
        .ok()?;
    Some(LayoutFrame {
        x: x as f32,
        y: y as f32,
        width: size.width,
        height: size.height,
    })
}

/// Register `onAreaChange` on a mounted node handle.
///
/// `node` is the shared `Rc` that ArkUI uses as the event-dispatch user-data
/// target, so the callback registered here is the one ArkUI's dispatcher finds
/// — `onAreaChange` fires reliably without any polling fallback.
fn register_on_area_change(node: &HostNode, on_change: impl Fn(&HostNode) + 'static) {
    let node_for_cb = node.clone();
    // Borrow the shared node for the duration of event registration. The
    // callback is stored on the node's `event_handle` (inside the `RefCell`),
    // so registration must hold the borrow. ArkUI keeps the node alive via its
    // user-data `Rc`, so the `event_handle` outlives this borrow.
    let mut borrowed = node.borrow_mut();
    EventNode {
        node: &mut borrowed,
    }
    .on_area_change(move |_| {
        on_change(&node_for_cb);
    });
    drop(borrowed);
}

// ---------------------------------------------------------------------------
// Event-registration wrapper (mirrors arkit_arkui's EventNode)
// ---------------------------------------------------------------------------

struct EventNode<'a> {
    node: &'a mut ArkUINode,
}
impl ArkUIAttributeBasic for EventNode<'_> {
    fn raw(&self) -> &ArkUINode {
        self.node
    }
    fn borrow_mut(&mut self) -> &mut ArkUINode {
        self.node
    }
}
impl ArkUIEvent for EventNode<'_> {}

/// Observer hook state: the on_change callback and the last node we registered
/// against (for node-change rebinding and cache cleanup on unmount).
struct LayoutObserverState<F: ?Sized> {
    on_change: Rc<F>,
    registered_key: Rc<std::cell::Cell<Option<usize>>>,
}

impl<F: ?Sized> Clone for LayoutObserverState<F> {
    fn clone(&self) -> Self {
        Self {
            on_change: self.on_change.clone(),
            registered_key: self.registered_key.clone(),
        }
    }
}

/// Observe the [`LayoutSize`] of the current element.
///
/// Registers ArkUI `onAreaChange` on the backing node (resolved via
/// [`use_ark_node`]) and invokes `on_change` whenever the measured size
/// changes meaningfully (>= 0.5 px). The dedup cache is cleaned up on unmount.
#[track_caller]
pub fn use_layout_size(on_change: impl Fn(LayoutSize) + 'static) {
    let node_ref = use_ark_node();
    let signal = node_ref.signal();

    let state = use_hook(|| LayoutObserverState::<dyn Fn(LayoutSize)> {
        on_change: Rc::new(on_change) as Rc<dyn Fn(LayoutSize)>,
        registered_key: Rc::new(std::cell::Cell::new(None)),
    });

    let effect_state = state.clone();
    use_effect(move || {
        let Some(node) = (signal)() else {
            return;
        };
        let key = layout_observer_key(unsafe { &*node.as_ptr() });
        if effect_state.registered_key.get() == Some(key) {
            return;
        }
        if let Some(previous_key) = effect_state.registered_key.replace(Some(key)) {
            clear_layout_observer_cache(previous_key);
        }

        // Emit the current size immediately.
        {
            let n = node.borrow();
            if let Some(size) = read_layout_size(&n) {
                emit_layout_size_if_changed(&n, size, &*effect_state.on_change);
            }
        }

        let on_change_cb = effect_state.on_change.clone();
        register_on_area_change(&node, move |node| {
            let n = node.borrow();
            if let Some(size) = read_layout_size(&n) {
                emit_layout_size_if_changed(&n, size, &*on_change_cb);
            }
        });
    });

    let cleanup_key = state.registered_key.clone();
    use_drop(move || {
        if let Some(key) = cleanup_key.get() {
            clear_layout_observer_cache(key);
        }
    });
}

/// Observe the [`LayoutFrame`] (window-relative position + size) of the current
/// element.
///
/// Same registration model as [`use_layout_size`], driven by `onAreaChange`.
#[track_caller]
pub fn use_layout_frame(on_change: impl Fn(LayoutFrame) + 'static) {
    let node_ref = use_ark_node();
    let signal = node_ref.signal();

    let state = use_hook(|| LayoutObserverState::<dyn Fn(LayoutFrame)> {
        on_change: Rc::new(on_change) as Rc<dyn Fn(LayoutFrame)>,
        registered_key: Rc::new(std::cell::Cell::new(None)),
    });

    let effect_state = state.clone();
    use_effect(move || {
        let Some(node) = (signal)() else {
            return;
        };
        let key = layout_observer_key(unsafe { &*node.as_ptr() });
        if effect_state.registered_key.get() == Some(key) {
            return;
        }
        if let Some(previous_key) = effect_state.registered_key.replace(Some(key)) {
            clear_layout_observer_cache(previous_key);
        }

        {
            let n = node.borrow();
            if let Some(frame) = read_layout_frame(&n) {
                emit_layout_frame_if_changed(&n, frame, &*effect_state.on_change);
            }
        }

        let on_change_cb = effect_state.on_change.clone();
        register_on_area_change(&node, move |node| {
            let n = node.borrow();
            if let Some(frame) = read_layout_frame(&n) {
                emit_layout_frame_if_changed(&n, frame, &*on_change_cb);
            }
        });
    });

    let cleanup_key = state.registered_key.clone();
    use_drop(move || {
        if let Some(key) = cleanup_key.get() {
            clear_layout_observer_cache(key);
        }
    });
}

/// Observe the [`LayoutFrame`] of the current element, passing the backing
/// [`ArkUINode`] to the callback.
///
/// Use this (instead of [`use_layout_frame`] + a separate [`use_ark_node`])
/// when the callback needs to imperatively mutate the host node — e.g. to
/// attach native children sized to the measured frame. `use_ark_node` is
/// idempotent per scope, so mixing this with a direct `use_ark_node()` call is
/// safe, but this variant avoids the extra resolver round-trip and guarantees
/// the node and frame come from the same measurement.
#[track_caller]
pub fn use_layout_frame_node(on_change: impl Fn(ArkUINode, LayoutFrame) + 'static) {
    let node_ref = use_ark_node();
    let signal = node_ref.signal();

    let state = use_hook(|| LayoutObserverState::<dyn Fn(ArkUINode, LayoutFrame)> {
        on_change: Rc::new(on_change) as Rc<dyn Fn(ArkUINode, LayoutFrame)>,
        registered_key: Rc::new(std::cell::Cell::new(None)),
    });

    let effect_state = state.clone();
    use_effect(move || {
        let Some(node) = (signal)() else {
            return;
        };
        let key = layout_observer_key(unsafe { &*node.as_ptr() });
        if effect_state.registered_key.get() == Some(key) {
            return;
        }
        if let Some(previous_key) = effect_state.registered_key.replace(Some(key)) {
            clear_layout_observer_cache(previous_key);
        }

        {
            let n = node.borrow();
            if let Some(frame) = read_layout_frame(&n) {
                emit_layout_frame_node_if_changed(&n, frame, &*effect_state.on_change);
            }
        }

        let on_change_cb = effect_state.on_change.clone();
        register_on_area_change(&node, move |node| {
            let n = node.borrow();
            if let Some(frame) = read_layout_frame(&n) {
                emit_layout_frame_node_if_changed(&n, frame, &*on_change_cb);
            }
        });
    });

    let cleanup_key = state.registered_key.clone();
    use_drop(move || {
        if let Some(key) = cleanup_key.get() {
            clear_layout_observer_cache(key);
        }
    });
}
