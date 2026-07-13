use std::cell::RefCell;
use std::collections::VecDeque;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;

use arkit_animation_core::{
    AnimationValue, Easing, Modifier, SpringSpec, TargetName, TimePoint, TimeSpan,
    TimelinePosition, Vec2,
};

use crate::properties::{TRANSLATE_X, TRANSLATE_Y};
use crate::{Animation, AnimationControls, AnimationSelector, Timeline};
use arkit_prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DragAxis {
    X,
    Y,
    Both,
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
}

#[derive(Clone)]
pub struct DraggableHandle {
    inner: Rc<RefCell<Draggable>>,
}

impl DraggableHandle {
    pub fn grab(&self, at: TimePoint, pointer: Vec2) -> bool {
        self.inner.borrow_mut().grab(at, pointer)
    }

    pub fn drag(&self, at: TimePoint, pointer: Vec2) -> Option<DragUpdate> {
        self.inner.borrow_mut().drag(at, pointer)
    }

    pub fn release(&self) -> Option<DragUpdate> {
        self.inner.borrow_mut().release()
    }

    pub fn refresh(&self) {
        self.inner.borrow().refresh();
    }

    pub fn reset(&self) {
        self.inner.borrow_mut().reset();
    }

    pub fn revert(&self) {
        self.inner.borrow_mut().revert();
    }

    pub fn stop(&self) {
        self.inner.borrow_mut().stop();
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
    use_hook(|| DraggableHandle {
        inner: Rc::new(RefCell::new(
            Draggable::new(controls)
                .target(target)
                .config(config)
                .callbacks(callbacks),
        )),
    })
}

impl Draggable {
    pub fn new(controls: AnimationControls) -> Self {
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
        self.controls.pause();
        self.phase = DragPhase::Grabbed;
        self.origin = self.position;
        self.pointer_origin = pointer;
        self.tracker.clear();
        self.tracker.push(at, pointer);
        self.emit(self.callbacks.grab.as_ref());
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
        self.position = self.constrain(
            self.axis_point(Vec2::new(self.origin.x + delta.x, self.origin.y + delta.y)),
        );
        self.map_position_to_animation();
        let update = self.snapshot();
        invoke(&self.callbacks.drag, update);
        invoke(&self.callbacks.update, update);
        Some(update)
    }

    pub fn release(&mut self) -> Option<DragUpdate> {
        if !matches!(self.phase, DragPhase::Grabbed | DragPhase::Dragging) {
            return None;
        }
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
        invoke(&self.callbacks.release, release);
        if target != self.position {
            invoke(&self.callbacks.snap, release);
        }
        self.start_release_animation(target);
        self.position = target;
        Some(release)
    }

    pub fn settle(&mut self) {
        self.phase = DragPhase::Idle;
        self.controls.complete();
        self.emit(self.callbacks.settle.as_ref());
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

    pub fn refresh(&self) {
        self.controls.refresh();
        if let Some(callback) = &self.callbacks.resize {
            callback();
        }
    }

    pub fn reset(&mut self) {
        self.controls.reset();
        self.position = Vec2::default();
        self.phase = DragPhase::Idle;
    }

    pub fn revert(&mut self) {
        self.controls.revert();
        self.position = Vec2::default();
        self.phase = DragPhase::Idle;
    }

    pub fn stop(&mut self) {
        self.controls.pause();
        self.phase = DragPhase::Idle;
        self.tracker.clear();
    }

    fn start_release_animation(&self, target_position: Vec2) {
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
        self.controls.restart();
    }

    fn map_position_to_animation(&self) {
        let Some(constraints) = self.config.constraints else {
            return;
        };
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

    fn emit(&self, callback: Option<&Rc<dyn Fn(DragUpdate)>>) {
        if let Some(callback) = callback {
            callback(self.snapshot());
        }
    }
}

fn invoke(callback: &Option<Rc<dyn Fn(DragUpdate)>>, update: DragUpdate) {
    if let Some(callback) = callback {
        callback(update);
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
