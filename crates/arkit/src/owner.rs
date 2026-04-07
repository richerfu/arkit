use std::any::{Any, TypeId};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::{Rc, Weak};

thread_local! {
    static OWNER: RefCell<Option<Rc<Owner>>> = RefCell::new(None);
}

/// A node in the reactive ownership tree.
///
/// Each component/scope creates an `Owner` that tracks:
/// - cleanup functions (run on disposal)
/// - child owners (disposed recursively)
/// - context values (looked up by walking up the tree)
/// - reactive computations owned by this scope
pub struct Owner {
    parent: Option<Weak<Owner>>,
    cleanups: RefCell<Vec<Box<dyn FnOnce()>>>,
    children: RefCell<Vec<Rc<Owner>>>,
    contexts: RefCell<HashMap<TypeId, Box<dyn Any>>>,
    disposed: Cell<bool>,
}

impl Owner {
    /// Create a new root owner with no parent.
    pub(crate) fn new_root() -> Rc<Self> {
        Rc::new(Self {
            parent: None,
            cleanups: RefCell::new(Vec::new()),
            children: RefCell::new(Vec::new()),
            contexts: RefCell::new(HashMap::new()),
            disposed: Cell::new(false),
        })
    }

    /// Create a child owner under the given parent.
    pub(crate) fn new_child(parent: &Rc<Owner>) -> Rc<Self> {
        let child = Rc::new(Self {
            parent: Some(Rc::downgrade(parent)),
            cleanups: RefCell::new(Vec::new()),
            children: RefCell::new(Vec::new()),
            contexts: RefCell::new(HashMap::new()),
            disposed: Cell::new(false),
        });
        parent.children.borrow_mut().push(child.clone());
        child
    }

    /// Register a cleanup function on this owner.
    pub(crate) fn on_cleanup(&self, cleanup: impl FnOnce() + 'static) {
        if self.disposed.get() {
            cleanup();
            return;
        }
        self.cleanups.borrow_mut().push(Box::new(cleanup));
    }

    /// Provide a context value of type `T` on this owner.
    pub(crate) fn provide_context<T: 'static>(&self, value: T) {
        self.contexts
            .borrow_mut()
            .insert(TypeId::of::<T>(), Box::new(value));
    }

    /// Look up a context value of type `T` by walking up the owner tree.
    pub(crate) fn use_context<T: Clone + 'static>(&self) -> Option<T> {
        let type_id = TypeId::of::<T>();
        if let Some(value) = self.contexts.borrow().get(&type_id) {
            return value.downcast_ref::<T>().cloned();
        }
        if let Some(parent) = self.parent.as_ref().and_then(Weak::upgrade) {
            return parent.use_context::<T>();
        }
        None
    }

    /// Dispose this owner, running all cleanups and recursively disposing children.
    pub(crate) fn dispose(&self) {
        if self.disposed.replace(true) {
            return;
        }
        // Dispose children first (bottom-up)
        let children = std::mem::take(&mut *self.children.borrow_mut());
        for child in children {
            child.dispose();
        }
        // Run own cleanups in reverse order
        let cleanups = std::mem::take(&mut *self.cleanups.borrow_mut());
        for cleanup in cleanups.into_iter().rev() {
            cleanup();
        }
    }

    /// Remove a specific child from the children list (used when a child is disposed independently).
    pub(crate) fn remove_child(&self, child: &Rc<Owner>) {
        self.children.borrow_mut().retain(|c| !Rc::ptr_eq(c, child));
    }
}

impl Drop for Owner {
    fn drop(&mut self) {
        if !self.disposed.get() {
            self.disposed.set(true);
            let children = std::mem::take(&mut *self.children.borrow_mut());
            for child in children {
                child.dispose();
            }
            let cleanups = std::mem::take(&mut *self.cleanups.borrow_mut());
            for cleanup in cleanups.into_iter().rev() {
                cleanup();
            }
        }
    }
}

// ── Public API ──────────────────────────────────────────────────────────────

/// Run `f` inside a new root reactive scope. Returns a dispose function.
pub fn create_root<R>(f: impl FnOnce() -> R) -> (R, impl FnOnce()) {
    let owner = Owner::new_root();
    let result = with_owner(owner.clone(), f);
    let dispose = move || owner.dispose();
    (result, dispose)
}

/// Run `f` inside a new child reactive scope under the current owner.
/// All reactive computations created inside `f` are owned by this scope.
/// When the returned `Disposer` is dropped or called, the scope and all
/// its children are cleaned up.
pub fn create_scope<R>(f: impl FnOnce() -> R) -> (R, Disposer) {
    let parent = current_owner().expect("create_scope called outside of reactive scope");
    let child = Owner::new_child(&parent);
    let result = with_owner(child.clone(), f);
    (result, Disposer(Some(child)))
}

