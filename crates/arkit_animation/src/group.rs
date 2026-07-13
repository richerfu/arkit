//! A single-clock, multi-target timeline for Dioxus component trees.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::rc::Rc;
use std::time::Instant;

use arkit_prelude::*;

use crate::{apply_state, AnimationState, PlaybackState, Timeline};

/// One animation track placed on a [`TimelineGroup`].
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineTrack {
    pub target: String,
    pub start_ms: u32,
    pub timeline: Timeline,
}

impl TimelineTrack {
    pub fn end_ms(&self) -> u32 {
        self.start_ms
            .saturating_add(self.timeline.delay())
            .saturating_add(self.timeline.duration_ms())
    }
}

/// Error returned while resolving a named timeline position.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimelineGroupError {
    UnknownLabel(String),
}

impl Display for TimelineGroupError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownLabel(label) => write!(formatter, "unknown timeline label: {label}"),
        }
    }
}

impl Error for TimelineGroupError {}

/// A reusable, single-clock sequence containing tracks for multiple Dioxus
/// component targets.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineGroup {
    tracks: Vec<TimelineTrack>,
    labels: BTreeMap<String, u32>,
    duration_ms: u32,
    previous_start_ms: u32,
    previous_end_ms: u32,
    delay_ms: u32,
    loop_delay_ms: u32,
    iterations: i32,
    alternate: bool,
    reversed: bool,
    playback_rate: f32,
}

impl TimelineGroup {
    pub fn new() -> Self {
        Self::default()
    }

    /// Append a track after the current end of the group.
    pub fn add(self, target: impl Into<String>, timeline: Timeline) -> Self {
        let start_ms = self.duration_ms;
        self.add_at(target, timeline, start_ms)
    }

    /// Add a track at an absolute group position.
    pub fn add_at(mut self, target: impl Into<String>, timeline: Timeline, start_ms: u32) -> Self {
        let track = TimelineTrack {
            target: target.into(),
            start_ms,
            timeline,
        };
        self.previous_start_ms = start_ms;
        self.previous_end_ms = track.end_ms();
        self.duration_ms = self.duration_ms.max(self.previous_end_ms);
        self.tracks.push(track);
        self
    }

    /// Add relative to the beginning of the previously added track.
    pub fn add_with_previous(
        self,
        target: impl Into<String>,
        timeline: Timeline,
        offset_ms: i32,
    ) -> Self {
        let start_ms = offset_position(self.previous_start_ms, offset_ms);
        self.add_at(target, timeline, start_ms)
    }

    /// Add relative to the end of the previously added track.
    pub fn add_after_previous(
        self,
        target: impl Into<String>,
        timeline: Timeline,
        offset_ms: i32,
    ) -> Self {
        let start_ms = offset_position(self.previous_end_ms, offset_ms);
        self.add_at(target, timeline, start_ms)
    }

    /// Define a label at the current group end.
    pub fn label(mut self, name: impl Into<String>) -> Self {
        self.labels.insert(name.into(), self.duration_ms);
        self
    }

    /// Define a label at an absolute group position.
    pub fn label_at(mut self, name: impl Into<String>, at_ms: u32) -> Self {
        self.labels.insert(name.into(), at_ms);
        self
    }

