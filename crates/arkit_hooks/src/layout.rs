//! Tokenized, multi-subscriber ArkUI layout observation.

use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

use arkit_prelude::{use_drop, use_effect, use_hook};
use ohos_arkui_binding::api::node_custom_event::{IntOffset, IntSize};
use ohos_arkui_binding::common::node::ArkUINode;
use ohos_arkui_binding::component::attribute::{ArkUIAttributeBasic, ArkUIEvent};
use rustc_hash::FxHashMap;
use smallvec::SmallVec;

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

/// A measured frame in physical pixels, relative to the window.
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

fn layout_observer_key(node: &ArkUINode) -> usize {
    node.raw_handle() as usize
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

fn read_layout_size(node: &ArkUINode) -> Option<LayoutSize> {
    let IntSize { width, height } = node.layout_size().ok()?;
    Some(LayoutSize {
        width: width as f32,
        height: height as f32,
    })
}

fn read_layout_frame(node: &ArkUINode, size: LayoutSize) -> Option<LayoutFrame> {
    // Prefer layout position (no graphic translate). `position_with_translate`
    // accumulates ancestor translate/matrix offsets that do not match the painted
    // layout box — that shifted Select/Popover anchors right on device while
    // width/height stayed correct.
    let IntOffset { x, y } = node
        .layout_position_in_window()
        .or_else(|_| node.position_with_translate_in_window())
        .ok()?;
    Some(LayoutFrame {
        x: x as f32,
        y: y as f32,
        width: size.width,
        height: size.height,
    })
}

enum LayoutObserver {
    Size {
        callback: Rc<dyn Fn(LayoutSize)>,
        last: Option<LayoutSize>,
    },
    Frame {
        callback: Rc<dyn Fn(LayoutFrame)>,
        last: Option<LayoutFrame>,
    },
    FrameNode {
        callback: Rc<dyn Fn(ArkUINode, LayoutFrame)>,
        last: Option<LayoutFrame>,
    },
}

enum LayoutNotification {
    Size(Rc<dyn Fn(LayoutSize)>, LayoutSize),
    Frame(Rc<dyn Fn(LayoutFrame)>, LayoutFrame),
    FrameNode(Rc<dyn Fn(ArkUINode, LayoutFrame)>, ArkUINode, LayoutFrame),
}

struct NodeObservers {
    generation: u64,
    node: Weak<RefCell<ArkUINode>>,
    observers: FxHashMap<u64, LayoutObserver>,
}

#[derive(Default)]
struct LayoutHub {
    nodes: FxHashMap<usize, NodeObservers>,
    next_generation: u64,
    next_subscription: u64,
}

thread_local! {
    static LAYOUT_HUB: RefCell<LayoutHub> = RefCell::new(LayoutHub::default());
}

struct LayoutSubscription {
    node_key: usize,
    generation: u64,
    id: u64,
}

impl Drop for LayoutSubscription {
    fn drop(&mut self) {
        LAYOUT_HUB.with(|hub| {
            let mut hub = hub.borrow_mut();
            let Some(node) = hub.nodes.get_mut(&self.node_key) else {
                return;
            };
            if node.generation == self.generation {
                node.observers.remove(&self.id);
            }
        });
    }
}

fn subscribe_layout(node: &HostNode, observer: LayoutObserver) -> LayoutSubscription {
    let node_key = layout_observer_key(&node.borrow());
    let (generation, id, install_listener) = LAYOUT_HUB.with(|hub| {
        let mut hub = hub.borrow_mut();
        // Native handles can be recycled. Remove entries whose mounted
        // wrapper is gone before comparing raw keys or allocating a new
        // generation.
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
                .expect("arkit_hooks: layout observer generation space exhausted");
            let generation = hub.next_generation;
            hub.nodes.insert(
                node_key,
                NodeObservers {
                    generation,
                    node: Rc::downgrade(node),
                    observers: FxHashMap::default(),
                },
            );
        }
        hub.next_subscription = hub
            .next_subscription
            .checked_add(1)
            .expect("arkit_hooks: layout subscription id space exhausted");
        let id = hub.next_subscription;
        let entry = hub
            .nodes
            .get_mut(&node_key)
            .expect("layout hub entry must exist after insertion");
        entry.observers.insert(id, observer);
        (entry.generation, id, install_listener)
    });

    if install_listener {
        register_on_area_change(node, move |node| {
            dispatch_layout(node_key, generation, node);
        });
    }
    dispatch_layout(node_key, generation, node);

    LayoutSubscription {
        node_key,
        generation,
        id,
    }
}