/// Register a cleanup function on the current owner. The cleanup runs when
/// the owner is disposed (component unmount, scope destruction, etc.).
pub fn on_cleanup(f: impl FnOnce() + 'static) {
    if let Some(owner) = current_owner() {
        owner.on_cleanup(f);
    }
}

/// Register a callback to run once after the component mounts (deferred to UI loop).
pub fn on_mount(f: impl FnOnce() + 'static) {
    crate::runtime::queue_after_mount(f);
}

/// Provide a context value of type `T` on the current owner.
pub fn provide_context<T: 'static>(value: T) {
    let owner = current_owner().expect("provide_context called outside of reactive scope");
    owner.provide_context(value);
}

/// Look up a context value of type `T` by walking up the owner tree.
pub fn use_context<T: Clone + 'static>() -> Option<T> {
    let owner = current_owner()?;
    owner.use_context::<T>()
}

/// A handle that disposes a reactive scope when dropped or explicitly called.
pub struct Disposer(Option<Rc<Owner>>);

impl Disposer {
    /// Dispose the scope immediately.
    pub fn dispose(mut self) {
        if let Some(owner) = self.0.take() {
            // Remove from parent's children before disposing
            if let Some(parent) = owner.parent.as_ref().and_then(Weak::upgrade) {
                parent.remove_child(&owner);
            }
            owner.dispose();
        }
    }
}

impl Drop for Disposer {
    fn drop(&mut self) {
        if let Some(owner) = self.0.take() {
            if let Some(parent) = owner.parent.as_ref().and_then(Weak::upgrade) {
                parent.remove_child(&owner);
            }
            owner.dispose();
        }
    }
}

// ── Internal helpers ────────────────────────────────────────────────────────

/// Get the current owner from the thread-local stack.
pub(crate) fn current_owner() -> Option<Rc<Owner>> {
    OWNER.with(|o| o.borrow().clone())
}

/// Run `f` with the given owner set as the current owner.
pub(crate) fn with_owner<R>(owner: Rc<Owner>, f: impl FnOnce() -> R) -> R {
    OWNER.with(|o| {
        let previous = o.replace(Some(owner));
        let result = f();
        o.replace(previous);
        result
    })
}

/// Create a child owner under the current owner and run `f` inside it.
/// Returns the result and keeps the child owner alive in the parent.
pub(crate) fn with_child_owner<R>(f: impl FnOnce() -> R) -> (R, Rc<Owner>) {
    let parent = current_owner().expect("with_child_owner called outside of reactive scope");
    let child = Owner::new_child(&parent);
    let result = with_owner(child.clone(), f);
    (result, child)
}

// ── Scope guard (for component macro) ──────────────────────────────────────────

/// A guard that pushes a new child owner onto the thread-local OWNER stack.
///
/// Designed for use by the `#[component]` macro so that the component body
/// runs **directly** (not inside a closure), giving the LSP full visibility
/// into parameter references and local bindings.
///
/// ```ignore
/// let guard = enter_scope();
/// let result = { /* component body — LSP can analyse this */ };
/// let child_owner = guard.exit();
/// scope_owned(child_owner, result)
/// ```
#[doc(hidden)]
pub struct ScopeGuard {
    previous: Option<Rc<Owner>>,
    child: Rc<Owner>,
}

/// Push a new child owner onto the OWNER stack, returning a guard.
/// The guard MUST be paired with `.exit()` — dropping without calling
/// `.exit()` will restore the previous owner as a safety measure.
#[doc(hidden)]
pub fn enter_scope() -> ScopeGuard {
    let parent = current_owner().expect("enter_scope called outside of reactive scope");
    let child = Owner::new_child(&parent);
    let previous = OWNER.with(|o| o.replace(Some(child.clone())));
    ScopeGuard { previous, child }
}

impl ScopeGuard {
    /// Pop the child owner off the stack, restoring the previous owner.
    /// Returns the child `Rc<Owner>` for use by downstream APIs (e.g. `scope_owned`).
    pub fn exit(mut self) -> Rc<Owner> {
        // Manually destructure to avoid E0509 (cannot move out of Drop type).
        let previous = std::mem::replace(&mut self.previous, None);
        let child = self.child.clone();
        OWNER.with(|o| o.replace(previous));
        // Prevent Drop from running — we already restored the owner.
        std::mem::forget(self);
        child
    }
}

impl Drop for ScopeGuard {
    fn drop(&mut self) {
        // Safety net: if exit() was never called (e.g. due to panic),
        // restore the previous owner to avoid leaving a stale child as current.
        if let Some(previous) = self.previous.take() {
            OWNER.with(|o| o.replace(Some(previous)));
        }
    }
}
