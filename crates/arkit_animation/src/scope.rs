use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

use arkit_animation_core::{PlaybackSettings, ScopeMethodName, WindowMetrics};
use arkit_prelude::*;
use rustc_hash::FxHashMap;

use crate::{use_animation, AnimationControls, Timeline};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WindowCondition {
    MinWidth(f32),
    MaxWidth(f32),
    MinHeight(f32),
    MaxHeight(f32),
    Portrait,
    Landscape,
}

impl WindowCondition {
    pub fn matches(self, metrics: WindowMetrics) -> bool {
        match self {
            Self::MinWidth(value) => metrics.width_vp >= value,
            Self::MaxWidth(value) => metrics.width_vp <= value,
            Self::MinHeight(value) => metrics.height_vp >= value,
            Self::MaxHeight(value) => metrics.height_vp <= value,
            Self::Portrait => metrics.height_vp >= metrics.width_vp,
            Self::Landscape => metrics.width_vp > metrics.height_vp,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScopeCleanupPolicy {
    Cancel,
    #[default]
    Revert,
}

#[derive(Debug, Clone)]
pub struct AnimationScopeDefaults {
    pub playback: PlaybackSettings,
    pub cleanup: ScopeCleanupPolicy,
    pub keep_time: bool,
}

impl Default for AnimationScopeDefaults {
    fn default() -> Self {
        Self {
            playback: PlaybackSettings::default(),
            cleanup: ScopeCleanupPolicy::Revert,
            keep_time: false,
        }
    }
}

type ScopeMethod = Rc<dyn Fn()>;
type EventCleanup = Box<dyn FnOnce()>;

struct AnimationScopeInner {
    defaults: RefCell<AnimationScopeDefaults>,
    controls: RefCell<Vec<Weak<crate::controls::ControlsInner>>>,
    methods: RefCell<FxHashMap<ScopeMethodName, ScopeMethod>>,
    event_cleanups: RefCell<Vec<EventCleanup>>,
    disposed: Cell<bool>,
}

impl Drop for AnimationScopeInner {
    fn drop(&mut self) {
        self.disposed.set(true);
        for cleanup in self.event_cleanups.get_mut().drain(..).rev() {
            cleanup();
        }
        for inner in self
            .controls
            .get_mut()
            .drain(..)
            .filter_map(|weak| weak.upgrade())
        {
            let controls = AnimationControls { inner };
            match self.defaults.get_mut().cleanup {
                ScopeCleanupPolicy::Cancel => controls.cancel(),
                ScopeCleanupPolicy::Revert => controls.revert(),
            }
        }
        self.methods.get_mut().clear();
    }
}

#[derive(Clone)]
pub struct AnimationScope {
    inner: Rc<AnimationScopeInner>,
}

impl AnimationScope {
    pub fn new(defaults: AnimationScopeDefaults) -> Self {
        Self {
            inner: Rc::new(AnimationScopeInner {
                defaults: RefCell::new(defaults),
                controls: RefCell::new(Vec::new()),
                methods: RefCell::new(FxHashMap::default()),
                event_cleanups: RefCell::new(Vec::new()),
                disposed: Cell::new(false),
            }),
        }
    }

    pub fn defaults(&self) -> AnimationScopeDefaults {
        self.inner.defaults.borrow().clone()
    }

    pub fn set_defaults(&self, defaults: AnimationScopeDefaults) {
        *self.inner.defaults.borrow_mut() = defaults;
    }

    pub fn is_disposed(&self) -> bool {
        self.inner.disposed.get()
    }

    pub fn register(&self, controls: AnimationControls) {
        if self.is_disposed() {
            return;
        }
        let identity = controls.identity();
        let mut owned = self.inner.controls.borrow_mut();
        if !owned.iter().any(|candidate| candidate.as_ptr() == identity) {
            owned.push(Rc::downgrade(&controls.inner));
        }
    }

    pub fn method(&self, name: ScopeMethodName, callback: impl Fn() + 'static) {
        if !self.is_disposed() {
            self.inner
                .methods
                .borrow_mut()
                .insert(name, Rc::new(callback));
        }
    }

    pub fn call(&self, name: &ScopeMethodName) -> bool {
        let callback = self.inner.methods.borrow().get(name).cloned();
        if let Some(callback) = callback {
            callback();
            true
        } else {
            false
        }
    }

    pub fn register_event_cleanup(&self, cleanup: impl FnOnce() + 'static) {
        if !self.is_disposed() {
            self.inner
                .event_cleanups
                .borrow_mut()
                .push(Box::new(cleanup));
        }
    }

    pub fn refresh(&self) {
        for controls in self.live_controls() {
            controls.refresh();
        }
    }

    pub fn revert(&self) {
        for controls in self.live_controls() {
            controls.revert();
        }
    }

    fn live_controls(&self) -> Vec<AnimationControls> {
        let mut owned = self.inner.controls.borrow_mut();
        let mut live = Vec::with_capacity(owned.len());
        owned.retain(|weak| {
            let Some(inner) = weak.upgrade() else {
                return false;
            };
            live.push(AnimationControls { inner });
            true
        });
        live
    }
}

#[track_caller]
pub fn use_animation_scope(defaults: AnimationScopeDefaults) -> AnimationScope {
    let initial = defaults.clone();
    let scope = use_hook(|| AnimationScope::new(initial));
    scope.set_defaults(defaults);
    scope
}

#[track_caller]
pub fn use_scoped_animation(scope: &AnimationScope, timeline: Timeline) -> AnimationControls {
    let defaults = scope.defaults();
    let controls = use_animation(timeline.settings(defaults.playback));
    scope.register(controls.clone());
    let cleanup = defaults.cleanup;
    let cleanup_controls = controls.clone();
    use_drop(move || match cleanup {
        ScopeCleanupPolicy::Cancel => cleanup_controls.cancel(),
        ScopeCleanupPolicy::Revert => cleanup_controls.revert(),
    });
    controls
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_conditions_are_typed_and_deterministic() {
        let metrics = WindowMetrics {
            width_vp: 800.0,
            height_vp: 600.0,
            density: 2.0,
        };
        assert!(WindowCondition::MinWidth(700.0).matches(metrics));
        assert!(WindowCondition::Landscape.matches(metrics));
        assert!(!WindowCondition::Portrait.matches(metrics));
    }
}
