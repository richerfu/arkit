//! Frame-driven, composable animation timelines.

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

use arkit_prelude::*;

use crate::{apply_state, AnimationDelta, AnimationState};

/// Easing functions available to individual timeline segments.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Easing {
    Linear,
    EaseInQuad,
    EaseOutQuad,
    EaseInOutQuad,
    EaseInCubic,
    #[default]
    EaseOutCubic,
    EaseInOutCubic,
    EaseInQuart,
    EaseOutQuart,
    EaseInOutQuart,
    EaseInSine,
    EaseOutSine,
    EaseInOutSine,
    EaseInExpo,
    EaseOutExpo,
    EaseInOutExpo,
    EaseOutBack,
    EaseOutBounce,
    /// CSS-compatible cubic Bézier curve.
    CubicBezier {
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
    },
    /// Quantize progress into a fixed number of steps.
    Steps(u16),
    /// Damped spring response normalized to the segment duration.
    Spring {
        mass: f32,
        stiffness: f32,
        damping: f32,
    },
}

impl Easing {
    /// Sample the easing curve at a normalized `0.0..=1.0` position.
    pub fn sample(self, value: f32) -> f32 {
        let x = value.clamp(0.0, 1.0);
        match self {
            Self::Linear => x,
            Self::EaseInQuad => x * x,
            Self::EaseOutQuad => 1.0 - (1.0 - x) * (1.0 - x),
            Self::EaseInOutQuad if x < 0.5 => 2.0 * x * x,
            Self::EaseInOutQuad => 1.0 - (-2.0 * x + 2.0).powi(2) / 2.0,
            Self::EaseInCubic => x.powi(3),
            Self::EaseOutCubic => 1.0 - (1.0 - x).powi(3),
            Self::EaseInOutCubic if x < 0.5 => 4.0 * x.powi(3),
            Self::EaseInOutCubic => 1.0 - (-2.0 * x + 2.0).powi(3) / 2.0,
            Self::EaseInQuart => x.powi(4),
            Self::EaseOutQuart => 1.0 - (1.0 - x).powi(4),
            Self::EaseInOutQuart if x < 0.5 => 8.0 * x.powi(4),
            Self::EaseInOutQuart => 1.0 - (-2.0 * x + 2.0).powi(4) / 2.0,
            Self::EaseInSine => 1.0 - (x * std::f32::consts::FRAC_PI_2).cos(),
            Self::EaseOutSine => (x * std::f32::consts::FRAC_PI_2).sin(),
            Self::EaseInOutSine => -((std::f32::consts::PI * x).cos() - 1.0) / 2.0,
            Self::EaseInExpo if x == 0.0 => 0.0,
            Self::EaseInExpo => 2.0_f32.powf(10.0 * x - 10.0),
            Self::EaseOutExpo if x == 1.0 => 1.0,
            Self::EaseOutExpo => 1.0 - 2.0_f32.powf(-10.0 * x),
            Self::EaseInOutExpo if x == 0.0 || x == 1.0 => x,
            Self::EaseInOutExpo if x < 0.5 => 2.0_f32.powf(20.0 * x - 10.0) / 2.0,
            Self::EaseInOutExpo => (2.0 - 2.0_f32.powf(-20.0 * x + 10.0)) / 2.0,
            Self::EaseOutBack => {
                const C1: f32 = 1.70158;
                const C3: f32 = C1 + 1.0;
                1.0 + C3 * (x - 1.0).powi(3) + C1 * (x - 1.0).powi(2)
            }
            Self::EaseOutBounce => ease_out_bounce(x),
            Self::CubicBezier { x1, y1, x2, y2 } => cubic_bezier(x, x1, y1, x2, y2),
            Self::Steps(steps) => {
                let steps = steps.max(1) as f32;
                (x * steps).floor() / steps
            }
            Self::Spring {
                mass,
                stiffness,
                damping,
            } => spring(x, mass, stiffness, damping),
        }
    }
}