fn dispatch_layout(node_key: usize, generation: u64, node: &HostNode) {
    let Some((needs_frame, needs_node_value)) = LAYOUT_HUB.with(|hub| {
        let hub = hub.borrow();
        let entry = hub.nodes.get(&node_key)?;
        if entry.generation != generation {
            return None;
        }
        Some((
            entry.observers.values().any(|observer| {
                matches!(
                    observer,
                    LayoutObserver::Frame { .. } | LayoutObserver::FrameNode { .. }
                )
            }),
            entry
                .observers
                .values()
                .any(|observer| matches!(observer, LayoutObserver::FrameNode { .. })),
        ))
    }) else {
        return;
    };
    let (size, frame, node_value) = {
        let node = node.borrow();
        let size = read_layout_size(&node);
        let frame = needs_frame
            .then(|| size.and_then(|size| read_layout_frame(&node, size)))
            .flatten();
        let node_value = needs_node_value.then(|| node.clone());
        (size, frame, node_value)
    };
    let notifications = LAYOUT_HUB.with(|hub| {
        let mut hub = hub.borrow_mut();
        let Some(entry) = hub.nodes.get_mut(&node_key) else {
            return SmallVec::new();
        };
        if entry.generation != generation {
            return SmallVec::new();
        }
        // Most nodes have one or two layout consumers. Keep that dispatch
        // entirely on the stack while preserving reentrant callback safety.
        let mut notifications = SmallVec::<[LayoutNotification; 4]>::new();
        for observer in entry.observers.values_mut() {
            match observer {
                LayoutObserver::Size { callback, last } => {
                    if let Some(next) = size {
                        if !last.is_some_and(|previous| layout_size_close(previous, next)) {
                            *last = Some(next);
                            notifications.push(LayoutNotification::Size(callback.clone(), next));
                        }
                    }
                }
                LayoutObserver::Frame { callback, last } => {
                    if let Some(next) = frame {
                        if !last.is_some_and(|previous| layout_frame_close(previous, next)) {
                            *last = Some(next);
                            notifications.push(LayoutNotification::Frame(callback.clone(), next));
                        }
                    }
                }
                LayoutObserver::FrameNode { callback, last } => {
                    if let Some(next) = frame {
                        if !last.is_some_and(|previous| layout_frame_close(previous, next)) {
                            *last = Some(next);
                            if let Some(node_value) = node_value.as_ref() {
                                notifications.push(LayoutNotification::FrameNode(
                                    callback.clone(),
                                    node_value.clone(),
                                    next,
                                ));
                            }
                        }
                    }
                }
            }
        }
        notifications
    });

    // Application callbacks are invoked after the hub borrow is released so
    // callbacks may mount/unmount other observers without RefCell reentrancy.
    for notification in notifications {
        match notification {
            LayoutNotification::Size(callback, value) => callback(value),
            LayoutNotification::Frame(callback, value) => callback(value),
            LayoutNotification::FrameNode(callback, node, value) => callback(node, value),
        }
    }
}

fn register_on_area_change(node: &HostNode, on_change: impl Fn(&HostNode) + 'static) {
    let weak_node = Rc::downgrade(node);
    let mut borrowed = node.borrow_mut();
    EventNode {
        node: &mut borrowed,
    }
    .on_area_change(move |_| {
        if let Some(node) = weak_node.upgrade() {
            on_change(&node);
        }
    });
}

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

struct LayoutHookState<C: ?Sized> {
    callback: Rc<RefCell<Rc<C>>>,
    subscription: Rc<RefCell<Option<LayoutSubscription>>>,
    node_key: Rc<Cell<Option<usize>>>,
}

impl<C: ?Sized> Clone for LayoutHookState<C> {
    fn clone(&self) -> Self {
        Self {
            callback: self.callback.clone(),
            subscription: self.subscription.clone(),
            node_key: self.node_key.clone(),
        }
    }
}

