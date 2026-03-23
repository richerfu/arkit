use std::any::Any;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use crate::lifecycle::LifecycleEvent;

thread_local! {
    static SCHEDULER: RefCell<Option<Rc<dyn Fn()>>> = RefCell::new(None);
    static HOOKS: RefCell<Option<Rc<RefCell<HookState>>>> = RefCell::new(None);
    static LIFECYCLE_OBSERVERS: RefCell<Vec<(usize, Rc<dyn Fn(LifecycleEvent)>)>> = RefCell::new(Vec::new());
    static LIFECYCLE_NEXT_ID: Cell<usize> = const { Cell::new(1) };
}

pub(crate) fn set_scheduler(scheduler: Option<Rc<dyn Fn()>>) {
    SCHEDULER.with(|state| {
        state.replace(scheduler);
    });
}

fn notify_scheduler() {
    SCHEDULER.with(|state| {
        if let Some(cb) = state.borrow().as_ref() {
            cb();
        }
    });
}

pub(crate) struct HookState {
    cursor: usize,
    slots: Vec<HookSlot>,
    signal_subscriber: Option<Rc<dyn Fn()>>,
}

struct ComponentLifecycleHook {
    on_unmount: Rc<dyn Fn()>,
}

struct SignalHook {
    signal: Box<dyn Any>,
    cleanup: Option<Box<dyn FnOnce()>>,
}

enum HookSlot {
    Signal(SignalHook),
    ComponentLifecycle(ComponentLifecycleHook),
    LifecycleObserver(usize),
}

impl HookSlot {
    fn cleanup(self) {
        match self {
            HookSlot::Signal(mut slot) => {
                if let Some(cleanup) = slot.cleanup.take() {
                    cleanup();
                }
                let _ = slot.signal;
            }
            HookSlot::ComponentLifecycle(slot) => {
                (slot.on_unmount.as_ref())();
            }
            HookSlot::LifecycleObserver(id) => {
                unregister_lifecycle_observer(id);
            }
        }
    }
}

impl HookState {
    pub(crate) fn new() -> Self {
        Self {
            cursor: 0,
            slots: Vec::new(),
            signal_subscriber: None,
        }
    }

    pub(crate) fn with_signal_subscriber(signal_subscriber: Rc<dyn Fn()>) -> Self {
        Self {
            cursor: 0,
            slots: Vec::new(),
            signal_subscriber: Some(signal_subscriber),
        }
    }

    pub(crate) fn reset_cursor(&mut self) {
        self.cursor = 0;
    }

    pub(crate) fn finalize_render(&mut self) {
        while self.slots.len() > self.cursor {
            if let Some(slot) = self.slots.pop() {
                slot.cleanup();
            }
        }
    }

    pub(crate) fn cleanup_all(&mut self) {
        self.cursor = 0;
        while let Some(slot) = self.slots.pop() {
            slot.cleanup();
        }
    }
}

pub(crate) fn with_hook_state<R>(hooks: Rc<RefCell<HookState>>, f: impl FnOnce() -> R) -> R {
    HOOKS.with(|state| {
        let previous = state.replace(Some(hooks));
        let result = f();
        state.replace(previous);
        result
    })
}

pub struct Signal<T> {
    inner: Rc<RefCell<SignalState<T>>>,
}

impl<T> Clone for Signal<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

struct SignalState<T> {
    value: T,
    subscribers: HashMap<usize, Rc<dyn Fn()>>,
    next_subscriber_id: usize,
    notify_scheduler: bool,
}

impl<T> Signal<T> {
    pub fn new(value: T) -> Self {
        Self::new_with_scheduler(value, true)
    }

    pub(crate) fn new_with_scheduler(value: T, notify_scheduler: bool) -> Self {
        Self {
            inner: Rc::new(RefCell::new(SignalState {
                value,
                subscribers: HashMap::new(),
                next_subscriber_id: 1,
                notify_scheduler,
            })),
        }
    }

    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        let state = self.inner.borrow();
        f(&state.value)
    }

    pub fn set(&self, value: T) {
        {
            let mut state = self.inner.borrow_mut();
            state.value = value;
        }
        self.notify();
    }

    pub fn update(&self, f: impl FnOnce(&mut T)) {
        {
            let mut state = self.inner.borrow_mut();
            f(&mut state.value);
        }
        self.notify();
    }

    pub fn subscribe(&self, callback: impl Fn() + 'static) -> usize {
        let mut state = self.inner.borrow_mut();
        let id = state.next_subscriber_id;
        state.next_subscriber_id += 1;
        state
            .subscribers
            .insert(id, Rc::new(callback) as Rc<dyn Fn()>);
        id
    }

    pub fn unsubscribe(&self, id: usize) -> bool {
        self.inner.borrow_mut().subscribers.remove(&id).is_some()
    }

    fn notify(&self) {
        let (subscribers, notify_global_scheduler) = {
            let state = self.inner.borrow();
            (
                state.subscribers.values().cloned().collect::<Vec<_>>(),
                state.notify_scheduler,
            )
        };
        for callback in subscribers {
            callback();
        }
        if notify_global_scheduler {
            notify_scheduler();
        }
    }
}