    /// Add relative to a previously defined label.
    pub fn add_at_label(
        self,
        target: impl Into<String>,
        timeline: Timeline,
        label: &str,
        offset_ms: i32,
    ) -> Result<Self, TimelineGroupError> {
        let Some(position) = self.labels.get(label).copied() else {
            return Err(TimelineGroupError::UnknownLabel(label.to_string()));
        };
        Ok(self.add_at(target, timeline, offset_position(position, offset_ms)))
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

    pub fn tracks(&self) -> &[TimelineTrack] {
        &self.tracks
    }

    pub fn duration_ms(&self) -> u32 {
        self.duration_ms
    }

    pub fn label_position(&self, name: &str) -> Option<u32> {
        self.labels.get(name).copied()
    }

    pub fn targets(&self) -> impl Iterator<Item = &str> {
        self.tracks
            .iter()
            .map(|track| track.target.as_str())
            .collect::<BTreeSet<_>>()
            .into_iter()
    }

    fn sample_targets(&self, position_ms: f32) -> Vec<(String, AnimationState)> {
        let mut targets = BTreeMap::<&str, Vec<&TimelineTrack>>::new();
        for track in &self.tracks {
            targets.entry(&track.target).or_default().push(track);
        }

        targets
            .into_iter()
            .filter_map(|(target, mut tracks)| {
                tracks.sort_by_key(|track| track.start_ms);
                let selected = tracks
                    .iter()
                    .rev()
                    .find(|track| position_ms >= track.start_ms as f32)
                    .copied()
                    .or_else(|| tracks.first().copied())?;
                let local_ms =
                    (position_ms - selected.start_ms as f32 - selected.timeline.delay() as f32)
                        .clamp(0.0, selected.timeline.duration_ms() as f32);
                Some((target.to_string(), selected.timeline.sample_at(local_ms)))
            })
            .collect()
    }
}

impl Default for TimelineGroup {
    fn default() -> Self {
        Self {
            tracks: Vec::new(),
            labels: BTreeMap::new(),
            duration_ms: 0,
            previous_start_ms: 0,
            previous_end_ms: 0,
            delay_ms: 0,
            loop_delay_ms: 0,
            iterations: 1,
            alternate: false,
            reversed: false,
            playback_rate: 1.0,
        }
    }
}

fn offset_position(position: u32, offset_ms: i32) -> u32 {
    if offset_ms >= 0 {
        position.saturating_add(offset_ms as u32)
    } else {
        position.saturating_sub(offset_ms.unsigned_abs())
    }
}

#[derive(Clone)]
struct TargetRegistry {
    targets: Rc<RefCell<BTreeMap<String, arkit_hooks::ArkNodeRef>>>,
    version: Signal<u64>,
}

impl TargetRegistry {
    fn register(&self, id: String, node_ref: arkit_hooks::ArkNodeRef) {
        let inserted = {
            let mut targets = self.targets.borrow_mut();
            if let std::collections::btree_map::Entry::Vacant(entry) = targets.entry(id) {
                entry.insert(node_ref);
                true
            } else {
                false
            }
        };
        if inserted {
            let mut version = self.version;
            version += 1;
        }
    }

    fn unregister(&self, id: &str) {
        if self.targets.borrow_mut().remove(id).is_some() {
            let mut version = self.version;
            version += 1;
        }
    }
}

/// Handle returned by [`use_animation_target`].
#[derive(Clone)]
pub struct AnimationTarget {
    id: String,
    node_ref: arkit_hooks::ArkNodeRef,
}

impl AnimationTarget {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn is_ready(&self) -> bool {
        self.node_ref.get().is_some()
    }
}

/// Register the native node backing the current Dioxus component under a
/// stable target id in the nearest [`use_timeline_group`] scope.
#[must_use]
pub fn use_animation_target(id: impl Into<String>) -> AnimationTarget {
    let registry = use_context::<TargetRegistry>();
    let node_ref = arkit_hooks::use_ark_node();
    let id = id.into();
    let stable_id = use_hook(move || id);
    registry.register(stable_id.clone(), node_ref);

    let registry_on_drop = registry.clone();
    let id_on_drop = stable_id.clone();
    use_drop(move || registry_on_drop.unregister(&id_on_drop));

    AnimationTarget {
        id: stable_id,
        node_ref,
    }
}

struct GroupPlayer {
    group: TimelineGroup,
    position_ms: f32,
    direction: f32,
    completed_iterations: u32,
    delay_remaining_ms: f32,
    loop_delay_remaining_ms: f32,
    playback_rate: f32,
    state: PlaybackState,
    generation: u64,
    last_tick: Option<Instant>,
    on_begin: Option<Rc<dyn Fn()>>,
    on_update: Option<Rc<dyn Fn(f32)>>,
    on_loop: Option<Rc<dyn Fn(u32)>>,
    on_pause: Option<Rc<dyn Fn()>>,
    on_complete: Option<Rc<dyn Fn()>>,
}

impl GroupPlayer {
    fn new(group: TimelineGroup) -> Self {
        let mut player = Self {
            position_ms: 0.0,
            direction: 1.0,
            completed_iterations: 0,
            delay_remaining_ms: 0.0,
            loop_delay_remaining_ms: 0.0,
            playback_rate: group.playback_rate,
            state: PlaybackState::Idle,
            generation: 0,
            last_tick: None,
            on_begin: None,
            on_update: None,
            on_loop: None,
            on_pause: None,
            on_complete: None,
            group,
        };
        player.reset_position();
        player
    }

