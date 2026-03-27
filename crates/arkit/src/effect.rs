use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::owner::{current_owner, with_owner, Owner};

thread_local! {
    /// The currently executing reactive computation (effect or memo).
    /// When active, signal reads register themselves as dependencies.
    static LISTENER: RefCell<Option<Rc<Computation>>> = RefCell::new(None);

    /// Queue of computations to re-run in the current batch.
    static PENDING_EFFECTS: RefCell<Vec<Rc<Computation>>> = RefCell::new(Vec::new());

    /// Batch depth counter - when > 0, effects are queued instead of run immediately.
    static BATCH_DEPTH: Cell<usize> = const { Cell::new(0) };
}

/// A reactive computation (effect or memo).
pub(crate) struct Computation {
    /// The user's effect function, wrapped to re-execute with tracking.
    execute: RefCell<Option<Box<dyn FnMut()>>>,
    /// Sources this computation depends on (signals call `mark_dirty` on us).
    sources: RefCell<Vec<SubscriptionHandle>>,
    /// The owner this computation was created under.
    owner: Rc<Owner>,
    /// Whether this computation needs to re-run.
    dirty: Cell<bool>,
    /// Whether this computation has been disposed.
    disposed: Cell<bool>,
}

/// A handle to unsubscribe from a signal source.
struct SubscriptionHandle {
    unsubscribe: Option<Box<dyn FnOnce()>>,
}

impl Computation {
    fn new(owner: Rc<Owner>, execute: impl FnMut() + 'static) -> Rc<Self> {
        Rc::new(Self {
            execute: RefCell::new(Some(Box::new(execute))),
            sources: RefCell::new(Vec::new()),
            owner,
            dirty: Cell::new(false),
            disposed: Cell::new(false),
        })
    }

    /// Re-run this computation with dependency tracking.
    fn run(self: &Rc<Self>) {
        if self.disposed.get() {
            return;
        }
        // Clear old subscriptions
        self.clear_sources();
        // Set as current listener for auto-tracking
        let previous = set_listener(Some(self.clone()));
        // Execute inside own owner
        if let Some(f) = self.execute.borrow_mut().as_mut() {
            with_owner(self.owner.clone(), || f());
        }
        // Restore previous listener
        set_listener(previous);
        self.dirty.set(false);
    }

    /// Mark this computation as dirty and schedule it.
    pub(crate) fn mark_dirty(self: &Rc<Self>) {
        if self.disposed.get() || self.dirty.get() {
            return;
        }
        self.dirty.set(true);

        if is_batching() {
            PENDING_EFFECTS.with(|q| q.borrow_mut().push(self.clone()));
        } else {
            self.run();
        }
    }

    /// Record a subscription to a signal source.
    pub(crate) fn add_source(&self, unsubscribe: impl FnOnce() + 'static) {
        self.sources.borrow_mut().push(SubscriptionHandle {
            unsubscribe: Some(Box::new(unsubscribe)),
        });
    }

    /// Drop all source subscriptions.
    fn clear_sources(&self) {
        let sources = std::mem::take(&mut *self.sources.borrow_mut());
        for mut handle in sources {
            if let Some(unsub) = handle.unsubscribe.take() {
                unsub();
            }
        }
    }

    /// Dispose this computation permanently.
    pub(crate) fn dispose(&self) {
        if self.disposed.replace(true) {
            return;
        }
        self.clear_sources();
        // Use try_borrow_mut because dispose() can be called while this
        // computation is currently executing (run() holds a borrow_mut on
        // execute). In that case the disposed flag is already set, so the
        // computation won't run again, and the execute fn will be dropped
        // when the Rc is released.
        if let Ok(mut exec) = self.execute.try_borrow_mut() {
            exec.take();
        }
    }
}

// ── Public API ──────────────────────────────────────────────────────────────

/// Create a reactive side-effect. The function `f` runs immediately and re-runs
/// whenever any signal it reads changes. Dependencies are tracked automatically.
///
/// Aligned with SolidJS `createEffect`.
pub fn create_effect(mut f: impl FnMut() + 'static) {
    let owner = current_owner().expect("create_effect called outside of reactive scope");
    let comp = Computation::new(owner.clone(), move || f());

    // Register cleanup to dispose the computation when the owner is disposed
    let comp_cleanup = comp.clone();
    owner.on_cleanup(move || comp_cleanup.dispose());

    // Initial execution with tracking
    comp.run();
}

/// Create a reactive side-effect that tracks a specific source signal.
///
/// Aligned with SolidJS `createEffect` with explicit `on()` source.
pub fn create_effect_on<S: Clone + 'static>(
    source: &crate::signal::Signal<S>,
    effect: impl Fn(&S) + 'static,
) {
    let source = source.clone();
    create_effect(move || {
        let value = source.get();
        effect(&value);
    });
}

/// Groups multiple signal updates into a single batch. Effects are deferred
/// until the outermost batch completes.
///
/// Aligned with SolidJS `batch()`.
pub fn batch<R>(f: impl FnOnce() -> R) -> R {
    BATCH_DEPTH.with(|d| d.set(d.get() + 1));
    let result = f();
    let depth = BATCH_DEPTH.with(|d| {
        let next = d.get() - 1;
        d.set(next);
        next
    });
    if depth == 0 {
        flush_pending();
    }
    result
}

/// Prevent tracking inside the given closure. Reads inside `untrack(|| ...)`
/// will not register as dependencies of the current computation.
///
/// Aligned with SolidJS `untrack()`.
pub fn untrack<R>(f: impl FnOnce() -> R) -> R {
    let previous = set_listener(None);
    let result = f();
    set_listener(previous);
    result
}

// ── Internal helpers ────────────────────────────────────────────────────────

/// Get the current listener (active computation for dependency tracking).
pub(crate) fn current_listener() -> Option<Rc<Computation>> {
    LISTENER.with(|l| l.borrow().clone())
}

/// Set the current listener, returning the previous one.
fn set_listener(listener: Option<Rc<Computation>>) -> Option<Rc<Computation>> {
    LISTENER.with(|l| l.replace(listener))
}

fn is_batching() -> bool {
    BATCH_DEPTH.with(|d| d.get() > 0)
}

/// Flush all pending effects after a batch completes.
fn flush_pending() {
    loop {
        let effects = PENDING_EFFECTS.with(|q| std::mem::take(&mut *q.borrow_mut()));
        if effects.is_empty() {
            break;
        }
        for effect in effects {
            if effect.dirty.get() && !effect.disposed.get() {
                effect.run();
            }
        }
    }
}