impl<T: Clone> Signal<T> {
    pub fn get(&self) -> T {
        self.inner.borrow().value.clone()
    }
}

pub fn signal<T>(value: T) -> Signal<T> {
    Signal::new(value)
}

pub fn use_signal<T: 'static>(init: impl FnOnce() -> T) -> Signal<T> {
    HOOKS.with(|hooks_state| {
        let hooks = hooks_state
            .borrow()
            .as_ref()
            .cloned()
            .expect("use_signal must be called during arkit render");

        let mut hooks = hooks.borrow_mut();
        let current = hooks.cursor;
        hooks.cursor += 1;

        if let Some(slot) = hooks.slots.get(current) {
            if let HookSlot::Signal(signal_hook) = slot {
                if let Some(signal) = signal_hook.signal.downcast_ref::<Signal<T>>() {
                    return signal.clone();
                }
            }

            panic!("use_signal hook type mismatch at slot {current}");
        }

        let signal = Signal::new_with_scheduler(init(), hooks.signal_subscriber.is_none());
        let cleanup = hooks.signal_subscriber.as_ref().map(|subscriber| {
            let signal_for_cleanup = signal.clone();
            let subscriber = subscriber.clone();
            let subscription_id = signal.subscribe(move || subscriber());
            Box::new(move || {
                signal_for_cleanup.unsubscribe(subscription_id);
            }) as Box<dyn FnOnce()>
        });
        hooks.slots.push(HookSlot::Signal(SignalHook {
            signal: Box::new(signal.clone()),
            cleanup,
        }));
        signal
    })
}

pub fn use_component_lifecycle(on_mount: impl FnOnce() + 'static, on_unmount: impl Fn() + 'static) {
    HOOKS.with(|hooks_state| {
        let hooks = hooks_state
            .borrow()
            .as_ref()
            .cloned()
            .expect("use_component_lifecycle must be called during arkit render");

        let mut hooks = hooks.borrow_mut();
        let current = hooks.cursor;
        hooks.cursor += 1;

        if let Some(slot) = hooks.slots.get(current) {
            match slot {
                HookSlot::ComponentLifecycle(_) => return,
                _ => panic!("use_component_lifecycle hook type mismatch at slot {current}"),
            }
        }

        on_mount();
        hooks
            .slots
            .push(HookSlot::ComponentLifecycle(ComponentLifecycleHook {
                on_unmount: Rc::new(on_unmount),
            }));
    });
}

pub fn use_lifecycle(callback: impl Fn(LifecycleEvent) + 'static) {
    HOOKS.with(|hooks_state| {
        let hooks = hooks_state
            .borrow()
            .as_ref()
            .cloned()
            .expect("use_lifecycle must be called during arkit render");

        let mut hooks = hooks.borrow_mut();
        let current = hooks.cursor;
        hooks.cursor += 1;

        if let Some(slot) = hooks.slots.get(current) {
            match slot {
                HookSlot::LifecycleObserver(_) => return,
                _ => panic!("use_lifecycle hook type mismatch at slot {current}"),
            }
        }

        let observer: Rc<dyn Fn(LifecycleEvent)> = Rc::new(callback);
        let observer_id = register_lifecycle_observer(observer);
        hooks.slots.push(HookSlot::LifecycleObserver(observer_id));
    });
}

pub(crate) fn emit_lifecycle_event(event: LifecycleEvent) {
    let callbacks = LIFECYCLE_OBSERVERS.with(|state| {
        state
            .borrow()
            .iter()
            .map(|(_, callback)| callback.clone())
            .collect::<Vec<_>>()
    });

    for callback in callbacks {
        callback(event.clone());
    }
}

fn register_lifecycle_observer(callback: Rc<dyn Fn(LifecycleEvent)>) -> usize {
    let id = LIFECYCLE_NEXT_ID.with(|next| {
        let id = next.get();
        next.set(id + 1);
        id
    });

    LIFECYCLE_OBSERVERS.with(|state| {
        state.borrow_mut().push((id, callback));
    });

    id
}

fn unregister_lifecycle_observer(id: usize) {
    LIFECYCLE_OBSERVERS.with(|state| {
        let mut state = state.borrow_mut();
        state.retain(|(observer_id, _)| *observer_id != id);
    });
}
