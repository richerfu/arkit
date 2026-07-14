use std::cell::{Cell, RefCell};
use std::rc::Rc;

use arkit_animation_core::{Easing, TimePoint, TimeSpan};

use crate::frame_driver::FrameSourceSubscription;
use crate::{AnimationControls, FrameDriver};
use arkit_prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScrollAxis {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScrollDirection {
    Stationary,
    Forward,
    Backward,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScrollThreshold {
    Numeric(f32),
    Start,
    Center,
    End,
    Relative(f32),
    Min,
    Max,
}

impl ScrollThreshold {
    pub fn resolve(self, target_start: f32, target_extent: f32, container_extent: f32) -> f32 {
        match self {
            Self::Numeric(value) => value,
            Self::Start | Self::Min => target_start,
            Self::Center => target_start + (target_extent - container_extent) * 0.5,
            Self::End | Self::Max => target_start + target_extent - container_extent,
            Self::Relative(factor) => target_start + target_extent * factor,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrollRange {
    pub start: f32,
    pub end: f32,
}

impl ScrollRange {
    pub fn progress(self, offset: f32) -> f32 {
        let extent = self.end - self.start;
        if extent.abs() <= f32::EPSILON {
            return f32::from(offset >= self.end);
        }
        ((offset - self.start) / extent).clamp(0.0, 1.0)
    }

    pub fn contains(self, offset: f32) -> bool {
        let min = self.start.min(self.end);
        let max = self.start.max(self.end);
        offset >= min && offset <= max
    }
}

#[derive(Clone)]
pub enum ScrollSync {
    Method,
    Progress,
    Eased(Easing),
    Smooth { factor: f32, easing: Easing },
}

impl std::fmt::Debug for ScrollSync {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Method => formatter.write_str("Method"),
            Self::Progress => formatter.write_str("Progress"),
            Self::Eased(easing) => formatter.debug_tuple("Eased").field(easing).finish(),
            Self::Smooth { factor, easing } => formatter
                .debug_struct("Smooth")
                .field("factor", factor)
                .field("easing", easing)
                .finish(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrollSample {
    pub at: TimePoint,
    pub offset: f32,
    pub progress: f32,
    pub velocity: f32,
    pub direction: ScrollDirection,
    pub in_view: bool,
}

#[derive(Default, Clone)]
pub struct ScrollCallbacks {
    pub enter: Option<Rc<dyn Fn(ScrollSample)>>,
    pub leave: Option<Rc<dyn Fn(ScrollSample)>>,
    pub forward: Option<Rc<dyn Fn(ScrollSample)>>,
    pub backward: Option<Rc<dyn Fn(ScrollSample)>>,
    pub update: Option<Rc<dyn Fn(ScrollSample)>>,
}

#[derive(Debug, Clone, Copy)]
struct PendingScroll {
    at: TimePoint,
    offset: f32,
}

pub struct ScrollObserver {
    controls: AnimationControls,
    axis: Cell<ScrollAxis>,
    range: Cell<ScrollRange>,
    duration: Cell<TimeSpan>,
    sync: RefCell<ScrollSync>,
    callbacks: RefCell<ScrollCallbacks>,
    repeat: Cell<bool>,
    once: Cell<bool>,
    consumed: Cell<bool>,
    pending: Cell<Option<PendingScroll>>,
    last: RefCell<Option<ScrollSample>>,
    smoothed_progress: Cell<f32>,
    driver: RefCell<Option<Rc<FrameDriver>>>,
    frame_source: RefCell<Option<FrameSourceSubscription>>,
}

impl ScrollObserver {
    pub fn new(controls: AnimationControls, range: ScrollRange, duration: TimeSpan) -> Self {
        Self {
            controls,
            axis: Cell::new(ScrollAxis::Vertical),
            range: Cell::new(range),
            duration: Cell::new(duration),
            sync: RefCell::new(ScrollSync::Progress),
            callbacks: RefCell::new(ScrollCallbacks::default()),
            repeat: Cell::new(true),
            once: Cell::new(false),
            consumed: Cell::new(false),
            pending: Cell::new(None),
            last: RefCell::new(None),
            smoothed_progress: Cell::new(0.0),
            driver: RefCell::new(None),
            frame_source: RefCell::new(None),
        }
    }

    pub fn axis(self, axis: ScrollAxis) -> Self {
        self.axis.set(axis);
        self
    }

    pub fn sync(self, sync: ScrollSync) -> Self {
        *self.sync.borrow_mut() = sync;
        self
    }

    pub fn callbacks(self, callbacks: ScrollCallbacks) -> Self {
        *self.callbacks.borrow_mut() = callbacks;
        self
    }

    pub fn repeat(self, repeat: bool) -> Self {
        self.repeat.set(repeat);
        self
    }

    pub fn once(self, once: bool) -> Self {
        self.once.set(once);
        self
    }

    pub fn axis_kind(&self) -> ScrollAxis {
        self.axis.get()
    }

    /// Stores only the latest event. Call [`Self::flush_frame`] from the root
    /// frame boundary to coalesce multiple platform events into one command.
    pub fn update_at(&self, at: TimePoint, offset: f32) {
        if !self.consumed.get() {
            self.pending.set(Some(PendingScroll { at, offset }));
            if let Some(driver) = self.driver.borrow().as_ref() {
                driver.request();
            }
        }
    }

    /// Compatibility helper for callers without a platform timestamp.
    pub fn update(&self, offset: f32) {
        self.update_at(TimePoint::ZERO, offset);
        self.flush_frame();
    }

    pub fn flush_frame(&self) -> Option<ScrollSample> {
        let pending = self.pending.take()?;
        let range = self.range.get();
        let raw_progress = range.progress(pending.offset);
        let previous = *self.last.borrow();
        let (velocity, direction) = previous.map_or((0.0, ScrollDirection::Stationary), |last| {
            let seconds = (pending.at - last.at).as_nanos() as f32
                / arkit_animation_core::NANOS_PER_SECOND as f32;
            let velocity = if seconds <= f32::EPSILON {
                0.0
            } else {
                (pending.offset - last.offset) / seconds
            };
            let direction = if pending.offset > last.offset {
                ScrollDirection::Forward
            } else if pending.offset < last.offset {
                ScrollDirection::Backward
            } else {
                ScrollDirection::Stationary
            };
            (velocity, direction)
        });
        let in_view = range.contains(pending.offset);
        let progress = self.synchronized_progress(raw_progress);
        let sample = ScrollSample {
            at: pending.at,
            offset: pending.offset,
            progress,
            velocity,
            direction,
            in_view,
        };
        self.dispatch(previous, sample);
        self.drive(sample);
        *self.last.borrow_mut() = Some(sample);
        if self.once.get() && in_view {
            self.consumed.set(true);
        }
        Some(sample)
    }

    pub fn refresh(&self, range: ScrollRange) {
        self.range.set(range);
        self.controls.refresh();
        self.consumed.set(false);
    }

    pub fn revert(&self) {
        self.pending.set(None);
        *self.last.borrow_mut() = None;
        self.smoothed_progress.set(0.0);
        self.consumed.set(false);
        self.controls.revert();
    }

    pub fn is_in_view(&self) -> bool {
        self.last.borrow().is_some_and(|sample| sample.in_view)
    }

    fn attach_driver(self: &Rc<Self>, driver: Rc<FrameDriver>) {
        let weak = Rc::downgrade(self);
        let subscription = driver.subscribe(Rc::new(move |_| {
            if let Some(observer) = weak.upgrade() {
                observer.flush_frame();
            }
        }));
        *self.driver.borrow_mut() = Some(driver);
        *self.frame_source.borrow_mut() = Some(subscription);
    }

    fn synchronized_progress(&self, raw: f32) -> f32 {
        match &*self.sync.borrow() {
            ScrollSync::Method | ScrollSync::Progress => raw,
            ScrollSync::Eased(easing) => easing.sample(raw),
            ScrollSync::Smooth { factor, easing } => {
                let previous = self.smoothed_progress.get();
                let next = previous + (raw - previous) * factor.clamp(0.0, 1.0);
                let next = easing.sample(next.clamp(0.0, 1.0));
                self.smoothed_progress.set(next);
                next
            }
        }
    }

    fn drive(&self, sample: ScrollSample) {
        match &*self.sync.borrow() {
            ScrollSync::Method => {
                if sample.in_view {
                    self.controls.play();
                } else if self.repeat.get() {
                    self.controls.reverse();
                } else {
                    self.controls.pause();
                }
            }
            _ => {
                let nanos = (self.duration.get().as_nanos() as f64 * f64::from(sample.progress))
                    .round() as u64;
                self.controls.seek(TimePoint::from_nanos(nanos));
            }
        }
    }

    fn dispatch(&self, previous: Option<ScrollSample>, sample: ScrollSample) {
        let callbacks = self.callbacks.borrow().clone();
        if previous.is_none_or(|previous| !previous.in_view) && sample.in_view {
            invoke(&callbacks.enter, sample);
        }
        if previous.is_some_and(|previous| previous.in_view) && !sample.in_view {
            invoke(&callbacks.leave, sample);
        }
        match sample.direction {
            ScrollDirection::Forward => invoke(&callbacks.forward, sample),
            ScrollDirection::Backward => invoke(&callbacks.backward, sample),
            ScrollDirection::Stationary => {}
        }
        invoke(&callbacks.update, sample);
    }

    fn update_configuration(
        &self,
        range: ScrollRange,
        duration: TimeSpan,
        sync: ScrollSync,
        callbacks: ScrollCallbacks,
    ) {
        self.range.set(range);
        self.duration.set(duration);
        *self.sync.borrow_mut() = sync;
        *self.callbacks.borrow_mut() = callbacks;
    }
}

#[track_caller]
pub fn use_scroll_observer(
    controls: AnimationControls,
    range: ScrollRange,
    duration: TimeSpan,
    sync: ScrollSync,
    callbacks: ScrollCallbacks,
) -> Rc<ScrollObserver> {
    let driver = controls.inner.driver.clone();
    let initial_sync = sync.clone();
    let initial_callbacks = callbacks.clone();
    let observer = use_hook(|| {
        let observer = Rc::new(
            ScrollObserver::new(controls, range, duration)
                .sync(initial_sync)
                .callbacks(initial_callbacks),
        );
        observer.attach_driver(driver);
        observer
    });
    observer.update_configuration(range, duration, sync, callbacks);
    observer
}

fn invoke(callback: &Option<Rc<dyn Fn(ScrollSample)>>, sample: ScrollSample) {
    if let Some(callback) = callback {
        callback(sample);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn range_and_thresholds_are_stable() {
        assert_eq!(
            ScrollRange {
                start: 100.0,
                end: 300.0
            }
            .progress(200.0),
            0.5
        );
        assert_eq!(ScrollThreshold::Center.resolve(100.0, 200.0, 50.0), 175.0);
    }
}