fn cubic_bezier(x: f32, x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    fn axis(value: f32, p1: f32, p2: f32) -> f32 {
        let inverse = 1.0 - value;
        3.0 * inverse * inverse * value * p1
            + 3.0 * inverse * value * value * p2
            + value * value * value
    }

    let mut lower = 0.0;
    let mut upper = 1.0;
    for _ in 0..14 {
        let midpoint = (lower + upper) / 2.0;
        if axis(midpoint, x1, x2) < x {
            lower = midpoint;
        } else {
            upper = midpoint;
        }
    }
    axis((lower + upper) / 2.0, y1, y2)
}

fn spring(x: f32, mass: f32, stiffness: f32, damping: f32) -> f32 {
    if x == 0.0 || x == 1.0 {
        return x;
    }
    let mass = mass.max(0.01);
    let stiffness = stiffness.max(0.01);
    let damping = damping.max(0.0);
    let angular = (stiffness / mass).sqrt();
    let decay = damping / (2.0 * mass);

    let response = |time: f32| {
        if decay < angular {
            let damped = (angular * angular - decay * decay).sqrt();
            1.0 - (-decay * time).exp()
                * ((damped * time).cos() + decay / damped * (damped * time).sin())
        } else {
            1.0 - (-angular * time).exp() * (1.0 + angular * time)
        }
    };
    let end = response(1.0);
    if end.abs() < f32::EPSILON {
        x
    } else {
        response(x) / end
    }
}

fn ease_out_bounce(mut x: f32) -> f32 {
    const N1: f32 = 7.5625;
    const D1: f32 = 2.75;
    if x < 1.0 / D1 {
        N1 * x * x
    } else if x < 2.0 / D1 {
        x -= 1.5 / D1;
        N1 * x * x + 0.75
    } else if x < 2.5 / D1 {
        x -= 2.25 / D1;
        N1 * x * x + 0.9375
    } else {
        x -= 2.625 / D1;
        N1 * x * x + 0.984375
    }
}

/// One visual state at an absolute position in a [`Timeline`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimelineKeyframe {
    pub at_ms: u32,
    pub state: AnimationState,
    /// Easing used while approaching this keyframe from the previous one.
    pub easing: Easing,
}

/// A reusable animation sequence composed of absolute or chained keyframes.
#[derive(Debug, Clone, PartialEq)]
pub struct Timeline {
    keyframes: Vec<TimelineKeyframe>,
    delay_ms: u32,
    loop_delay_ms: u32,
    iterations: i32,
    alternate: bool,
    reversed: bool,
    playback_rate: f32,
}

impl Timeline {
    pub fn new(initial: AnimationState) -> Self {
        Self {
            keyframes: vec![TimelineKeyframe {
                at_ms: 0,
                state: initial,
                easing: Easing::Linear,
            }],
            delay_ms: 0,
            loop_delay_ms: 0,
            iterations: 1,
            alternate: false,
            reversed: false,
            playback_rate: 1.0,
        }
    }

    /// Append a keyframe `duration_ms` after the current last keyframe.
    pub fn to(self, state: AnimationState, duration_ms: u32) -> Self {
        self.to_with(state, duration_ms, Easing::default())
    }

    /// Append a keyframe with a segment-specific easing curve.
    pub fn to_with(mut self, state: AnimationState, duration_ms: u32, easing: Easing) -> Self {
        let at_ms = self.duration_ms().saturating_add(duration_ms);
        self.keyframes.push(TimelineKeyframe {
            at_ms,
            state,
            easing,
        });
        self
    }

    /// Append a keyframe relative to the current final state.
    pub fn to_relative(self, delta: AnimationDelta, duration_ms: u32, easing: Easing) -> Self {
        let state = self
            .keyframes
            .last()
            .map_or_else(AnimationState::default, |frame| delta.apply(frame.state));
        self.to_with(state, duration_ms, easing)
    }

    /// Insert or replace a keyframe at an absolute timeline position.
    pub fn keyframe(mut self, at_ms: u32, state: AnimationState, easing: Easing) -> Self {
        if let Some(frame) = self.keyframes.iter_mut().find(|frame| frame.at_ms == at_ms) {
            *frame = TimelineKeyframe {
                at_ms,
                state,
                easing,
            };
        } else {
            self.keyframes.push(TimelineKeyframe {
                at_ms,
                state,
                easing,
            });
            self.keyframes.sort_by_key(|frame| frame.at_ms);
        }
        self
    }

