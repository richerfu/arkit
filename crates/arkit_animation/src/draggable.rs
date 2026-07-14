use std::cell::RefCell;
use std::collections::VecDeque;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;

use arkit_animation_core::{
    AnimationValue, Easing, Modifier, OutputSeek, PlaybackState, SpringSpec, TargetName, TimePoint,
    TimeSpan, TimelinePosition, Vec2,
};

use crate::controls::TimelineParts;
use crate::frame_driver::FrameSourceSubscription;
use crate::properties::{TRANSLATE_X, TRANSLATE_Y};
use crate::{Animation, AnimationControls, AnimationSelector, AnimationSubscription, Timeline};
use arkit_prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DragAxis {
    X,
    Y,
    Both,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum DragMapping {
    /// Seek translate-x and translate-y independently so the target remains
    /// under the pointer for unconstrained two-dimensional movement.
    DirectPosition,
    /// Map the selected axis (or the mean of both axes) to one timeline clock.
    #[default]
    TimelineProgress,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DragConstraints {
    pub min: Vec2,
    pub max: Vec2,
    pub padding: Vec2,
}

impl DragConstraints {
    pub fn new(min: Vec2, max: Vec2) -> Self {
        Self {
            min,
            max,
            padding: Vec2::default(),
        }
    }

    pub fn clamp(self, point: Vec2) -> Vec2 {
        let min_x = (self.min.x + self.padding.x).min(self.max.x - self.padding.x);
        let max_x = (self.min.x + self.padding.x).max(self.max.x - self.padding.x);
        let min_y = (self.min.y + self.padding.y).min(self.max.y - self.padding.y);
        let max_y = (self.min.y + self.padding.y).max(self.max.y - self.padding.y);
        Vec2::new(point.x.clamp(min_x, max_x), point.y.clamp(min_y, max_y))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct PointerSample {
    at: TimePoint,
    point: Vec2,
}

pub struct VelocityTracker {
    samples: VecDeque<PointerSample>,
    capacity: usize,
}

impl VelocityTracker {
    pub fn new(capacity: usize) -> Self {
        Self {
            samples: VecDeque::with_capacity(capacity.max(2)),
            capacity: capacity.max(2),
        }
    }

    pub fn clear(&mut self) {
        self.samples.clear();
    }

    pub fn push(&mut self, at: TimePoint, point: Vec2) {
        if self.samples.len() == self.capacity {
            self.samples.pop_front();
        }
        self.samples.push_back(PointerSample { at, point });
    }

    pub fn velocity(&self) -> Vec2 {
        let (Some(first), Some(last)) = (self.samples.front(), self.samples.back()) else {
            return Vec2::default();
        };
        let seconds =
            (last.at - first.at).as_nanos() as f32 / arkit_animation_core::NANOS_PER_SECOND as f32;
        if seconds <= f32::EPSILON {
            return Vec2::default();
        }
        Vec2::new(
            (last.point.x - first.point.x) / seconds,
            (last.point.y - first.point.y) / seconds,
        )
    }
}

#[derive(Clone, Default)]
pub enum DragSnap {
    #[default]
    None,
    Points(Rc<[Vec2]>),
    Grid(Vec2),
    Function(Rc<dyn Fn(Vec2) -> Vec2>),
}

impl Debug for DragSnap {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => formatter.write_str("None"),
            Self::Points(points) => formatter.debug_tuple("Points").field(points).finish(),
            Self::Grid(grid) => formatter.debug_tuple("Grid").field(grid).finish(),
            Self::Function(_) => formatter.write_str("Function(..)"),
        }
    }
}

impl DragSnap {
    fn apply(&self, point: Vec2) -> Vec2 {
        match self {
            Self::None => point,
            Self::Points(points) => points
                .iter()
                .copied()
                .min_by(|left, right| {
                    squared_distance(*left, point).total_cmp(&squared_distance(*right, point))
                })
                .unwrap_or(point),
            Self::Grid(step) => Vec2::new(snap_axis(point.x, step.x), snap_axis(point.y, step.y)),
            Self::Function(function) => function(point),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AutoScroll {
    pub viewport: DragConstraints,
    pub threshold: f32,
    pub max_speed: f32,
}

impl AutoScroll {
    fn velocity(self, point: Vec2) -> Vec2 {
        let axis = |value: f32, min: f32, max: f32| {
            if value < min + self.threshold {
                -self.max_speed * (1.0 - ((value - min) / self.threshold).clamp(0.0, 1.0))
            } else if value > max - self.threshold {
                self.max_speed * (1.0 - ((max - value) / self.threshold).clamp(0.0, 1.0))
            } else {
                0.0
            }
        };
        Vec2::new(
            axis(point.x, self.viewport.min.x, self.viewport.max.x),
            axis(point.y, self.viewport.min.y, self.viewport.max.y),
        )
    }
}

#[derive(Debug, Clone)]
pub struct DraggableConfig {
    pub axis: DragAxis,
    pub mapping: DragMapping,
    pub constraints: Option<DragConstraints>,
    pub threshold: f32,
    pub modifier: Modifier,
    pub snap: DragSnap,
    pub min_velocity: f32,
    pub max_velocity: f32,
    pub inertia: bool,
    pub release_duration: TimeSpan,
    pub map_duration: TimeSpan,
    pub spring: SpringSpec,
    pub container_friction: f32,
    pub release_friction: f32,
    pub auto_scroll: Option<AutoScroll>,
}

impl Default for DraggableConfig {
    fn default() -> Self {
        Self {
            axis: DragAxis::Both,
            mapping: DragMapping::TimelineProgress,
            constraints: None,
            threshold: 3.0,
            modifier: Modifier::Identity,
            snap: DragSnap::None,
            min_velocity: 0.0,
            max_velocity: 8_000.0,
            inertia: true,
            release_duration: TimeSpan::from_millis(420),
            map_duration: TimeSpan::from_millis(1_000),
            spring: SpringSpec::default(),
            container_friction: 0.55,
            release_friction: 0.18,
            auto_scroll: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DragPhase {
    Idle,
    Grabbed,
    Dragging,
    Settling,
    Disabled,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DragUpdate {
    pub phase: DragPhase,
    pub position: Vec2,
    pub velocity: Vec2,
    pub auto_scroll_velocity: Vec2,
}

#[derive(Default, Clone)]
pub struct DraggableCallbacks {
    pub grab: Option<Rc<dyn Fn(DragUpdate)>>,
    pub drag: Option<Rc<dyn Fn(DragUpdate)>>,
    pub update: Option<Rc<dyn Fn(DragUpdate)>>,
    pub release: Option<Rc<dyn Fn(DragUpdate)>>,
    pub snap: Option<Rc<dyn Fn(DragUpdate)>>,
    pub settle: Option<Rc<dyn Fn(DragUpdate)>>,
    pub resize: Option<Rc<dyn Fn()>>,
    /// Applies auto-scroll velocity to the owning scroll container.
    pub auto_scroll: Option<Rc<dyn Fn(Vec2)>>,
}

enum DragNotification {
    Update(Rc<dyn Fn(DragUpdate)>, DragUpdate),
    Simple(Rc<dyn Fn()>),
    AutoScroll(Rc<dyn Fn(Vec2)>, Vec2),
}

pub struct Draggable {
    controls: AnimationControls,
    target: Option<TargetName>,
    config: DraggableConfig,
    callbacks: DraggableCallbacks,
    phase: DragPhase,
    origin: Vec2,
    pointer_origin: Vec2,
    position: Vec2,
    tracker: VelocityTracker,
    pending_drag: Option<DragUpdate>,
    defer_to_frame: bool,
    release_active: bool,
    mapping_parts: TimelineParts,
    output_x: Option<OutputSeek>,
    output_y: Option<OutputSeek>,
    notifications: Vec<DragNotification>,
    release_from: Vec2,
    release_to: Vec2,
}

#[derive(Clone)]
pub struct DraggableHandle {
    inner: Rc<RefCell<Draggable>>,
    _frame_source: Rc<FrameSourceSubscription>,
    _settle_subscription: Rc<AnimationSubscription>,
}

impl DraggableHandle {
    pub fn grab(&self, at: TimePoint, pointer: Vec2) -> bool {
        drive_draggable(&self.inner, |draggable| draggable.grab(at, pointer))
    }

    pub fn drag(&self, at: TimePoint, pointer: Vec2) -> Option<DragUpdate> {
        drive_draggable(&self.inner, |draggable| draggable.drag(at, pointer))
    }

    pub fn release(&self) -> Option<DragUpdate> {
        drive_draggable(&self.inner, Draggable::release)
    }

    pub fn refresh(&self) {
        drive_draggable(&self.inner, |draggable| {
            draggable.refresh();
        });
    }

    pub fn reset(&self) {
        drive_draggable(&self.inner, |draggable| draggable.reset());
    }

    pub fn revert(&self) {
        drive_draggable(&self.inner, |draggable| draggable.revert());
    }

    pub fn stop(&self) {
        drive_draggable(&self.inner, |draggable| draggable.stop());
    }

    pub fn phase(&self) -> DragPhase {
        self.inner.borrow().phase()
    }
}

#[track_caller]
pub fn use_draggable(
    controls: AnimationControls,
    target: TargetName,
    config: DraggableConfig,
    callbacks: DraggableCallbacks,
) -> DraggableHandle {
    let driver = controls.inner.driver.clone();
    let observed_controls = controls.clone();
    let initial_target = target.clone();
    let initial_config = config.clone();
    let initial_callbacks = callbacks.clone();
    let handle = use_hook(move || {
        let inner = Rc::new(RefCell::new(
            Draggable::new(controls)
                .target(initial_target)
                .config(initial_config)
                .callbacks(initial_callbacks),
        ));
        inner.borrow_mut().defer_to_frame = true;
        let frame_inner = Rc::downgrade(&inner);
        let frame_source = Rc::new(driver.subscribe(Rc::new(move |_| {
            if let Some(inner) = frame_inner.upgrade() {
                drive_draggable(&inner, Draggable::flush_frame);
            }
        })));
        let settle_inner = Rc::downgrade(&inner);
        let settle_subscription = Rc::new(observed_controls.subscribe(move |snapshot| {
            if snapshot.state == PlaybackState::Completed {
                if let Some(inner) = settle_inner.upgrade() {
                    drive_draggable(&inner, |draggable| draggable.settle());
                }
            }
        }));
        DraggableHandle {
            inner,
            _frame_source: frame_source,
            _settle_subscription: settle_subscription,
        }
    });
    {
        let mut draggable = handle.inner.borrow_mut();
        draggable.target = Some(target);
        draggable.config = config;
        draggable.callbacks = callbacks;
    }
    handle
}

impl Draggable {
    pub fn new(controls: AnimationControls) -> Self {
        let mapping_parts = controls.inner.timeline_parts();
        Self {
            controls,
            target: None,
            config: DraggableConfig::default(),
            callbacks: DraggableCallbacks::default(),
            phase: DragPhase::Idle,
            origin: Vec2::default(),
            pointer_origin: Vec2::default(),
            position: Vec2::default(),
            tracker: VelocityTracker::new(8),
            pending_drag: None,
            defer_to_frame: false,
            release_active: false,
            mapping_parts,
            output_x: None,
            output_y: None,
            notifications: Vec::new(),
            release_from: Vec2::default(),
            release_to: Vec2::default(),
        }
    }

    pub fn target(mut self, target: TargetName) -> Self {
        self.target = Some(target);
        self
    }

    pub fn config(mut self, config: DraggableConfig) -> Self {
        self.config = config;
        self
    }

    pub fn callbacks(mut self, callbacks: DraggableCallbacks) -> Self {
        self.callbacks = callbacks;
        self
    }

    pub fn phase(&self) -> DragPhase {
        self.phase
    }

    pub fn position(&self) -> Vec2 {
        self.position
    }

    pub fn grab(&mut self, at: TimePoint, pointer: Vec2) -> bool {
        if self.phase == DragPhase::Disabled {
            return false;
        }
        if self.release_active {
            self.position = self.sample_release_position();
        }
        self.restore_mapping_timeline();
        self.controls.pause();
        self.phase = DragPhase::Grabbed;
        self.origin = self.position;
        self.pointer_origin = pointer;
        self.pending_drag = None;
        self.tracker.clear();
        self.tracker.push(at, pointer);
        self.emit(self.callbacks.grab.clone());
        true
    }

    pub fn drag(&mut self, at: TimePoint, pointer: Vec2) -> Option<DragUpdate> {
        if !matches!(self.phase, DragPhase::Grabbed | DragPhase::Dragging) {
            return None;
        }
        self.tracker.push(at, pointer);
        let delta = Vec2::new(
            pointer.x - self.pointer_origin.x,
            pointer.y - self.pointer_origin.y,
        );
        if self.phase == DragPhase::Grabbed
            && squared_distance(pointer, self.pointer_origin).sqrt() < self.config.threshold
        {
            return Some(self.snapshot());
        }
        self.phase = DragPhase::Dragging;
        self.position = self.constrain_drag(
            self.axis_point(Vec2::new(self.origin.x + delta.x, self.origin.y + delta.y)),
        );
        let update = self.snapshot();
        self.pending_drag = Some(update);
        if self.defer_to_frame {
            self.controls.inner.driver.request();
        } else {
            self.flush_frame();
        }
        Some(update)
    }

    pub fn flush_frame(&mut self) -> Option<DragUpdate> {
        self.flush_pending_drag(true)
    }

    fn flush_pending_drag(&mut self, map_to_animation: bool) -> Option<DragUpdate> {
        let update = self.pending_drag.take()?;
        if map_to_animation {
            self.map_position_to_animation();
        }
        self.emit_update(self.callbacks.drag.clone(), update);
        self.emit_update(self.callbacks.update.clone(), update);
        if update.auto_scroll_velocity != Vec2::default() {
            if let Some(callback) = self.callbacks.auto_scroll.clone() {
                self.notifications.push(DragNotification::AutoScroll(
                    callback,
                    update.auto_scroll_velocity,
                ));
            }
        }
        Some(update)
    }

    pub fn release(&mut self) -> Option<DragUpdate> {
        if !matches!(self.phase, DragPhase::Grabbed | DragPhase::Dragging) {
            return None;
        }
        // A release timeline starts at the latest logical position, so a raw
        // move that arrived after the last frame only needs its callbacks. A
        // stale seek must not be applied after the release plan replaces the
        // mapping plan.
        self.flush_pending_drag(false);
        let velocity = self.clamp_velocity(self.tracker.velocity());
        let projected = if self.config.inertia {
            Vec2::new(
                self.position.x + velocity.x * self.config.release_friction,
                self.position.y + velocity.y * self.config.release_friction,
            )
        } else {
            self.position
        };
        let target = self.constrain(self.config.snap.apply(projected));
        self.phase = DragPhase::Settling;
        let release = DragUpdate {
            phase: self.phase,
            position: self.position,
            velocity,
            auto_scroll_velocity: Vec2::default(),
        };
        self.emit_update(self.callbacks.release.clone(), release);
        if target != self.position {
            self.emit_update(self.callbacks.snap.clone(), release);
        }
        self.start_release_animation(target);
        self.position = target;
        Some(release)
    }

    pub fn settle(&mut self) {
        if self.phase != DragPhase::Settling {
            return;
        }
        self.phase = DragPhase::Idle;
        self.restore_mapping_timeline();
        self.emit(self.callbacks.settle.clone());
    }

    pub fn enable(&mut self) {
        if self.phase == DragPhase::Disabled {
            self.phase = DragPhase::Idle;
        }
    }

    pub fn disable(&mut self) {
        self.stop();
        self.phase = DragPhase::Disabled;
    }

    pub fn refresh(&mut self) {
        self.controls.refresh();
        if let Some(callback) = self.callbacks.resize.clone() {
            self.notifications.push(DragNotification::Simple(callback));
        }
    }

    pub fn reset(&mut self) {
        self.restore_mapping_timeline();
        self.controls.reset();
        self.position = Vec2::default();
        self.phase = DragPhase::Idle;
        self.pending_drag = None;
    }

    pub fn revert(&mut self) {
        self.restore_mapping_timeline();
        self.controls.revert();
        self.position = Vec2::default();
        self.phase = DragPhase::Idle;
        self.pending_drag = None;
    }

    pub fn stop(&mut self) {
        self.restore_mapping_timeline();
        self.controls.pause();
        self.phase = DragPhase::Idle;
        self.pending_drag = None;
        self.tracker.clear();
    }

    fn start_release_animation(&mut self, target_position: Vec2) {
        let Some(target) = self.target.clone() else {
            self.controls.play();
            return;
        };
        let animation = Animation::new(AnimationSelector::Target(target))
            .tween(
                &TRANSLATE_X,
                arkit_animation_core::Length::vp(self.position.x),
                arkit_animation_core::Length::vp(target_position.x),
                self.config.release_duration,
            )
            .configure_last(
                Easing::Spring(self.config.spring),
                Default::default(),
                Default::default(),
                TimeSpan::ZERO,
                0,
            )
            .tween(
                &TRANSLATE_Y,
                arkit_animation_core::Length::vp(self.position.y),
                arkit_animation_core::Length::vp(target_position.y),
                self.config.release_duration,
            )
            .configure_last(
                Easing::Spring(self.config.spring),
                Default::default(),
                Default::default(),
                TimeSpan::ZERO,
                0,
            );
        self.controls
            .set_timeline(Timeline::new().add(animation, TimelinePosition::START));
        self.release_from = self.position;
        self.release_to = target_position;
        self.release_active = true;
        self.controls.restart();
    }

    fn map_position_to_animation(&mut self) {
        let Some(constraints) = self.config.constraints else {
            return;
        };
        if self.config.mapping == DragMapping::DirectPosition
            && self.map_position_to_outputs(constraints)
        {
            return;
        }
        let extent_x = constraints.max.x - constraints.min.x;
        let extent_y = constraints.max.y - constraints.min.y;
        let progress = match self.config.axis {
            DragAxis::X => normalized(self.position.x, constraints.min.x, extent_x),
            DragAxis::Y => normalized(self.position.y, constraints.min.y, extent_y),
            DragAxis::Both => {
                (normalized(self.position.x, constraints.min.x, extent_x)
                    + normalized(self.position.y, constraints.min.y, extent_y))
                    * 0.5
            }
        };
        let duration = self.config.map_duration;
        let nanos = (duration.as_nanos() as f64 * f64::from(progress)).round() as u64;
        self.controls.seek(TimePoint::from_nanos(nanos));
    }

    fn map_position_to_outputs(&mut self, constraints: DragConstraints) -> bool {
        let Some(target) = self.target.as_ref() else {
            return false;
        };
        if self.output_x.is_none() {
            self.output_x = self.controls.inner.host.arkui().output_seek(
                target,
                TRANSLATE_X.name(),
                TimePoint::ZERO,
            );
        }
        if self.output_y.is_none() {
            self.output_y = self.controls.inner.host.arkui().output_seek(
                target,
                TRANSLATE_Y.name(),
                TimePoint::ZERO,
            );
        }
        let duration = self.config.map_duration;
        let position = |value: f32, min: f32, max: f32| {
            let progress = normalized(value, min, max - min);
            TimePoint::from_nanos((duration.as_nanos() as f64 * f64::from(progress)).round() as u64)
        };
        let mut x = self.output_x;
        let mut y = self.output_y;
        if let Some(output) = &mut x {
            output.position = position(self.position.x, constraints.min.x, constraints.max.x);
        }
        if let Some(output) = &mut y {
            output.position = position(self.position.y, constraints.min.y, constraints.max.y);
        }
        match self.config.axis {
            DragAxis::X => x.is_some_and(|x| {
                self.controls.inner.seek_outputs(x, None);
                true
            }),
            DragAxis::Y => y.is_some_and(|y| {
                self.controls.inner.seek_outputs(y, None);
                true
            }),
            DragAxis::Both => match (x, y) {
                (Some(x), Some(y)) => {
                    self.controls.inner.seek_outputs(x, Some(y));
                    true
                }
                _ => false,
            },
        }
    }

    fn restore_mapping_timeline(&mut self) {
        if !self.release_active {
            return;
        }
        self.controls
            .inner
            .replace_timeline_parts(&self.mapping_parts);
        self.release_active = false;
    }

    fn axis_point(&self, point: Vec2) -> Vec2 {
        match self.config.axis {
            DragAxis::X => Vec2::new(point.x, self.origin.y),
            DragAxis::Y => Vec2::new(self.origin.x, point.y),
            DragAxis::Both => point,
        }
    }

    fn constrain(&self, point: Vec2) -> Vec2 {
        let point = self
            .config
            .constraints
            .map_or(point, |bounds| bounds.clamp(point));
        Vec2::new(self.modify(point.x), self.modify(point.y))
    }

    fn constrain_drag(&self, point: Vec2) -> Vec2 {
        let point = self.config.constraints.map_or(point, |bounds| {
            let friction = self.config.container_friction.clamp(0.0, 1.0);
            Vec2::new(
                resist_axis(point.x, bounds.min.x, bounds.max.x, friction),
                resist_axis(point.y, bounds.min.y, bounds.max.y, friction),
            )
        });
        Vec2::new(self.modify(point.x), self.modify(point.y))
    }

    fn sample_release_position(&self) -> Vec2 {
        let Some(snapshot) = self.controls.snapshot() else {
            return self.position;
        };
        let duration = self.config.release_duration.as_nanos();
        if duration == 0 {
            return self.release_to;
        }
        let progress = snapshot.local_time.as_nanos() as f32 / duration as f32;
        let eased = Easing::Spring(self.config.spring).sample(progress.clamp(0.0, 1.0));
        Vec2::new(
            self.release_from.x + (self.release_to.x - self.release_from.x) * eased,
            self.release_from.y + (self.release_to.y - self.release_from.y) * eased,
        )
    }

    fn modify(&self, value: f32) -> f32 {
        match self.config.modifier.apply(AnimationValue::Scalar(value)) {
            Ok(AnimationValue::Scalar(value)) => value,
            _ => value,
        }
    }

    fn clamp_velocity(&self, velocity: Vec2) -> Vec2 {
        let clamp = |value: f32| {
            let magnitude = value.abs();
            if magnitude < self.config.min_velocity {
                0.0
            } else {
                value.clamp(-self.config.max_velocity, self.config.max_velocity)
            }
        };
        match self.config.axis {
            DragAxis::X => Vec2::new(clamp(velocity.x), 0.0),
            DragAxis::Y => Vec2::new(0.0, clamp(velocity.y)),
            DragAxis::Both => Vec2::new(clamp(velocity.x), clamp(velocity.y)),
        }
    }

    fn snapshot(&self) -> DragUpdate {
        DragUpdate {
            phase: self.phase,
            position: self.position,
            velocity: self.clamp_velocity(self.tracker.velocity()),
            auto_scroll_velocity: self
                .config
                .auto_scroll
                .map_or(Vec2::default(), |scroll| scroll.velocity(self.position)),
        }
    }

    fn emit(&mut self, callback: Option<Rc<dyn Fn(DragUpdate)>>) {
        if let Some(callback) = callback {
            let update = self.snapshot();
            self.notifications
                .push(DragNotification::Update(callback, update));
        }
    }

    fn emit_update(&mut self, callback: Option<Rc<dyn Fn(DragUpdate)>>, update: DragUpdate) {
        if let Some(callback) = callback {
            self.notifications
                .push(DragNotification::Update(callback, update));
        }
    }

    fn take_notifications(&mut self) -> Vec<DragNotification> {
        std::mem::take(&mut self.notifications)
    }
}

fn drive_draggable<R>(
    inner: &Rc<RefCell<Draggable>>,
    operation: impl FnOnce(&mut Draggable) -> R,
) -> R {
    let (result, notifications) = {
        let mut draggable = inner.borrow_mut();
        let result = operation(&mut draggable);
        let notifications = draggable.take_notifications();
        (result, notifications)
    };
    for notification in notifications {
        match notification {
            DragNotification::Update(callback, update) => callback(update),
            DragNotification::Simple(callback) => callback(),
            DragNotification::AutoScroll(callback, velocity) => callback(velocity),
        }
    }
    result
}

fn resist_axis(value: f32, min: f32, max: f32, friction: f32) -> f32 {
    if value < min {
        min + (value - min) * friction
    } else if value > max {
        max + (value - max) * friction
    } else {
        value
    }
}

fn normalized(value: f32, min: f32, extent: f32) -> f32 {
    if extent.abs() <= f32::EPSILON {
        0.0
    } else {
        ((value - min) / extent).clamp(0.0, 1.0)
    }
}

fn snap_axis(value: f32, step: f32) -> f32 {
    if step.abs() <= f32::EPSILON {
        value
    } else {
        (value / step).round() * step
    }
}

fn squared_distance(left: Vec2, right: Vec2) -> f32 {
    (left.x - right.x).powi(2) + (left.y - right.y).powi(2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn velocity_and_snap_are_deterministic_without_sleep() {
        let mut tracker = VelocityTracker::new(4);
        tracker.push(TimePoint::ZERO, Vec2::new(0.0, 0.0));
        tracker.push(TimePoint::from_nanos(100_000_000), Vec2::new(10.0, -5.0));
        assert_eq!(tracker.velocity(), Vec2::new(100.0, -50.0));
        assert_eq!(
            DragSnap::Grid(Vec2::new(10.0, 5.0)).apply(Vec2::new(16.0, 7.0)),
            Vec2::new(20.0, 5.0)
        );
    }
}
