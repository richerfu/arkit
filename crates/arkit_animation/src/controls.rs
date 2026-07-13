use std::cell::{Cell, RefCell};
use std::future::Future;
use std::pin::Pin;
use std::rc::{Rc, Weak};
use std::task::{Context, Poll, Waker};

use arkit_animation_core::{
    AnimationInstanceSnapshot, AnimationOutcome, EngineCommand, EngineEvent, InstanceId,
    PlaybackDirection, PlaybackRate, SeekMode, TimePoint, TimeSpan,
};
use arkit_hooks::ArkNodeRef;

use crate::api::Timeline;
use crate::callbacks::AnimationCallbacks;
use crate::{AnimationHost, FrameDriver};
use crate::{CapabilityRequirements, ExecutionPolicy, LoweringReport};

type SnapshotObserver = Rc<dyn Fn(AnimationInstanceSnapshot)>;

#[derive(Default)]
struct FinishedState {
    outcome: Option<AnimationOutcome>,
    waiters: Vec<Waker>,
}

pub struct AnimationFinished {
    state: Rc<RefCell<FinishedState>>,
}

impl Future for AnimationFinished {
    type Output = AnimationOutcome;

    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        let mut state = self.state.borrow_mut();
        if let Some(outcome) = state.outcome {
            return Poll::Ready(outcome);
        }
        if !state
            .waiters
            .iter()
            .any(|waker| waker.will_wake(context.waker()))
        {
            state.waiters.push(context.waker().clone());
        }
        Poll::Pending
    }
}

pub(crate) struct ControlsInner {
    pub host: Rc<AnimationHost>,
    pub driver: Rc<FrameDriver>,
    pub node: ArkNodeRef,
    pub instance: Cell<Option<InstanceId>>,
    pub listener: Cell<Option<usize>>,
    callbacks: RefCell<AnimationCallbacks>,
    finished: Rc<RefCell<FinishedState>>,
    pub source: RefCell<arkit_animation_core::TimelineSource>,
    pub policy: Cell<ExecutionPolicy>,
    pub requirements: Cell<CapabilityRequirements>,
    pub lowering_report: RefCell<Option<LoweringReport>>,
    pub calls: RefCell<Vec<Rc<dyn Fn()>>>,
    observers: RefCell<Vec<Option<SnapshotObserver>>>,
}

impl ControlsInner {
    pub fn new(
        host: Rc<AnimationHost>,
        driver: Rc<FrameDriver>,
        node: ArkNodeRef,
        source: arkit_animation_core::TimelineSource,
        policy: ExecutionPolicy,
        requirements: CapabilityRequirements,
        calls: Vec<Rc<dyn Fn()>>,
    ) -> Rc<Self> {
        let inner = Rc::new(Self {
            host,
            driver,
            node,
            instance: Cell::new(None),
            listener: Cell::new(None),
            callbacks: RefCell::new(AnimationCallbacks::default()),
            finished: Rc::new(RefCell::new(FinishedState::default())),
            source: RefCell::new(source),
            policy: Cell::new(policy),
            requirements: Cell::new(requirements),
            lowering_report: RefCell::new(None),
            calls: RefCell::new(calls),
            observers: RefCell::new(Vec::new()),
        });
        let weak = Rc::downgrade(&inner);
        let listener = inner.host.subscribe(Rc::new(move |event| {
            if let Some(inner) = weak.upgrade() {
                inner.handle_event(event);
            }
        }));
        inner.listener.set(Some(listener));
        inner
    }

