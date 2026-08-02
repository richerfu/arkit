//! Per-root runtime services.

use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::rc::{Rc, Weak};
use std::task::Waker;

use super::EmbeddedRuntimeInner;

type BackPressHandler = Rc<dyn Fn() -> bool>;

/// Stable identity for one mounted Arkit root.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RuntimeId(u64);

impl RuntimeId {
    pub(crate) const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u64 {
        self.0
    }
}

struct RuntimeSessionState {
    id: RuntimeId,
    next_registration: Cell<u64>,
    active: Cell<bool>,
    ui_waker: RefCell<Option<Rc<dyn Fn()>>>,
    scheduler_waker: RefCell<Option<Waker>>,
    ui_effects: RefCell<VecDeque<Box<dyn FnOnce()>>>,
    async_handle: tokio::runtime::Handle,
    back_handlers: RefCell<Vec<(u64, BackPressHandler)>>,
    embedded_runtimes: RefCell<Vec<(u64, Weak<RefCell<EmbeddedRuntimeInner>>)>>,
}

/// Cloneable, root-specific UI/async/back-dispatch handle.
#[derive(Clone)]
pub struct RuntimeHandle {
    state: Rc<RuntimeSessionState>,
}

impl std::fmt::Debug for RuntimeHandle {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RuntimeHandle")
            .field("id", &self.id())
            .field("active", &self.is_active())
            .finish()
    }
}

impl PartialEq for RuntimeHandle {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.state, &other.state)
    }
}

impl Eq for RuntimeHandle {}

impl RuntimeHandle {
    pub(crate) fn new(id: RuntimeId, async_handle: tokio::runtime::Handle) -> Self {
        Self {
            state: Rc::new(RuntimeSessionState {
                id,
                next_registration: Cell::new(0),
                active: Cell::new(true),
                ui_waker: RefCell::new(None),
                scheduler_waker: RefCell::new(None),
                ui_effects: RefCell::new(VecDeque::new()),
                async_handle,
                back_handlers: RefCell::new(Vec::new()),
                embedded_runtimes: RefCell::new(Vec::new()),
            }),
        }
    }

    pub fn id(&self) -> RuntimeId {
        self.state.id
    }

    pub fn is_active(&self) -> bool {
        self.state.active.get()
    }

    pub fn tokio(&self) -> tokio::runtime::Handle {
        self.state.async_handle.clone()
    }

    /// Queue owned work for this root's next UI tick.
    pub fn queue_ui(&self, effect: impl FnOnce() + 'static) {
        if !self.is_active() {
            return;
        }
        self.state
            .ui_effects
            .borrow_mut()
            .push_back(Box::new(effect));
        self.wake();
    }

    pub fn register_back_handler(&self, handler: Rc<dyn Fn() -> bool>) -> BackPressRegistration {
        if !self.is_active() {
            return BackPressRegistration {
                state: Weak::new(),
                id: 0,
            };
        }
        let id = self.next_registration();
        self.state.back_handlers.borrow_mut().push((id, handler));
        BackPressRegistration {
            state: Rc::downgrade(&self.state),
            id,
        }
    }

    pub(crate) fn set_ui_waker(&self, waker: Rc<dyn Fn()>) {
        *self.state.ui_waker.borrow_mut() = Some(waker);
    }

    pub(crate) fn set_scheduler_waker(&self, waker: Waker) {
        *self.state.scheduler_waker.borrow_mut() = Some(waker);
    }

    pub(crate) fn scheduler_waker(&self) -> Waker {
        self.state
            .scheduler_waker
            .borrow()
            .clone()
            .expect("arkit_runtime: scheduler waker is not installed")
    }

    pub(crate) fn wake(&self) {
        if !self.is_active() {
            return;
        }
        let waker = self.state.ui_waker.borrow().clone();
        if let Some(waker) = waker {
            waker();
        }
    }

    pub(crate) fn run_ui_effects(&self) -> Vec<Box<dyn FnOnce()>> {
        if !self.is_active() {
            return Vec::new();
        }
        self.state.ui_effects.borrow_mut().drain(..).collect()
    }

    pub(crate) fn dispatch_back_press(&self) -> bool {
        if !self.is_active() {
            return false;
        }
        let handlers = self
            .state
            .back_handlers
            .borrow()
            .iter()
            .rev()
            .map(|(_, handler)| handler.clone())
            .collect::<Vec<_>>();
        handlers.into_iter().any(|handler| handler())
    }

    pub(crate) fn register_embedded(
        &self,
        runtime: Weak<RefCell<EmbeddedRuntimeInner>>,
    ) -> EmbeddedRuntimeRegistration {
        if !self.is_active() {
            return EmbeddedRuntimeRegistration {
                state: Weak::new(),
                id: 0,
            };
        }
        let id = self.next_registration();
        self.state
            .embedded_runtimes
            .borrow_mut()
            .push((id, runtime));
        EmbeddedRuntimeRegistration {
            state: Rc::downgrade(&self.state),
            id,
        }
    }

    pub(crate) fn embedded_runtimes(&self) -> Vec<Rc<RefCell<EmbeddedRuntimeInner>>> {
        if !self.is_active() {
            return Vec::new();
        }
        let mut runtimes = self.state.embedded_runtimes.borrow_mut();
        runtimes.retain(|(_, runtime)| runtime.strong_count() > 0);
        runtimes
            .iter()
            .filter_map(|(_, runtime)| runtime.upgrade())
            .collect()
    }

    pub(crate) fn close(&self) {
        if !self.state.active.replace(false) {
            return;
        }
        self.state.ui_waker.borrow_mut().take();
        self.state.scheduler_waker.borrow_mut().take();
        self.state.ui_effects.borrow_mut().clear();
        self.state.back_handlers.borrow_mut().clear();
        self.state.embedded_runtimes.borrow_mut().clear();
    }

    fn next_registration(&self) -> u64 {
        let id = self
            .state
            .next_registration
            .get()
            .checked_add(1)
            .expect("arkit_runtime: per-root registration space exhausted");
        self.state.next_registration.set(id);
        id
    }
}

/// Consume the handle for the exact Dioxus root containing the current scope.
#[track_caller]
pub fn use_runtime_handle() -> RuntimeHandle {
    dioxus_core::consume_context::<RuntimeHandle>()
}

/// RAII registration for one root-local back-press handler.
pub struct BackPressRegistration {
    state: Weak<RuntimeSessionState>,
    id: u64,
}

impl Drop for BackPressRegistration {
    fn drop(&mut self) {
        if let Some(state) = self.state.upgrade() {
            state
                .back_handlers
                .borrow_mut()
                .retain(|(id, _)| *id != self.id);
        }
    }
}

pub(crate) struct EmbeddedRuntimeRegistration {
    state: Weak<RuntimeSessionState>,
    id: u64,
}

impl Drop for EmbeddedRuntimeRegistration {
    fn drop(&mut self) {
        if let Some(state) = self.state.upgrade() {
            state
                .embedded_runtimes
                .borrow_mut()
                .retain(|(id, _)| *id != self.id);
        }
    }
}
