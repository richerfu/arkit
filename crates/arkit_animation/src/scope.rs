use std::cell::{Cell, RefCell};
use std::rc::Rc;

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
    defaults: AnimationScopeDefaults,
    controls: RefCell<Vec<AnimationControls>>,
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
        for controls in self.controls.get_mut() {
            match self.defaults.cleanup {
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
                defaults,
                controls: RefCell::new(Vec::new()),
                methods: RefCell::new(FxHashMap::default()),
                event_cleanups: RefCell::new(Vec::new()),
                disposed: Cell::new(false),
            }),
        }
    }

    pub fn defaults(&self) -> &AnimationScopeDefaults {
        &self.inner.defaults
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
        if !owned
            .iter()
            .any(|candidate| candidate.identity() == identity)
        {
            owned.push(controls);
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
        for controls in self.inner.controls.borrow().iter() {
            controls.refresh();
        }
    }

    pub fn revert(&self) {
        for controls in self.inner.controls.borrow().iter() {
            controls.revert();
        }
    }
}

#[track_caller]
pub fn use_animation_scope(defaults: AnimationScopeDefaults) -> AnimationScope {
    use_hook(|| AnimationScope::new(defaults))
}

#[track_caller]
pub fn use_scoped_animation(scope: &AnimationScope, timeline: Timeline) -> AnimationControls {
    let controls = use_animation(timeline.settings(scope.defaults().playback.clone()));
    scope.register(controls.clone());
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
