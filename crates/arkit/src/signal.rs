use std::any::Any;
use std::cell::{Cell, RefCell};
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
}

struct ComponentLifecycleHook {
    on_unmount: Rc<dyn Fn()>,
}

enum HookSlot {
    Signal(Box<dyn Any>),
    ComponentLifecycle(ComponentLifecycleHook),
    LifecycleObserver(usize),
}

impl HookSlot {
    fn cleanup(self) {
        match self {
            HookSlot::Signal(_) => {}
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
    subscribers: Vec<Rc<dyn Fn()>>,
}

impl<T> Signal<T> {
    pub fn new(value: T) -> Self {
        Self {
            inner: Rc::new(RefCell::new(SignalState {
                value,
                subscribers: Vec::new(),
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

    pub fn subscribe(&self, callback: impl Fn() + 'static) {
        self.inner
            .borrow_mut()
            .subscribers
            .push(Rc::new(callback) as Rc<dyn Fn()>);
    }

    fn notify(&self) {
        let subscribers = self.inner.borrow().subscribers.clone();
        for callback in subscribers {
            callback();
        }
        notify_scheduler();
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
            if let HookSlot::Signal(signal_any) = slot {
                if let Some(signal) = signal_any.downcast_ref::<Signal<T>>() {
                    return signal.clone();
                }
            }

            panic!("use_signal hook type mismatch at slot {current}");
        }

        let signal = Signal::new(init());
        hooks.slots.push(HookSlot::Signal(Box::new(signal.clone())));
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
