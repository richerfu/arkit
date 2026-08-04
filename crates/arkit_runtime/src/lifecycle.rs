//! Application lifecycle state shared by the OpenHarmony runtime and hooks.

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::rc::{Rc, Weak};

use openharmony_ability::Event as AbilityEvent;

/// Coarse application execution phase.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ApplicationLifecyclePhase {
    /// The mounted window is visible and the application may use foreground-only resources.
    #[default]
    Foreground,
    /// The application is paused or its window is hidden.
    Background,
    /// The owning ability has been destroyed.
    Destroyed,
}

/// A normalized lifecycle event emitted by the OpenHarmony host.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApplicationLifecycleEvent {
    AbilityCreated,
    AbilityDestroyed,
    WindowCreated,
    WindowDestroyed,
    WindowShown,
    WindowHidden,
    Resumed,
    Paused,
    FocusGained,
    FocusLost,
    SurfaceCreated,
    SurfaceDestroyed,
    LowMemory,
}

impl ApplicationLifecycleEvent {
    fn from_ability_event(event: &AbilityEvent<'_>) -> Option<Self> {
        Some(match event {
            AbilityEvent::Create => Self::AbilityCreated,
            AbilityEvent::Destroy => Self::AbilityDestroyed,
            AbilityEvent::WindowCreate => Self::WindowCreated,
            AbilityEvent::WindowDestroy => Self::WindowDestroyed,
            AbilityEvent::Start => Self::WindowShown,
            AbilityEvent::Stop => Self::WindowHidden,
            AbilityEvent::Resume(_) => Self::Resumed,
            AbilityEvent::Pause => Self::Paused,
            AbilityEvent::GainedFocus => Self::FocusGained,
            AbilityEvent::LostFocus => Self::FocusLost,
            AbilityEvent::SurfaceCreate => Self::SurfaceCreated,
            AbilityEvent::SurfaceDestroy => Self::SurfaceDestroyed,
            AbilityEvent::LowMemory => Self::LowMemory,
            AbilityEvent::WindowRedraw(_)
            | AbilityEvent::WindowResize(_)
            | AbilityEvent::ContentRectChange(_)
            | AbilityEvent::AvoidAreaChange(_)
            | AbilityEvent::ConfigChanged(_)
            | AbilityEvent::SaveState(_)
            | AbilityEvent::Input(_)
            | AbilityEvent::KeyboardEvent(_)
            | AbilityEvent::UserEvent => return None,
        })
    }
}

/// Current application/window lifecycle snapshot.
///
/// A freshly mounted Arkit runtime starts in the foreground. OpenHarmony may
/// have delivered the initial create/show callbacks before the Rust
/// `VirtualDom` is mounted, so treating the initial snapshot as background
/// would leave foreground-only components permanently suspended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ApplicationLifecycleState {
    pub phase: ApplicationLifecyclePhase,
    pub window_available: bool,
    pub window_visible: bool,
    pub focused: bool,
    pub surface_available: bool,
    /// Monotonic count of memory-pressure notifications received by this runtime.
    pub low_memory_events: u64,
}

impl Default for ApplicationLifecycleState {
    fn default() -> Self {
        Self {
            phase: ApplicationLifecyclePhase::Foreground,
            window_available: true,
            window_visible: true,
            focused: true,
            surface_available: false,
            low_memory_events: 0,
        }
    }
}

impl ApplicationLifecycleState {
    /// Whether foreground-only work may run.
    pub fn is_foreground(self) -> bool {
        self.phase == ApplicationLifecyclePhase::Foreground
            && self.window_available
            && self.window_visible
    }

    fn apply(mut self, event: ApplicationLifecycleEvent) -> Self {
        match event {
            ApplicationLifecycleEvent::AbilityCreated => {
                if self.phase == ApplicationLifecyclePhase::Destroyed {
                    self.phase = ApplicationLifecyclePhase::Background;
                }
            }
            ApplicationLifecycleEvent::AbilityDestroyed => {
                self.phase = ApplicationLifecyclePhase::Destroyed;
                self.window_available = false;
                self.window_visible = false;
                self.focused = false;
                self.surface_available = false;
            }
            ApplicationLifecycleEvent::WindowCreated => {
                self.window_available = true;
            }
            ApplicationLifecycleEvent::WindowDestroyed => {
                self.phase = ApplicationLifecyclePhase::Background;
                self.window_available = false;
                self.window_visible = false;
                self.focused = false;
                self.surface_available = false;
            }
            ApplicationLifecycleEvent::WindowShown => {
                self.phase = ApplicationLifecyclePhase::Foreground;
                self.window_available = true;
                self.window_visible = true;
            }
            ApplicationLifecycleEvent::WindowHidden => {
                self.phase = ApplicationLifecyclePhase::Background;
                self.window_visible = false;
                self.focused = false;
            }
            ApplicationLifecycleEvent::Resumed => {
                self.phase = ApplicationLifecyclePhase::Foreground;
            }
            ApplicationLifecycleEvent::Paused => {
                self.phase = ApplicationLifecyclePhase::Background;
                self.focused = false;
            }
            ApplicationLifecycleEvent::FocusGained => self.focused = true,
            ApplicationLifecycleEvent::FocusLost => self.focused = false,
            ApplicationLifecycleEvent::SurfaceCreated => self.surface_available = true,
            ApplicationLifecycleEvent::SurfaceDestroyed => self.surface_available = false,
            ApplicationLifecycleEvent::LowMemory => {
                self.low_memory_events = self.low_memory_events.saturating_add(1);
            }
        }
        self
    }
}

