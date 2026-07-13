use std::cell::{Cell, RefCell};
use std::rc::Rc;

use arkit_animation_core::{Easing, TimePoint, TimeSpan};

use crate::AnimationControls;
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
    axis: ScrollAxis,
    range: Cell<ScrollRange>,
    duration: TimeSpan,
    sync: ScrollSync,
    callbacks: ScrollCallbacks,
    repeat: bool,
    once: bool,
    consumed: Cell<bool>,
    pending: Cell<Option<PendingScroll>>,
    last: RefCell<Option<ScrollSample>>,
    smoothed_progress: Cell<f32>,
}

impl ScrollObserver {
    pub fn new(controls: AnimationControls, range: ScrollRange, duration: TimeSpan) -> Self {
        Self {
            controls,
            axis: ScrollAxis::Vertical,
            range: Cell::new(range),
            duration,
            sync: ScrollSync::Progress,
            callbacks: ScrollCallbacks::default(),
            repeat: true,
            once: false,
            consumed: Cell::new(false),
            pending: Cell::new(None),
            last: RefCell::new(None),
            smoothed_progress: Cell::new(0.0),
        }
    }

    pub fn axis(mut self, axis: ScrollAxis) -> Self {
        self.axis = axis;
        self
    }

    pub fn sync(mut self, sync: ScrollSync) -> Self {
        self.sync = sync;
        self
    }

    pub fn callbacks(mut self, callbacks: ScrollCallbacks) -> Self {
        self.callbacks = callbacks;
        self
    }

    pub fn repeat(mut self, repeat: bool) -> Self {
        self.repeat = repeat;
        self
    }

    pub fn once(mut self, once: bool) -> Self {
        self.once = once;
        self
    }

    pub fn axis_kind(&self) -> ScrollAxis {
        self.axis
    }

    /// Stores only the latest event. Call [`Self::flush_frame`] from the root
    /// frame boundary to coalesce multiple platform events into one command.
    pub fn update_at(&self, at: TimePoint, offset: f32) {
        if !self.consumed.get() {
            self.pending.set(Some(PendingScroll { at, offset }));
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
        if self.once && in_view {
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

    fn synchronized_progress(&self, raw: f32) -> f32 {
        match &self.sync {
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
        match self.sync {
            ScrollSync::Method => {
                if sample.in_view {
                    self.controls.play();
                } else if self.repeat {
                    self.controls.reverse();
                } else {
                    self.controls.pause();
                }
            }
            _ => {
                let nanos =
                    (self.duration.as_nanos() as f64 * f64::from(sample.progress)).round() as u64;
                self.controls.seek(TimePoint::from_nanos(nanos));
            }
        }
    }

    fn dispatch(&self, previous: Option<ScrollSample>, sample: ScrollSample) {
        if previous.is_none_or(|previous| !previous.in_view) && sample.in_view {
            invoke(&self.callbacks.enter, sample);
        }
        if previous.is_some_and(|previous| previous.in_view) && !sample.in_view {
            invoke(&self.callbacks.leave, sample);
        }
        match sample.direction {
            ScrollDirection::Forward => invoke(&self.callbacks.forward, sample),
            ScrollDirection::Backward => invoke(&self.callbacks.backward, sample),
            ScrollDirection::Stationary => {}
        }
        invoke(&self.callbacks.update, sample);
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
    use_hook(|| {
        Rc::new(
            ScrollObserver::new(controls, range, duration)
                .sync(sync)
                .callbacks(callbacks),
        )
    })
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