    fn reset_position(&mut self) {
        self.direction = if self.group.reversed { -1.0 } else { 1.0 };
        self.position_ms = if self.direction < 0.0 {
            self.group.duration_ms as f32
        } else {
            0.0
        };
        self.completed_iterations = 0;
        self.delay_remaining_ms = self.group.delay_ms as f32;
        self.loop_delay_remaining_ms = 0.0;
        self.playback_rate = self.group.playback_rate;
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

    fn advance(&mut self, elapsed_ms: f32) -> GroupAdvance {
        let mut elapsed_ms = elapsed_ms * self.playback_rate;
        if Self::consume_wait(&mut self.delay_remaining_ms, &mut elapsed_ms)
            || Self::consume_wait(&mut self.loop_delay_remaining_ms, &mut elapsed_ms)
        {
            return GroupAdvance::default();
        }

        let duration = self.group.duration_ms as f32;
        if duration <= 0.0 {
            self.state = PlaybackState::Finished;
            return GroupAdvance {
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
                return GroupAdvance {
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
            if self.group.iterations >= 0
                && self.completed_iterations >= self.group.iterations as u32
            {
                self.position_ms = if crossed_end { duration } else { 0.0 };
                self.state = PlaybackState::Finished;
                return GroupAdvance {
                    finished: true,
                    loops,
                };
            }
            loops += 1;

            if self.group.alternate {
                self.direction = -self.direction;
                self.position_ms = if crossed_end { duration } else { 0.0 };
            } else {
                self.position_ms = if self.direction > 0.0 { 0.0 } else { duration };
            }

            let loop_delay = self.group.loop_delay_ms as f32;
            if loop_delay > 0.0 {
                if overflow <= loop_delay {
                    self.loop_delay_remaining_ms = loop_delay - overflow;
                    return GroupAdvance {
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
struct GroupAdvance {
    finished: bool,
    loops: u32,
}

/// Unified playback controls for every target in a [`TimelineGroup`].
#[derive(Clone)]
pub struct TimelineGroupControls {
    player: Rc<RefCell<GroupPlayer>>,
    registry: TargetRegistry,
    driver_ref: arkit_hooks::ArkNodeRef,
    status: Signal<PlaybackState>,
    progress: Signal<f32>,
}

impl TimelineGroupControls {
    pub fn is_ready(&self) -> bool {
        let _ = (self.registry.version)();
        if self.driver_ref.get().is_none() {
            return false;
        }
        let player = self.player.borrow();
        let targets = self.registry.targets.borrow();
        let ready = player
            .group
            .targets()
            .all(|id| targets.get(id).is_some_and(|node| node.get().is_some()));
        ready
    }

    pub fn status(&self) -> PlaybackState {
        (self.status)()
    }

    pub fn progress(&self) -> f32 {
        (self.progress)()
    }

    pub fn duration_ms(&self) -> u32 {
        self.player.borrow().group.duration_ms
    }

    pub fn current_time_ms(&self) -> f32 {
        self.player.borrow().position_ms
    }

    pub fn set_group(&self, group: TimelineGroup) {
        {
            let mut player = self.player.borrow_mut();
            player.generation = player.generation.wrapping_add(1);
            player.group = group;
            player.reset_position();
            player.state = PlaybackState::Idle;
        }
        self.apply_current();
        self.set_status(PlaybackState::Idle);
        self.set_progress_from_position();
    }

    pub fn play(&self) {
        if !self.is_ready() {
            return;
        }
        let (generation, begin_callback) = {
            let mut player = self.player.borrow_mut();
            if player.state == PlaybackState::Running {
                return;
            }
            let should_begin = matches!(
                player.state,
                PlaybackState::Idle | PlaybackState::Finished | PlaybackState::Cancelled
            );
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
                player.generation,
                should_begin.then(|| player.on_begin.clone()).flatten(),
            )
        };
        self.apply_current();
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
                return;
            }
            player.state = PlaybackState::Paused;
            player.generation = player.generation.wrapping_add(1);
            player.last_tick = None;
            player.on_pause.clone()
        };
        self.set_status(PlaybackState::Paused);
        if let Some(callback) = callback {
            callback();
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
                let duration = player.group.duration_ms as f32;
                if player.direction < 0.0 && player.position_ms <= 0.0 {
                    player.position_ms = duration;
                } else if player.direction > 0.0 && player.position_ms >= duration {
                    player.position_ms = 0.0;
                }
                player.state = PlaybackState::Idle;
            }
        }
        if self.player.borrow().state != PlaybackState::Running {
            self.play();
        }
    }

    pub fn seek(&self, position_ms: f32) {
        {
            let mut player = self.player.borrow_mut();
            player.position_ms = position_ms.clamp(0.0, player.group.duration_ms as f32);
            player.last_tick = None;
        }
        self.apply_current();
        self.set_progress_from_position();
    }

    pub fn finish(&self) {
        let callback = {
            let mut player = self.player.borrow_mut();
            player.generation = player.generation.wrapping_add(1);
            player.position_ms = if player.direction >= 0.0 {
                player.group.duration_ms as f32
            } else {
                0.0
            };
            player.state = PlaybackState::Finished;
            player.on_complete.clone()
        };
        self.apply_current();
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
        {
            let mut player = self.player.borrow_mut();
            player.generation = player.generation.wrapping_add(1);
            player.reset_position();
            player.state = PlaybackState::Idle;
        }
        self.apply_current();
        self.set_status(PlaybackState::Idle);
        self.set_progress_from_position();
    }

    /// Cancel playback and restore every target's first visual state.
    pub fn revert(&self) {
        self.reset();
    }

    pub fn set_playback_rate(&self, value: f32) {
        self.player.borrow_mut().playback_rate = value.max(0.01);
    }

    pub fn on_begin(&self, callback: impl Fn() + 'static) {
        self.player.borrow_mut().on_begin = Some(Rc::new(callback));
    }

    pub fn on_update(&self, callback: impl Fn(f32) + 'static) {
        self.player.borrow_mut().on_update = Some(Rc::new(callback));
    }

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
        let Some(node) = self.driver_ref.peek() else {
            return;
        };
        let controls = self.clone();
        let result = node.borrow().post_frame_callback(move |_, _| {
            controls.tick(generation);
        });
        if let Err(error) = result {
            ohos_hilog_binding::warn(format!(
                "arkit_animation: group post_frame_callback failed: {error:?}"
            ));
            self.cancel();
        }
    }

    fn tick(&self, generation: u64) {
        let (
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
            let duration = player.group.duration_ms.max(1) as f32;
            let progress = (player.position_ms / duration).clamp(0.0, 1.0);
            (
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

        self.apply_current();
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

    fn apply_current(&self) {
        let samples = {
            let player = self.player.borrow();
            player.group.sample_targets(player.position_ms)
        };
        let targets = self.registry.targets.borrow();
        for (id, state) in samples {
            if let Some(node) = targets.get(&id).and_then(|node| node.peek()) {
                apply_state(&node.borrow(), state);
            }
        }
    }

    fn set_status(&self, state: PlaybackState) {
        let mut status = self.status;
        status.set(state);
    }

    fn set_progress_from_position(&self) {
        let player = self.player.borrow();
        let duration = player.group.duration_ms.max(1) as f32;
        let mut progress = self.progress;
        progress.set((player.position_ms / duration).clamp(0.0, 1.0));
    }
}

/// Create a shared multi-target animation scope driven by one frame clock.
/// Descendant components register their native roots with
/// [`use_animation_target`].
#[must_use]
pub fn use_timeline_group(group: TimelineGroup) -> TimelineGroupControls {
    let driver_ref = arkit_hooks::use_ark_node();
    let status = use_signal(PlaybackState::default);
    let progress = use_signal(|| 0.0_f32);
    let version = use_signal(|| 0_u64);
    let registry = use_hook(move || TargetRegistry {
        targets: Rc::new(RefCell::new(BTreeMap::new())),
        version,
    });
    use_context_provider({
        let registry = registry.clone();
        move || registry
    });

    let initial_group = group.clone();
    let player = use_hook(move || Rc::new(RefCell::new(GroupPlayer::new(initial_group))));
    let player_on_drop = player.clone();
    use_drop(move || {
        let mut player = player_on_drop.borrow_mut();
        player.generation = player.generation.wrapping_add(1);
        player.state = PlaybackState::Cancelled;
    });

    if player.borrow().state == PlaybackState::Idle && player.borrow().group != group {
        player.borrow_mut().group = group;
        player.borrow_mut().reset_position();
    }

    TimelineGroupControls {
        player,
        registry,
        driver_ref,
        status,
        progress,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Easing;

    fn state(x: f32) -> AnimationState {
        AnimationState::new().translate(x, 0.0)
    }

    #[test]
    fn positions_tracks_and_labels() {
        let track = Timeline::new(state(0.0)).to(state(100.0), 100);
        let group = TimelineGroup::new()
            .label_at("intro", 40)
            .add_at("a", track.clone(), 20)
            .add_with_previous("b", track.clone(), 30)
            .add_after_previous("c", track.clone(), -20)
            .add_at_label("d", track, "intro", 10)
            .unwrap();
        assert_eq!(group.tracks()[0].start_ms, 20);
        assert_eq!(group.tracks()[1].start_ms, 50);
        assert_eq!(group.tracks()[2].start_ms, 130);
        assert_eq!(group.tracks()[3].start_ms, 50);
        assert_eq!(group.duration_ms(), 230);
    }

    #[test]
    fn samples_multiple_targets_from_one_position() {
        let first = Timeline::new(state(0.0)).to_with(state(100.0), 100, Easing::Linear);
        let second = Timeline::new(state(200.0)).to_with(state(300.0), 100, Easing::Linear);
        let samples = TimelineGroup::new()
            .add_at("a", first, 0)
            .add_at("b", second, 50)
            .sample_targets(75.0)
            .into_iter()
            .collect::<BTreeMap<_, _>>();
        assert_eq!(samples["a"].translate_x, 75.0);
        assert_eq!(samples["b"].translate_x, 225.0);
    }

    #[test]
    fn loop_delay_holds_boundary_before_next_iteration() {
        let group = TimelineGroup::new()
            .add("a", Timeline::new(state(0.0)).to(state(100.0), 100))
            .iterations(2)
            .loop_delay_ms(50);
        let mut player = GroupPlayer::new(group);
        let boundary = player.advance(120.0);
        assert_eq!(boundary.loops, 1);
        assert_eq!(player.position_ms, 0.0);
        assert_eq!(player.loop_delay_remaining_ms, 30.0);
        let resumed = player.advance(40.0);
        assert!(!resumed.finished);
        assert_eq!(player.position_ms, 10.0);
    }
}