    pub fn delay_ms(mut self, value: u32) -> Self {
        self.delay_ms = value;
        self
    }

    pub fn loop_delay_ms(mut self, value: u32) -> Self {
        self.loop_delay_ms = value;
        self
    }

    /// Set repeat count (`-1` for infinite playback).
    pub fn iterations(mut self, value: i32) -> Self {
        self.iterations = if value < 0 { -1 } else { value.max(1) };
        self
    }

    pub fn alternate(mut self, value: bool) -> Self {
        self.alternate = value;
        self
    }

    pub fn reversed(mut self, value: bool) -> Self {
        self.reversed = value;
        self
    }

    pub fn playback_rate(mut self, value: f32) -> Self {
        self.playback_rate = value.max(0.01);
        self
    }

    pub fn duration_ms(&self) -> u32 {
        self.keyframes.last().map_or(0, |frame| frame.at_ms)
    }

    pub fn delay(&self) -> u32 {
        self.delay_ms
    }

    pub fn loop_delay(&self) -> u32 {
        self.loop_delay_ms
    }

    /// Scale every keyframe position to a new total duration.
    pub fn stretch(mut self, duration_ms: u32) -> Self {
        let previous_duration = self.duration_ms();
        if previous_duration == 0 {
            return self;
        }
        let factor = duration_ms as f64 / previous_duration as f64;
        for frame in &mut self.keyframes {
            frame.at_ms = (frame.at_ms as f64 * factor).round() as u32;
        }
        self
    }

    pub fn keyframes(&self) -> &[TimelineKeyframe] {
        &self.keyframes
    }

    /// Sample at normalized progress (`0.0..=1.0`).
    pub fn sample(&self, progress: f32) -> AnimationState {
        self.sample_at(self.duration_ms() as f32 * progress.clamp(0.0, 1.0))
    }

    /// Sample the interpolated state at an absolute position.
    pub fn sample_at(&self, position_ms: f32) -> AnimationState {
        let Some(first) = self.keyframes.first() else {
            return AnimationState::default();
        };
        if position_ms <= first.at_ms as f32 {
            return first.state;
        }
        let Some(last) = self.keyframes.last() else {
            return first.state;
        };
        if position_ms >= last.at_ms as f32 {
            return last.state;
        }

        for pair in self.keyframes.windows(2) {
            let from = pair[0];
            let to = pair[1];
            if position_ms <= to.at_ms as f32 {
                let span = (to.at_ms - from.at_ms).max(1) as f32;
                let progress = to.easing.sample((position_ms - from.at_ms as f32) / span);
                return interpolate_state(from.state, to.state, progress);
            }
        }
        last.state
    }
}

fn interpolate_state(from: AnimationState, to: AnimationState, progress: f32) -> AnimationState {
    fn lerp(from: f32, to: f32, progress: f32) -> f32 {
        from + (to - from) * progress
    }

    fn optional_lerp(from: Option<f32>, to: Option<f32>, progress: f32) -> Option<f32> {
        match (from, to) {
            (Some(from), Some(to)) => Some(lerp(from, to, progress)),
            (Some(from), None) => Some(from),
            (None, Some(to)) if progress > 0.0 => Some(to),
            _ => None,
        }
    }

    fn color_channel(value: u32, shift: u32) -> f32 {
        ((value >> shift) & 0xff) as f32
    }

    fn lerp_color(from: u32, to: u32, progress: f32) -> u32 {
        [24, 16, 8, 0].into_iter().fold(0_u32, |color, shift| {
            let channel = lerp(
                color_channel(from, shift),
                color_channel(to, shift),
                progress,
            )
            .round()
            .clamp(0.0, 255.0) as u32;
            color | (channel << shift)
        })
    }

    fn optional_color(from: Option<u32>, to: Option<u32>, progress: f32) -> Option<u32> {
        match (from, to) {
            (Some(from), Some(to)) => Some(lerp_color(from, to, progress)),
            (Some(from), None) => Some(from),
            (None, Some(to)) if progress > 0.0 => Some(to),
            _ => None,
        }
    }

    AnimationState {
        opacity: lerp(from.opacity, to.opacity, progress).clamp(0.0, 1.0),
        translate_x: lerp(from.translate_x, to.translate_x, progress),
        translate_y: lerp(from.translate_y, to.translate_y, progress),
        scale_x: lerp(from.scale_x, to.scale_x, progress),
        scale_y: lerp(from.scale_y, to.scale_y, progress),
        rotation_degrees: lerp(from.rotation_degrees, to.rotation_degrees, progress),
        background_color: optional_color(from.background_color, to.background_color, progress),
        font_color: optional_color(from.font_color, to.font_color, progress),
        border_radius: optional_lerp(from.border_radius, to.border_radius, progress),
        blur: optional_lerp(from.blur, to.blur, progress),
        width: optional_lerp(from.width, to.width, progress),
        height: optional_lerp(from.height, to.height, progress),
    }
}