#[track_caller]
pub fn use_layout_size(on_change: impl Fn(LayoutSize) + 'static) {
    let node_ref = use_ark_node();
    let signal = node_ref.signal();
    let next = Rc::new(on_change) as Rc<dyn Fn(LayoutSize)>;
    let initial = next.clone();
    let state = use_hook(move || LayoutHookState::<dyn Fn(LayoutSize)> {
        callback: Rc::new(RefCell::new(initial)),
        subscription: Rc::new(RefCell::new(None)),
        node_key: Rc::new(Cell::new(None)),
    });
    *state.callback.borrow_mut() = next;

    let effect_state = state.clone();
    use_effect(move || {
        let Some(node) = signal() else {
            effect_state.subscription.borrow_mut().take();
            effect_state.node_key.set(None);
            return;
        };
        let key = layout_observer_key(&node.borrow());
        if effect_state.node_key.get() == Some(key) {
            return;
        }
        let callback = effect_state.callback.clone();
        let observer = LayoutObserver::Size {
            callback: Rc::new(move |value| callback.borrow().clone()(value)),
            last: None,
        };
        let subscription = subscribe_layout(&node, observer);
        effect_state.subscription.replace(Some(subscription));
        effect_state.node_key.set(Some(key));
    });

    let cleanup = state.subscription.clone();
    use_drop(move || {
        cleanup.borrow_mut().take();
    });
}

#[track_caller]
pub fn use_layout_frame(on_change: impl Fn(LayoutFrame) + 'static) {
    let node_ref = use_ark_node();
    let signal = node_ref.signal();
    let next = Rc::new(on_change) as Rc<dyn Fn(LayoutFrame)>;
    let initial = next.clone();
    let state = use_hook(move || LayoutHookState::<dyn Fn(LayoutFrame)> {
        callback: Rc::new(RefCell::new(initial)),
        subscription: Rc::new(RefCell::new(None)),
        node_key: Rc::new(Cell::new(None)),
    });
    *state.callback.borrow_mut() = next;

    let effect_state = state.clone();
    use_effect(move || {
        let Some(node) = signal() else {
            effect_state.subscription.borrow_mut().take();
            effect_state.node_key.set(None);
            return;
        };
        let key = layout_observer_key(&node.borrow());
        if effect_state.node_key.get() == Some(key) {
            return;
        }
        let callback = effect_state.callback.clone();
        let observer = LayoutObserver::Frame {
            callback: Rc::new(move |value| callback.borrow().clone()(value)),
            last: None,
        };
        let subscription = subscribe_layout(&node, observer);
        effect_state.subscription.replace(Some(subscription));
        effect_state.node_key.set(Some(key));
    });

    let cleanup = state.subscription.clone();
    use_drop(move || {
        cleanup.borrow_mut().take();
    });
}

#[track_caller]
pub fn use_layout_frame_node(on_change: impl Fn(ArkUINode, LayoutFrame) + 'static) {
    let node_ref = use_ark_node();
    let signal = node_ref.signal();
    let next = Rc::new(on_change) as Rc<dyn Fn(ArkUINode, LayoutFrame)>;
    let initial = next.clone();
    let state = use_hook(move || LayoutHookState::<dyn Fn(ArkUINode, LayoutFrame)> {
        callback: Rc::new(RefCell::new(initial)),
        subscription: Rc::new(RefCell::new(None)),
        node_key: Rc::new(Cell::new(None)),
    });
    *state.callback.borrow_mut() = next;

    let effect_state = state.clone();
    use_effect(move || {
        let Some(node) = signal() else {
            effect_state.subscription.borrow_mut().take();
            effect_state.node_key.set(None);
            return;
        };
        let key = layout_observer_key(&node.borrow());
        if effect_state.node_key.get() == Some(key) {
            return;
        }
        let callback = effect_state.callback.clone();
        let observer = LayoutObserver::FrameNode {
            callback: Rc::new(move |node, value| callback.borrow().clone()(node, value)),
            last: None,
        };
        let subscription = subscribe_layout(&node, observer);
        effect_state.subscription.replace(Some(subscription));
        effect_state.node_key.set(Some(key));
    });

    let cleanup = state.subscription.clone();
    use_drop(move || {
        cleanup.borrow_mut().take();
    });
}