type LifecycleSubscriber = Rc<dyn Fn(ApplicationLifecycleEvent, ApplicationLifecycleState)>;

/// Shared lifecycle owner installed as a root Dioxus context by [`crate::ArkRuntime`].
#[derive(Clone)]
pub struct ApplicationLifecycleHandle(Rc<ApplicationLifecycleHandleInner>);

pub(crate) struct ApplicationLifecycleHandleInner {
    state: Cell<ApplicationLifecycleState>,
    subscribers: RefCell<BTreeMap<usize, LifecycleSubscriber>>,
    next_subscriber_id: Cell<usize>,
}

impl ApplicationLifecycleHandle {
    pub(crate) fn new(state: ApplicationLifecycleState) -> Self {
        Self(Rc::new(ApplicationLifecycleHandleInner {
            state: Cell::new(state),
            subscribers: RefCell::new(BTreeMap::new()),
            next_subscriber_id: Cell::new(0),
        }))
    }

    /// Weak reference for process-lifetime closures that must not keep a
    /// retired runtime's lifecycle state alive.
    pub(crate) fn downgrade(&self) -> std::rc::Weak<ApplicationLifecycleHandleInner> {
        Rc::downgrade(&self.0)
    }

    /// Re-wrap a strong inner reference (upgraded from a [`Self::downgrade`]).
    pub(crate) fn from_inner(inner: Rc<ApplicationLifecycleHandleInner>) -> Self {
        Self(inner)
    }

    pub fn get(&self) -> ApplicationLifecycleState {
        self.0.state.get()
    }

    /// Subscribe to normalized lifecycle events and their resulting snapshots.
    pub fn subscribe(
        &self,
        callback: impl Fn(ApplicationLifecycleEvent, ApplicationLifecycleState) + 'static,
    ) -> ApplicationLifecycleSubscription {
        let id = self.0.next_subscriber_id.get();
        self.0.next_subscriber_id.set(
            id.checked_add(1)
                .expect("arkit_runtime: lifecycle subscriber id space exhausted"),
        );
        self.0
            .subscribers
            .borrow_mut()
            .insert(id, Rc::new(callback));
        ApplicationLifecycleSubscription {
            _inner: Rc::new(ApplicationLifecycleSubscriptionInner {
                handle: Rc::downgrade(&self.0),
                id,
            }),
        }
    }

    pub(crate) fn update_from_ability_event(&self, event: &AbilityEvent<'_>) -> bool {
        let Some(event) = ApplicationLifecycleEvent::from_ability_event(event) else {
            return false;
        };
        let state = self.get().apply(event);
        self.0.state.set(state);
        let subscribers = self
            .0
            .subscribers
            .borrow()
            .values()
            .cloned()
            .collect::<Vec<_>>();
        for subscriber in subscribers {
            subscriber(event, state);
        }
        true
    }

    pub(crate) fn handles_ability_event(event: &AbilityEvent<'_>) -> bool {
        ApplicationLifecycleEvent::from_ability_event(event).is_some()
    }
}

/// Lifetime guard for an [`ApplicationLifecycleHandle`] subscription.
#[derive(Clone)]
pub struct ApplicationLifecycleSubscription {
    _inner: Rc<ApplicationLifecycleSubscriptionInner>,
}

struct ApplicationLifecycleSubscriptionInner {
    handle: Weak<ApplicationLifecycleHandleInner>,
    id: usize,
}

impl Drop for ApplicationLifecycleSubscriptionInner {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.upgrade() {
            handle.subscribers.borrow_mut().remove(&self.id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pause_and_resume_gate_foreground_work() {
        let handle = ApplicationLifecycleHandle::new(ApplicationLifecycleState::default());

        assert!(handle.get().is_foreground());
        assert!(handle.update_from_ability_event(&AbilityEvent::Pause));
        assert!(!handle.get().is_foreground());
        assert!(handle.update_from_ability_event(&AbilityEvent::Start));
        assert!(handle.get().is_foreground());
    }

    #[test]
    fn hidden_window_stays_background_until_shown() {
        let handle = ApplicationLifecycleHandle::new(ApplicationLifecycleState::default());

        handle.update_from_ability_event(&AbilityEvent::Stop);
        handle.update_from_ability_event(&AbilityEvent::GainedFocus);
        assert!(!handle.get().is_foreground());

        handle.update_from_ability_event(&AbilityEvent::Start);
        assert!(handle.get().is_foreground());
    }

    #[test]
    fn subscriptions_receive_normalized_events_and_are_revoked_on_drop() {
        let handle = ApplicationLifecycleHandle::new(ApplicationLifecycleState::default());
        let observed = Rc::new(RefCell::new(Vec::new()));
        let callback_observed = observed.clone();
        let subscription = handle.subscribe(move |event, state| {
            callback_observed.borrow_mut().push((event, state));
        });

        handle.update_from_ability_event(&AbilityEvent::Pause);
        assert_eq!(observed.borrow().len(), 1);
        assert_eq!(observed.borrow()[0].0, ApplicationLifecycleEvent::Paused);
        assert!(!observed.borrow()[0].1.is_foreground());

        drop(subscription);
        handle.update_from_ability_event(&AbilityEvent::Start);
        assert_eq!(observed.borrow().len(), 1);
    }
}