/// Current playback state for a [`TimelineControls`] instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlaybackState {
    #[default]
    Idle,
    Running,
    Paused,
    Finished,
    Cancelled,
}

struct Player {
    timeline: Timeline,
    position_ms: f32,
    direction: f32,
    completed_iterations: u32,
    delay_remaining_ms: f32,
    loop_delay_remaining_ms: f32,
    playback_rate: f32,
    state: PlaybackState,
    generation: u64,
    last_tick: Option<Instant>,
    on_update: Option<Rc<dyn Fn(f32)>>,
    on_begin: Option<Rc<dyn Fn()>>,
    on_loop: Option<Rc<dyn Fn(u32)>>,
    on_pause: Option<Rc<dyn Fn()>>,
    on_complete: Option<Rc<dyn Fn()>>,
}

impl Player {
    fn new(timeline: Timeline) -> Self {
        let mut player = Self {
            position_ms: 0.0,
            direction: 1.0,
            completed_iterations: 0,
            delay_remaining_ms: 0.0,
            loop_delay_remaining_ms: 0.0,
            playback_rate: timeline.playback_rate,
            state: PlaybackState::Idle,
            generation: 0,
            last_tick: None,
            on_update: None,
            on_begin: None,
            on_loop: None,
            on_pause: None,
            on_complete: None,
            timeline,
        };
        player.reset_position();
        player
    }

    fn reset_position(&mut self) {
        self.direction = if self.timeline.reversed { -1.0 } else { 1.0 };
        self.position_ms = if self.direction < 0.0 {
            self.timeline.duration_ms() as f32
        } else {
            0.0
        };
        self.completed_iterations = 0;
        self.delay_remaining_ms = self.timeline.delay_ms as f32;
        self.loop_delay_remaining_ms = 0.0;
        self.playback_rate = self.timeline.playback_rate;
        self.last_tick = None;
    }

    fn consume_wait(wait: &mut f32, elapsed_ms: &mut f32) -> bool {
        if *wait <= 0.0 {
            return false;
        }
        let consumed = (*elapsed_ms).min(*wait);
        *wait -= consumed;
        *elapsed_ms -= consumed;
        *elapsed_ms <= 0.0
    }