    fn handle_event(&self, event: EngineEvent) {
        let Some(instance) = self.instance.get() else {
            return;
        };
        let event_instance = match event {
            EngineEvent::Begin { instance }
            | EngineEvent::BeforeUpdate { instance, .. }
            | EngineEvent::Update { instance, .. }
            | EngineEvent::Render { instance, .. }
            | EngineEvent::Loop { instance, .. }
            | EngineEvent::Pause { instance }
            | EngineEvent::RefreshRequested { instance, .. }
            | EngineEvent::Call { instance, .. }
            | EngineEvent::StateChanged { instance, .. }
            | EngineEvent::Complete { instance }
            | EngineEvent::Cancel { instance }
            | EngineEvent::Revert { instance }
            | EngineEvent::Settled { instance, .. }
            | EngineEvent::Error { instance, .. } => instance,
        };
        if event_instance != instance {
            return;
        }
        let callbacks = self.callbacks.borrow();
        match event {
            EngineEvent::Begin { .. } => invoke(&callbacks.begin),
            EngineEvent::BeforeUpdate { at, .. } => {
                if let Some(callback) = &callbacks.before_update {
                    callback(at);
                }
            }
            EngineEvent::Update { progress, .. } => {
                if let Some(callback) = &callbacks.update {
                    callback(progress);
                }
            }
            EngineEvent::Render { .. } => invoke(&callbacks.render),
            EngineEvent::Loop {
                completed_iterations,
                ..
            } => {
                if let Some(callback) = &callbacks.looped {
                    callback(completed_iterations);
                }
            }
            EngineEvent::Complete { .. } => invoke(&callbacks.complete),
            EngineEvent::Cancel { .. } => invoke(&callbacks.cancel),
            EngineEvent::Pause { .. } => invoke(&callbacks.pause),
            EngineEvent::Settled { outcome, .. } => {
                drop(callbacks);
                let mut finished = self.finished.borrow_mut();
                finished.outcome = Some(outcome);
                for waiter in finished.waiters.drain(..) {
                    waiter.wake();
                }
            }
            EngineEvent::Call { call, .. } => {
                drop(callbacks);
                if let Some(callback) = self.calls.borrow().get(call.index()).cloned() {
                    callback();
                }
            }
            EngineEvent::RefreshRequested { .. } => {
                drop(callbacks);
                if let Err(error) = self.host.refresh_timeline(instance, &self.source.borrow()) {
                    ohos_hilog_binding::error(format!("animation refresh failed: {error}"));
                }
            }
            _ => {}
        }
        if matches!(
            event,
            EngineEvent::Update { .. }
                | EngineEvent::Render { .. }
                | EngineEvent::StateChanged { .. }
                | EngineEvent::Settled { .. }
        ) {
            self.notify_observers();
        }
    }

    fn command(&self, command: impl FnOnce(InstanceId) -> EngineCommand) {
        let Some(instance) = self.instance.get() else {
            return;
        };
        self.host.enqueue(command(instance));
        if let Some(node) = self.node.peek() {
            self.driver.request(&node);
        }
    }

    fn notify_observers(&self) {
        let Some(snapshot) = self
            .instance
            .get()
            .and_then(|instance| self.host.snapshot(instance))
        else {
            return;
        };
        for observer in self.observers.borrow().iter().flatten() {
            observer(snapshot);
        }
    }
}

impl Drop for ControlsInner {
    fn drop(&mut self) {
        if let Some(listener) = self.listener.take() {
            self.host.unsubscribe(listener);
        }
        if let Some(instance) = self.instance.take() {
            self.host.enqueue(EngineCommand::Remove(instance));
        }
    }
}

fn invoke(callback: &Option<Rc<dyn Fn()>>) {
    if let Some(callback) = callback {
        callback();
    }
}

#[derive(Clone)]
pub struct AnimationControls {
    pub(crate) inner: Rc<ControlsInner>,
}

pub struct AnimationSubscription {
    controls: Weak<ControlsInner>,
    id: usize,
}

impl Drop for AnimationSubscription {
    fn drop(&mut self) {
        if let Some(controls) = self.controls.upgrade() {
            if let Some(slot) = controls.observers.borrow_mut().get_mut(self.id) {
                *slot = None;
            }
        }
    }
}

impl AnimationControls {
    pub(crate) fn identity(&self) -> *const ControlsInner {
        Rc::as_ptr(&self.inner)
    }

    pub fn is_ready(&self) -> bool {
        self.inner.instance.get().is_some()
    }

    pub fn play(&self) {
        let mut finished = self.inner.finished.borrow_mut();
        finished.outcome = None;
        drop(finished);
        self.inner.command(EngineCommand::Play);
    }

    pub fn pause(&self) {
        self.inner.command(EngineCommand::Pause);
    }

    pub fn resume(&self) {
        self.inner.command(EngineCommand::Resume);
    }

    pub fn restart(&self) {
        let mut finished = self.inner.finished.borrow_mut();
        finished.outcome = None;
        drop(finished);
        self.inner.command(EngineCommand::Restart);
    }

    pub fn reverse(&self) {
        self.inner.command(EngineCommand::Reverse);
    }

    pub fn complete(&self) {
        self.inner.command(EngineCommand::Complete);
    }

    pub fn cancel(&self) {
        self.inner.command(EngineCommand::Cancel);
    }

