use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::effect::current_listener;
use crate::owner::current_owner;

// ── Signal ──────────────────────────────────────────────────────────────────

/// A reactive value. Reading inside a reactive computation (effect/memo)
/// automatically tracks the dependency. Writing notifies all subscribers.
///
/// This is a unified handle (SolidJS provides split read/write; we keep
/// the unified `Signal<T>` for ergonomic Rust API and add `ReadSignal<T>`
/// as a read-only view).
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
}

impl<T: 'static> Signal<T> {
    pub fn new(value: T) -> Self {
        Self {
            inner: Rc::new(RefCell::new(SignalState {
                value,
                subscribers: HashMap::new(),
                next_subscriber_id: 1,
            })),
        }
    }

    /// Borrow the value without cloning. **Does** track dependencies.
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        self.track();
        let state = self.inner.borrow();
        f(&state.value)
    }

    /// Borrow the value without cloning and **without** tracking.
    pub fn with_untracked<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        let state = self.inner.borrow();
        f(&state.value)
    }

    /// Set the signal value and notify all subscribers.
    pub fn set(&self, value: T) {
        {
            let mut state = self.inner.borrow_mut();
            state.value = value;
        }
        self.notify();
    }

    /// Mutate the value in-place and notify.
    pub fn update(&self, f: impl FnOnce(&mut T)) {
        {
            let mut state = self.inner.borrow_mut();
            f(&mut state.value);
        }
        self.notify();
    }

    /// Subscribe to changes. Returns a subscription ID for later removal.
    /// Note: For most cases, `create_effect` with automatic tracking is preferred.
    pub fn subscribe(&self, callback: impl Fn() + 'static) -> usize {
        let mut state = self.inner.borrow_mut();
        let id = state.next_subscriber_id;
        state.next_subscriber_id += 1;
        state
            .subscribers
            .insert(id, Rc::new(callback) as Rc<dyn Fn()>);
        id
    }

    /// Remove a subscription.
    pub fn unsubscribe(&self, id: usize) -> bool {
        self.inner.borrow_mut().subscribers.remove(&id).is_some()
    }

    /// Register this signal as a dependency of the current computation.
    fn track(&self) {
        if let Some(listener) = current_listener() {
            let signal = self.clone();
            let listener_weak = Rc::downgrade(&listener);
            let sub_id = self.subscribe(move || {
                if let Some(comp) = listener_weak.upgrade() {
                    comp.mark_dirty();
                }
            });
            listener.add_source(move || {
                signal.unsubscribe(sub_id);
            });
        }
    }

    fn notify(&self) {
        let subscribers = {
            let state = self.inner.borrow();
            state.subscribers.values().cloned().collect::<Vec<_>>()
        };
        for callback in subscribers {
            callback();
        }
    }
}

impl<T: Clone + 'static> Signal<T> {
    /// Get the signal value by cloning. **Does** track dependencies.
    pub fn get(&self) -> T {
        self.track();
        self.inner.borrow().value.clone()
    }

    /// Get the signal value by cloning **without** tracking.
    pub fn get_untracked(&self) -> T {
        self.inner.borrow().value.clone()
    }
}

// ── ReadSignal ──────────────────────────────────────────────────────────────

/// A read-only view of a signal. Cannot be written to directly.
pub struct ReadSignal<T> {
    inner: Signal<T>,
}

impl<T> Clone for ReadSignal<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T: 'static> ReadSignal<T> {
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        self.inner.with(f)
    }

    pub fn with_untracked<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        self.inner.with_untracked(f)
    }
}

impl<T: Clone + 'static> ReadSignal<T> {
    pub fn get(&self) -> T {
        self.inner.get()
    }

    pub fn get_untracked(&self) -> T {
        self.inner.get_untracked()
    }
}

impl<T> From<Signal<T>> for ReadSignal<T> {
    fn from(signal: Signal<T>) -> Self {
        Self { inner: signal }
    }
}

// ── Constructors ────────────────────────────────────────────────────────────

/// Create a new signal. Shorthand for `Signal::new(value)`.
pub fn signal<T: 'static>(value: T) -> Signal<T> {
    Signal::new(value)
}

/// Create a signal and register it on the current owner.
/// When the owner is disposed, the signal's subscribers are cleared.
pub fn create_signal<T: 'static>(value: T) -> Signal<T> {
    let signal = Signal::new(value);
    if let Some(owner) = current_owner() {
        let cleanup_signal = signal.clone();
        owner.on_cleanup(move || {
            // Clear all subscribers on disposal
            let mut state = cleanup_signal.inner.borrow_mut();
            state.subscribers.clear();
        });
    }
    signal
}

// ── Memo ────────────────────────────────────────────────────────────────────

/// Create a derived reactive computation. Re-computes when dependencies change
/// and only notifies downstream if the value actually changed.
///
/// Aligned with SolidJS `createMemo`.
pub fn create_memo<T>(compute: impl Fn() -> T + 'static) -> ReadSignal<T>
where
    T: Clone + PartialEq + 'static,
{
    // Use untrack for initial computation to avoid registering in parent effect
    let initial = crate::effect::untrack(|| compute());
    let memo_signal = create_signal(initial);

    let writer = memo_signal.clone();
    crate::effect::create_effect(move || {
        let next = compute();
        let changed = writer.with_untracked(|current| *current != next);
        if changed {
            writer.set(next);
        }
    });

    memo_signal.into()
}

/// Backward-compatible overload: create a memo from an explicit source signal.
pub fn create_memo_on<S, T>(source: &Signal<S>, compute: impl Fn(&S) -> T + 'static) -> ReadSignal<T>
where
    S: Clone + 'static,
    T: Clone + PartialEq + 'static,
{
    let source = source.clone();
    create_memo(move || source.with(|v| compute(v)))
}