    fn advance(&mut self, elapsed_ms: f32) -> AdvanceResult {
        let mut elapsed_ms = elapsed_ms * self.playback_rate;
        if Self::consume_wait(&mut self.delay_remaining_ms, &mut elapsed_ms)
            || Self::consume_wait(&mut self.loop_delay_remaining_ms, &mut elapsed_ms)
        {
            return AdvanceResult::default();
        }

        let duration = self.timeline.duration_ms() as f32;
        if duration <= 0.0 {
            self.state = PlaybackState::Finished;
            return AdvanceResult {
                finished: true,
                loops: 0,
            };
        }

        self.position_ms += elapsed_ms * self.direction;
        let mut loops = 0;
        loop {
            let crossed_end = self.direction > 0.0 && self.position_ms >= duration;
            let crossed_start = self.direction < 0.0 && self.position_ms <= 0.0;
            if !crossed_end && !crossed_start {
                return AdvanceResult {
                    finished: false,
                    loops,
                };
            }

            let mut overflow = if crossed_end {
                self.position_ms - duration
            } else {
                -self.position_ms
            };
            self.completed_iterations = self.completed_iterations.saturating_add(1);
            if self.timeline.iterations >= 0
                && self.completed_iterations >= self.timeline.iterations as u32
            {
                self.position_ms = if crossed_end { duration } else { 0.0 };
                self.state = PlaybackState::Finished;
                return AdvanceResult {
                    finished: true,
                    loops,
                };
            }

            loops += 1;

            if self.timeline.alternate {
                self.direction = -self.direction;
                self.position_ms = if crossed_end { duration } else { 0.0 };
            } else {
                self.position_ms = if self.direction > 0.0 { 0.0 } else { duration };
            }

            let loop_delay = self.timeline.loop_delay_ms as f32;
            if loop_delay > 0.0 {
                if overflow <= loop_delay {
                    self.loop_delay_remaining_ms = loop_delay - overflow;
                    return AdvanceResult {
                        finished: false,
                        loops,
                    };
                }
                overflow -= loop_delay;
            }
            self.position_ms += overflow * self.direction;
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct AdvanceResult {
    finished: bool,
    loops: u32,
}

/// Imperative playback controls returned by [`use_timeline`].
#[derive(Clone)]
pub struct TimelineControls {
    player: Rc<RefCell<Player>>,
    node_ref: arkit_hooks::ArkNodeRef,
    status: Signal<PlaybackState>,
    progress: Signal<f32>,
}

impl TimelineControls {
    pub fn is_ready(&self) -> bool {
        self.node_ref.get().is_some()
    }

    pub fn status(&self) -> PlaybackState {
        (self.status)()
    }

    pub fn progress(&self) -> f32 {
        (self.progress)()
    }

    pub fn duration_ms(&self) -> u32 {
        self.player.borrow().timeline.duration_ms()
    }

    pub fn current_time_ms(&self) -> f32 {
        self.player.borrow().position_ms
    }

    /// Replace the sequence used by subsequent playback and reset to idle.
    pub fn set_timeline(&self, timeline: Timeline) {
        let state = {
            let mut player = self.player.borrow_mut();
            player.generation = player.generation.wrapping_add(1);
            player.timeline = timeline;
            player.reset_position();
            player.state = PlaybackState::Idle;
            player.timeline.sample_at(player.position_ms)
        };
        self.apply(state);
        self.set_status(PlaybackState::Idle);
        self.set_progress_from_position();
    }

    pub fn play(&self) {
        if self.node_ref.peek().is_none() {
            return;
        }
        let (state, generation, begin_callback) = {
            let mut player = self.player.borrow_mut();
            if player.state == PlaybackState::Running {
                return;
            }
            let should_begin = matches!(
                player.state,
                PlaybackState::Finished | PlaybackState::Cancelled
            ) || player.state == PlaybackState::Idle;
            if matches!(
                player.state,
                PlaybackState::Finished | PlaybackState::Cancelled
            ) {
                player.reset_position();
            }
            player.state = PlaybackState::Running;
            player.generation = player.generation.wrapping_add(1);
            player.last_tick = None;
            (
                player.timeline.sample_at(player.position_ms),
                player.generation,
                should_begin.then(|| player.on_begin.clone()).flatten(),
            )
        };
        self.apply(state);
        self.set_status(PlaybackState::Running);
        if let Some(callback) = begin_callback {
            callback();
        }
        self.schedule_frame(generation);
    }

    pub fn pause(&self) {
        let callback = {
            let mut player = self.player.borrow_mut();
            if player.state != PlaybackState::Running {
                None
            } else {
                player.state = PlaybackState::Paused;
                player.generation = player.generation.wrapping_add(1);
                player.last_tick = None;
                Some(player.on_pause.clone())
            }
        };
        if let Some(callback) = callback {
            self.set_status(PlaybackState::Paused);
            if let Some(callback) = callback {
                callback();
            }
        }
    }

    pub fn resume(&self) {
        self.play();
    }

    pub fn restart(&self) {
        self.reset();
        self.play();
    }

    pub fn reverse(&self) {
        {
            let mut player = self.player.borrow_mut();
            player.direction = -player.direction;
            player.last_tick = None;
            if player.state != PlaybackState::Running && player.state != PlaybackState::Paused {
                let duration = player.timeline.duration_ms() as f32;
                if player.direction < 0.0 && player.position_ms <= 0.0 {
                    player.position_ms = duration;
                } else if player.direction > 0.0 && player.position_ms >= duration {
                    player.position_ms = 0.0;
                }
                // Preserve the selected direction and boundary. `play()` only
                // resets Finished/Cancelled players, while Idle starts as-is.
                player.state = PlaybackState::Idle;
            }
        }
        if self.player.borrow().state != PlaybackState::Running {
            self.play();
        }
    }

    pub fn seek(&self, position_ms: f32) {
        let state = {
            let mut player = self.player.borrow_mut();
            let duration = player.timeline.duration_ms() as f32;
            player.position_ms = position_ms.clamp(0.0, duration);
            player.timeline.sample_at(player.position_ms)
        };
        self.apply(state);
        self.set_progress_from_position();
    }

    pub fn finish(&self) {
        let (state, callback) = {
            let mut player = self.player.borrow_mut();
            player.generation = player.generation.wrapping_add(1);
            player.position_ms = if player.direction >= 0.0 {
                player.timeline.duration_ms() as f32
            } else {
                0.0
            };
            player.state = PlaybackState::Finished;
            (
                player.timeline.sample_at(player.position_ms),
                player.on_complete.clone(),
            )
        };
        self.apply(state);
        self.set_status(PlaybackState::Finished);
        self.set_progress_from_position();
        if let Some(callback) = callback {
            callback();
        }
    }

    /// Alias matching Anime.js terminology.
    pub fn complete(&self) {
        self.finish();
    }

    pub fn cancel(&self) {
        {
            let mut player = self.player.borrow_mut();
            player.generation = player.generation.wrapping_add(1);
            player.state = PlaybackState::Cancelled;
            player.last_tick = None;
        }
        self.set_status(PlaybackState::Cancelled);
    }

    pub fn reset(&self) {
        let state = {
            let mut player = self.player.borrow_mut();
            player.generation = player.generation.wrapping_add(1);
            player.reset_position();
            player.state = PlaybackState::Idle;
            player.timeline.sample_at(player.position_ms)
        };
        self.apply(state);
        self.set_status(PlaybackState::Idle);
        self.set_progress_from_position();
    }

    /// Cancel playback and restore the first visual state.
    pub fn revert(&self) {
        self.reset();
    }

    /// Resize the timeline while preserving current normalized progress.
    pub fn stretch(&self, duration_ms: u32) {
        let (state, running, generation) = {
            let mut player = self.player.borrow_mut();
            let old_duration = player.timeline.duration_ms().max(1) as f32;
            let progress = (player.position_ms / old_duration).clamp(0.0, 1.0);
            player.timeline = player.timeline.clone().stretch(duration_ms);
            player.position_ms = duration_ms as f32 * progress;
            player.generation = player.generation.wrapping_add(1);
            player.last_tick = None;
            (
                player.timeline.sample_at(player.position_ms),
                player.state == PlaybackState::Running,
                player.generation,
            )
        };
        self.apply(state);
        self.set_progress_from_position();
        if running {
            self.schedule_frame(generation);
        }
    }

    pub fn set_playback_rate(&self, value: f32) {
        self.player.borrow_mut().playback_rate = value.max(0.01);
    }

    pub fn on_update(&self, callback: impl Fn(f32) + 'static) {
        self.player.borrow_mut().on_update = Some(Rc::new(callback));
    }

    pub fn on_begin(&self, callback: impl Fn() + 'static) {
        self.player.borrow_mut().on_begin = Some(Rc::new(callback));
    }

    /// Run after each completed non-terminal iteration. The callback receives
    /// the number of iterations completed so far.
    pub fn on_loop(&self, callback: impl Fn(u32) + 'static) {
        self.player.borrow_mut().on_loop = Some(Rc::new(callback));
    }

    pub fn on_pause(&self, callback: impl Fn() + 'static) {
        self.player.borrow_mut().on_pause = Some(Rc::new(callback));
    }

    pub fn on_complete(&self, callback: impl Fn() + 'static) {
        self.player.borrow_mut().on_complete = Some(Rc::new(callback));
    }

    fn schedule_frame(&self, generation: u64) {
        let Some(node) = self.node_ref.peek() else {
            return;
        };
        let controls = self.clone();
        let result = node.borrow().post_frame_callback(move |_, _| {
            controls.tick(generation);
        });
        if let Err(error) = result {
            ohos_hilog_binding::warn(format!(
                "arkit_animation: timeline post_frame_callback failed: {error:?}"
            ));
            self.cancel();
        }
    }

    fn tick(&self, generation: u64) {
        let (
            state,
            progress,
            advance,
            completed_iterations,
            update_callback,
            loop_callback,
            complete_callback,
        ) = {
            let mut player = self.player.borrow_mut();
            if player.generation != generation || player.state != PlaybackState::Running {
                return;
            }
            let now = Instant::now();
            let elapsed_ms = player
                .last_tick
                .replace(now)
                .map_or(0.0, |last| now.duration_since(last).as_secs_f32() * 1_000.0);
            let advance = player.advance(elapsed_ms);
            let duration = player.timeline.duration_ms().max(1) as f32;
            let progress = (player.position_ms / duration).clamp(0.0, 1.0);
            (
                player.timeline.sample_at(player.position_ms),
                progress,
                advance,
                player.completed_iterations,
                player.on_update.clone(),
                player.on_loop.clone(),
                advance
                    .finished
                    .then(|| player.on_complete.clone())
                    .flatten(),
            )
        };

        self.apply(state);
        let mut progress_signal = self.progress;
        progress_signal.set(progress);
        if let Some(callback) = update_callback {
            callback(progress);
        }
        if advance.loops > 0 {
            if let Some(callback) = loop_callback {
                let first = completed_iterations.saturating_sub(advance.loops) + 1;
                for iteration in first..=completed_iterations {
                    callback(iteration);
                }
            }
        }

        if advance.finished {
            self.set_status(PlaybackState::Finished);
            if let Some(callback) = complete_callback {
                callback();
            }
        } else {
            self.schedule_frame(generation);
        }
    }

    fn apply(&self, state: AnimationState) {
        if let Some(node) = self.node_ref.peek() {
            apply_state(&node.borrow(), state);
        }
    }

    fn set_status(&self, state: PlaybackState) {
        let mut status = self.status;
        status.set(state);
    }

    fn set_progress_from_position(&self) {
        let player = self.player.borrow();
        let duration = player.timeline.duration_ms().max(1) as f32;
        let mut progress = self.progress;
        progress.set((player.position_ms / duration).clamp(0.0, 1.0));
    }
}

/// Create a frame-driven timeline bound to the native node backing the current
/// Dioxus component scope.
#[must_use]
pub fn use_timeline(timeline: Timeline) -> TimelineControls {
    let node_ref = arkit_hooks::use_ark_node();
    let status = use_signal(PlaybackState::default);
    let progress = use_signal(|| 0.0_f32);
    let initial_timeline = timeline.clone();
    let player = use_hook(move || Rc::new(RefCell::new(Player::new(initial_timeline))));
    let player_on_drop = player.clone();
    use_drop(move || {
        let mut player = player_on_drop.borrow_mut();
        player.generation = player.generation.wrapping_add(1);
        player.state = PlaybackState::Cancelled;
    });

    if player.borrow().state == PlaybackState::Idle && player.borrow().timeline != timeline {
        player.borrow_mut().timeline = timeline;
        player.borrow_mut().reset_position();
    }

    TimelineControls {
        player,
        node_ref,
        status,
        progress,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state(x: f32) -> AnimationState {
        AnimationState::new().translate(x, 0.0)
    }

    #[test]
    fn timeline_interpolates_and_honors_segment_easing() {
        let timeline = Timeline::new(state(0.0))
            .to_with(state(100.0), 100, Easing::Linear)
            .to_with(state(200.0), 100, Easing::EaseInQuad);
        assert_eq!(timeline.duration_ms(), 200);
        assert_eq!(timeline.sample_at(50.0).translate_x, 50.0);
        assert_eq!(timeline.sample_at(150.0).translate_x, 125.0);
    }

    #[test]
    fn absolute_keyframes_are_sorted_and_replace_duplicates() {
        let timeline = Timeline::new(state(0.0))
            .keyframe(200, state(200.0), Easing::Linear)
            .keyframe(100, state(100.0), Easing::Linear)
            .keyframe(100, state(120.0), Easing::Linear);
        assert_eq!(timeline.keyframes()[1].state.translate_x, 120.0);
        assert_eq!(timeline.sample_at(150.0).translate_x, 160.0);
    }

    #[test]
    fn easing_endpoints_are_stable() {
        let easings = [
            Easing::Linear,
            Easing::EaseInOutCubic,
            Easing::EaseInOutExpo,
            Easing::EaseOutBack,
            Easing::EaseOutBounce,
            Easing::CubicBezier {
                x1: 0.42,
                y1: 0.0,
                x2: 0.58,
                y2: 1.0,
            },
            Easing::Spring {
                mass: 1.0,
                stiffness: 100.0,
                damping: 10.0,
            },
        ];
        for easing in easings {
            assert!((easing.sample(0.0) - 0.0).abs() < 0.0001);
            assert!((easing.sample(1.0) - 1.0).abs() < 0.0001);
        }
    }

    #[test]
    fn interpolates_optional_colors_and_layout_properties() {
        let timeline = Timeline::new(
            AnimationState::new()
                .background_color(0xff000000)
                .border_radius(4.0)
                .size(100.0, 40.0),
        )
        .to_with(
            AnimationState::new()
                .background_color(0xffffffff)
                .border_radius(20.0)
                .size(200.0, 80.0),
            100,
            Easing::Linear,
        );
        let middle = timeline.sample(0.5);
        assert_eq!(middle.background_color, Some(0xff808080));
        assert_eq!(middle.border_radius, Some(12.0));
        assert_eq!(middle.width, Some(150.0));
        assert_eq!(middle.height, Some(60.0));
    }

    #[test]
    fn player_reports_non_terminal_loops() {
        let timeline = Timeline::new(state(0.0))
            .to(state(100.0), 100)
            .iterations(3);
        let mut player = Player::new(timeline);
        let first = player.advance(250.0);
        assert!(!first.finished);
        assert_eq!(first.loops, 2);
        assert_eq!(player.completed_iterations, 2);
        assert_eq!(player.position_ms, 50.0);

        let final_tick = player.advance(50.0);
        assert!(final_tick.finished);
        assert_eq!(final_tick.loops, 0);
        assert_eq!(player.completed_iterations, 3);
    }

    #[test]
    fn loop_delay_holds_the_iteration_boundary() {
        let timeline = Timeline::new(state(0.0))
            .to(state(100.0), 100)
            .iterations(2)
            .loop_delay_ms(50);
        let mut player = Player::new(timeline);
        let boundary = player.advance(120.0);
        assert_eq!(boundary.loops, 1);
        assert_eq!(player.position_ms, 0.0);
        assert_eq!(player.loop_delay_remaining_ms, 30.0);
        player.advance(40.0);
        assert_eq!(player.position_ms, 10.0);
    }

    #[test]
    fn stretches_keyframe_positions() {
        let timeline = Timeline::new(state(0.0))
            .to(state(100.0), 100)
            .to(state(200.0), 100)
            .stretch(500);
        assert_eq!(timeline.duration_ms(), 500);
        assert_eq!(timeline.keyframes()[1].at_ms, 250);
        assert_eq!(timeline.sample_at(125.0).translate_x, 50.0);
    }

    #[test]
    fn appends_relative_keyframes() {
        let timeline = Timeline::new(
            AnimationState::new()
                .translate(10.0, 20.0)
                .uniform_scale(2.0)
                .rotate(15.0)
                .size(100.0, 40.0),
        )
        .to_relative(
            AnimationDelta::new()
                .translate_by(5.0, -10.0)
                .uniform_scale_by(0.5)
                .rotate_by(45.0)
                .resize_by(20.0, 10.0),
            100,
            Easing::Linear,
        );
        let final_state = timeline.sample(1.0);
        assert_eq!(final_state.translate_x, 15.0);
        assert_eq!(final_state.translate_y, 10.0);
        assert_eq!(final_state.scale_x, 1.0);
        assert_eq!(final_state.rotation_degrees, 60.0);
        assert_eq!(final_state.width, Some(120.0));
        assert_eq!(final_state.height, Some(50.0));
    }
}