    pub fn reset(&self) {
        self.inner.command(EngineCommand::Reset);
    }

    pub fn revert(&self) {
        self.inner.command(EngineCommand::Revert);
    }

    pub fn refresh(&self) {
        self.inner.command(EngineCommand::Refresh);
    }

    pub fn seek(&self, position: TimePoint) {
        self.inner.command(|instance| EngineCommand::Seek {
            instance,
            position,
            mode: SeekMode::SuppressEvents,
        });
    }

    pub fn seek_with_events(&self, position: TimePoint) {
        self.inner.command(|instance| EngineCommand::Seek {
            instance,
            position,
            mode: SeekMode::FireCrossingEvents,
        });
    }

    pub fn stretch(&self, duration: TimeSpan) {
        self.inner
            .command(|instance| EngineCommand::Stretch { instance, duration });
    }

    pub fn set_playback_rate(&self, rate: PlaybackRate) {
        self.inner
            .command(|instance| EngineCommand::SetPlaybackRate { instance, rate });
    }

    pub fn set_alternate(&self, enabled: bool) {
        self.inner
            .command(|instance| EngineCommand::SetAlternate { instance, enabled });
    }

    pub fn set_timeline(&self, timeline: Timeline) {
        let (source, policy, requirements, calls) = timeline.into_parts();
        *self.inner.source.borrow_mut() = source;
        self.inner.policy.set(policy);
        self.inner.requirements.set(requirements);
        self.inner.calls.replace(calls);
        if let Some(instance) = self.inner.instance.get() {
            if let Err(error) = self
                .inner
                .host
                .refresh_timeline(instance, &self.inner.source.borrow())
            {
                ohos_hilog_binding::error(format!(
                    "animation timeline replacement failed: {error}"
                ));
            }
        }
    }

    pub fn lowering_report(&self) -> Option<LoweringReport> {
        self.inner.lowering_report.borrow().clone()
    }

    pub fn subscribe(
        &self,
        observer: impl Fn(AnimationInstanceSnapshot) + 'static,
    ) -> AnimationSubscription {
        let mut observers = self.inner.observers.borrow_mut();
        let observer = Rc::new(observer) as SnapshotObserver;
        let id = if let Some((id, slot)) = observers
            .iter_mut()
            .enumerate()
            .find(|(_, slot)| slot.is_none())
        {
            *slot = Some(observer);
            id
        } else {
            observers.push(Some(observer));
            observers.len() - 1
        };
        AnimationSubscription {
            controls: Rc::downgrade(&self.inner),
            id,
        }
    }

    pub fn snapshot(&self) -> Option<AnimationInstanceSnapshot> {
        self.inner
            .instance
            .get()
            .and_then(|instance| self.inner.host.snapshot(instance))
    }

    pub fn direction(&self) -> Option<PlaybackDirection> {
        self.snapshot().map(|snapshot| snapshot.direction)
    }

    pub fn finished(&self) -> AnimationFinished {
        AnimationFinished {
            state: self.inner.finished.clone(),
        }
    }

    pub fn on_begin(&self, callback: impl Fn() + 'static) {
        self.inner.callbacks.borrow_mut().begin = Some(Rc::new(callback));
    }

    pub fn on_update(&self, callback: impl Fn(f32) + 'static) {
        self.inner.callbacks.borrow_mut().update = Some(Rc::new(callback));
    }

    pub fn on_before_update(&self, callback: impl Fn(TimePoint) + 'static) {
        self.inner.callbacks.borrow_mut().before_update = Some(Rc::new(callback));
    }

    pub fn on_render(&self, callback: impl Fn() + 'static) {
        self.inner.callbacks.borrow_mut().render = Some(Rc::new(callback));
    }

    pub fn on_loop(&self, callback: impl Fn(u32) + 'static) {
        self.inner.callbacks.borrow_mut().looped = Some(Rc::new(callback));
    }

    pub fn on_complete(&self, callback: impl Fn() + 'static) {
        self.inner.callbacks.borrow_mut().complete = Some(Rc::new(callback));
    }

    pub fn on_cancel(&self, callback: impl Fn() + 'static) {
        self.inner.callbacks.borrow_mut().cancel = Some(Rc::new(callback));
    }

    pub fn on_pause(&self, callback: impl Fn() + 'static) {
        self.inner.callbacks.borrow_mut().pause = Some(Rc::new(callback));
    }
}
