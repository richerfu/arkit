use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use ohos_native_window_binding::NativeWindow;
use tokio::sync::mpsc::UnboundedSender;

use crate::renderer::LottieRenderer;
use crate::{
    LottieAlignment, LottieComposition, LottieError, LottieFit, LottieFrame, LottieRepeatMode,
    LottieSource, LottieStatus,
};

const FRAME_EVENT_INTERVAL: Duration = Duration::from_millis(100);

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PlayerConfiguration {
    pub(crate) source: LottieSource,
    pub(crate) active: bool,
    pub(crate) playing: bool,
    pub(crate) repeat: LottieRepeatMode,
    pub(crate) speed: f32,
    pub(crate) fit: LottieFit,
    pub(crate) alignment: LottieAlignment,
    pub(crate) quality: u8,
    pub(crate) max_frames_per_second: u16,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum PlaybackCommand {
    Play,
    Pause,
    Stop,
    SeekProgress(f32),
    SeekFrame(f32),
    Speed(f32),
    Repeat(LottieRepeatMode),
}

pub(crate) enum WorkerMessage {
    Configure(PlayerConfiguration),
    SourceLoaded {
        key: Arc<str>,
        result: crate::LottieResult<Arc<[u8]>>,
    },
    Playback(PlaybackCommand),
    SurfaceAvailable(NativeWindow),
    SurfaceLost,
    Tick(Instant),
    Shutdown,
}

#[derive(Debug, Clone)]
pub(crate) enum UiEvent {
    Status(LottieStatus),
    Composition(LottieComposition),
    Frame(LottieFrame),
    Completed,
    Error(LottieError),
}

pub(crate) struct WorkerHandle {
    sender: Sender<WorkerMessage>,
    tick_pending: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl WorkerHandle {
    pub(crate) fn spawn(events: UnboundedSender<UiEvent>) -> crate::LottieResult<Self> {
        let (sender, receiver) = mpsc::channel();
        let tick_pending = Arc::new(AtomicBool::new(false));
        let worker_tick_pending = tick_pending.clone();
        let thread = thread::Builder::new()
            .name("arkit-lottie".into())
            .spawn(move || run_worker(receiver, events, worker_tick_pending))
            .map_err(|error| {
                LottieError::worker_stopped("LottieWorker::spawn")
                    .with_worker_message(error.to_string())
            })?;
        Ok(Self {
            sender,
            tick_pending,
            thread: Some(thread),
        })
    }

    pub(crate) fn sender(&self) -> Sender<WorkerMessage> {
        self.sender.clone()
    }

    pub(crate) fn tick_pending(&self) -> Arc<AtomicBool> {
        self.tick_pending.clone()
    }

    pub(crate) fn send(&self, message: WorkerMessage) -> crate::LottieResult<()> {
        self.sender
            .send(message)
            .map_err(|_| LottieError::worker_stopped("LottieWorker::send"))
    }
}

impl Drop for WorkerHandle {
    fn drop(&mut self) {
        let _ = self.sender.send(WorkerMessage::Shutdown);
        if let Some(thread) = self.thread.take() {
            if thread.join().is_err() {
                ohos_hilog_binding::error("arkit_lottie: render worker panicked");
            }
        }
    }
}

struct WorkerState {
    active: bool,
    playing: bool,
    configured_playing: Option<bool>,
    repeat: LottieRepeatMode,
    configured_repeat: Option<LottieRepeatMode>,
    speed: f32,
    configured_speed: Option<f32>,
    fit: LottieFit,
    alignment: LottieAlignment,
    quality: u8,
    max_frames_per_second: u16,
    source: Option<LottieSource>,
    composition: Option<LottieComposition>,
    frame: f32,
    bounce_direction: f32,
    surface: Option<NativeWindow>,
    dirty: bool,
    render_blocked: bool,
    completed: bool,
    last_tick: Option<Instant>,
    last_render: Option<Instant>,
    last_frame_event: Option<Instant>,
    last_status: LottieStatus,
}

impl Default for WorkerState {
    fn default() -> Self {
        Self {
            active: false,
            playing: true,
            configured_playing: None,
            repeat: LottieRepeatMode::Loop,
            configured_repeat: None,
            speed: 1.0,
            configured_speed: None,
            fit: LottieFit::Contain,
            alignment: LottieAlignment::Center,
            quality: 50,
            max_frames_per_second: 60,
            source: None,
            composition: None,
            frame: 0.0,
            bounce_direction: 1.0,
            surface: None,
            dirty: false,
            render_blocked: false,
            completed: false,
            last_tick: None,
            last_render: None,
            last_frame_event: None,
            last_status: LottieStatus::Idle,
        }
    }
}

fn run_worker(
    receiver: Receiver<WorkerMessage>,
    events: UnboundedSender<UiEvent>,
    tick_pending: Arc<AtomicBool>,
) {
    let threads = render_thread_count();
    let engine = match thorvg::Thorvg::init(threads) {
        Ok(engine) => engine,
        Err(error) => {
            emit_error(
                &events,
                LottieError::render("Thorvg::init", error.to_string()),
            );
            return;
        }
    };
    let mut renderer = match LottieRenderer::new(&engine) {
        Ok(renderer) => renderer,
        Err(error) => {
            emit_error(&events, error);
            return;
        }
    };
    let mut state = WorkerState::default();
    set_status(&mut state, &events, LottieStatus::WaitingForSurface);

    while let Ok(message) = receiver.recv() {
        match message {
            WorkerMessage::Configure(configuration) => {
                configure(&mut renderer, &mut state, configuration, &events);
                render_dirty(&mut renderer, &mut state, &events, Instant::now());
            }
            WorkerMessage::SourceLoaded { key, result } => {
                source_loaded(&mut renderer, &mut state, &events, key, result);
                render_dirty(&mut renderer, &mut state, &events, Instant::now());
            }
            WorkerMessage::Playback(command) => {
                playback(&mut renderer, &mut state, command, &events);
                render_dirty(&mut renderer, &mut state, &events, Instant::now());
            }
            WorkerMessage::SurfaceAvailable(surface) => {
                state.surface = Some(surface);
                state.render_blocked = false;
                state.dirty = true;
                state.last_tick = None;
                state.last_render = None;
                render_dirty(&mut renderer, &mut state, &events, Instant::now());
                reconcile_status(&mut state, &events);
            }
            WorkerMessage::SurfaceLost => {
                state.surface = None;
                state.last_tick = None;
                state.last_render = None;
                set_status(&mut state, &events, LottieStatus::WaitingForSurface);
            }
            WorkerMessage::Tick(now) => {
                // Clear before rendering so at most one new tick can queue while
                // a slow frame is in flight.
                tick_pending.store(false, Ordering::Release);
                tick(&mut renderer, &mut state, &events, now);
            }
            WorkerMessage::Shutdown => break,
        }
    }
}

fn configure(
    renderer: &mut LottieRenderer<'_>,
    state: &mut WorkerState,
    configuration: PlayerConfiguration,
    events: &UnboundedSender<UiEvent>,
) {
    let source_changed = state.source.as_ref() != Some(&configuration.source);
    let layout_changed =
        state.fit != configuration.fit || state.alignment != configuration.alignment;
    let quality_changed = state.quality != configuration.quality.min(100);
    let activity_changed = state.active != configuration.active;
    let playback_changed = state.configured_playing != Some(configuration.playing);
    let repeat_changed = state.configured_repeat != Some(configuration.repeat);
    let speed_changed = state.configured_speed != Some(configuration.speed);

    state.active = configuration.active;
    if playback_changed {
        state.configured_playing = Some(configuration.playing);
        state.playing = configuration.playing;
        if state.playing {
            restart_at_terminal_boundary(state);
        }
    }
    if repeat_changed {
        state.configured_repeat = Some(configuration.repeat);
        state.repeat = configuration.repeat;
    }
    if speed_changed {
        state.configured_speed = Some(configuration.speed);
        state.speed = validated_speed(configuration.speed, events);
    }
    state.fit = configuration.fit;
    state.alignment = configuration.alignment;
    state.quality = configuration.quality.min(100);
    state.max_frames_per_second = configuration.max_frames_per_second.clamp(1, 120);
    if activity_changed || playback_changed {
        state.last_tick = None;
    }
    if activity_changed && state.active {
        state.dirty = true;
    }

    if source_changed {
        state.source = Some(configuration.source.clone());
        state.composition = None;
        state.frame = 0.0;
        state.bounce_direction = 1.0;
        state.dirty = false;
        state.render_blocked = false;
        state.completed = false;
        state.playing = state.configured_playing.unwrap_or(true);
        set_status(state, events, LottieStatus::Loading);
        if let Err(error) = renderer.unload() {
            block_render(state, events, error);
            return;
        }
        if !configuration.source.is_network()
            && !load_composition(renderer, state, &configuration.source, events)
        {
            return;
        }
    } else {
        if layout_changed {
            renderer.configure_layout(state.fit, state.alignment);
            state.dirty = true;
        }
        if quality_changed {
            if let Err(error) = renderer.set_quality(state.quality) {
                block_render(state, events, error);
                return;
            }
            state.dirty = true;
        }
    }
    reconcile_status(state, events);
}

fn source_loaded(
    renderer: &mut LottieRenderer<'_>,
    state: &mut WorkerState,
    events: &UnboundedSender<UiEvent>,
    key: Arc<str>,
    result: crate::LottieResult<Arc<[u8]>>,
) {
    let is_current = state
        .source
        .as_ref()
        .is_some_and(|source| source.is_network() && source.key() == key.as_ref());
    if !is_current {
        return;
    }
    match result {
        Ok(bytes) => {
            let source = LottieSource::new(key, bytes);
            load_composition(renderer, state, &source, events);
        }
        Err(error) => block_render(state, events, error),
    }
}

fn load_composition(
    renderer: &mut LottieRenderer<'_>,
    state: &mut WorkerState,
    source: &LottieSource,
    events: &UnboundedSender<UiEvent>,
) -> bool {
    match renderer.load(source, state.quality, state.fit, state.alignment) {
        Ok(composition) => {
            state.composition = Some(composition);
            state.dirty = true;
            let _ = events.send(UiEvent::Composition(composition));
            set_status(state, events, LottieStatus::Ready);
            emit_frame(state, events, Instant::now(), true);
            reconcile_status(state, events);
            true
        }
        Err(error) => {
            block_render(state, events, error);
            false
        }
    }
}

fn playback(
    renderer: &mut LottieRenderer<'_>,
    state: &mut WorkerState,
    command: PlaybackCommand,
    events: &UnboundedSender<UiEvent>,
) {
    let Some(composition) = state.composition else {
        let error = LottieError::invalid_configuration(
            "LottieController",
            "a composition must be loaded before controlling playback",
        );
        let _ = events.send(UiEvent::Error(error));
        return;
    };
    match command {
        PlaybackCommand::Play => {
            state.playing = true;
            restart_at_terminal_boundary(state);
            state.last_tick = None;
        }
        PlaybackCommand::Pause => {
            state.playing = false;
            state.last_tick = None;
        }
        PlaybackCommand::Stop => {
            state.playing = false;
            state.frame = 0.0;
            state.bounce_direction = 1.0;
            state.completed = false;
            state.last_tick = None;
            state.dirty = true;
        }
        PlaybackCommand::SeekProgress(progress) => {
            if !progress.is_finite() {
                let error = LottieError::invalid_configuration(
                    "LottieController::seek",
                    "progress must be finite",
                );
                let _ = events.send(UiEvent::Error(error));
                return;
            }
            state.frame = progress.clamp(0.0, 1.0) * last_frame(composition);
            state.last_tick = None;
            state.dirty = true;
            state.completed = false;
        }
        PlaybackCommand::SeekFrame(frame) => {
            if !frame.is_finite() {
                let error = LottieError::invalid_configuration(
                    "LottieController::seek_frame",
                    "frame must be finite",
                );
                let _ = events.send(UiEvent::Error(error));
                return;
            }
            state.frame = frame.clamp(0.0, last_frame(composition));
            state.last_tick = None;
            state.dirty = true;
            state.completed = false;
        }
        PlaybackCommand::Speed(speed) => {
            state.speed = validated_speed(speed, events);
            state.last_tick = None;
        }
        PlaybackCommand::Repeat(repeat) => state.repeat = repeat,
    }
    if state.dirty {
        if let Err(error) = renderer.set_frame(state.frame) {
            block_render(state, events, error);
            return;
        }
        emit_frame(state, events, Instant::now(), true);
    }
    reconcile_status(state, events);
}

fn tick(
    renderer: &mut LottieRenderer<'_>,
    state: &mut WorkerState,
    events: &UnboundedSender<UiEvent>,
    now: Instant,
) {
    if !state.active || state.surface.is_none() || state.render_blocked {
        state.last_tick = None;
        return;
    }
    let Some(composition) = state.composition else {
        return;
    };
    if !state.playing {
        state.last_tick = None;
        render_dirty(renderer, state, events, now);
        return;
    }
    let delta = state
        .last_tick
        .replace(now)
        .map(|last| now.saturating_duration_since(last))
        .unwrap_or(Duration::ZERO);
    if delta.is_zero() {
        return;
    }
    let advance = composition.frames_per_second * delta.as_secs_f32() * state.speed;
    let outcome = advance_timeline(
        state.frame,
        advance,
        composition,
        state.repeat,
        state.bounce_direction,
    );
    state.frame = outcome.frame;
    state.bounce_direction = outcome.bounce_direction;
    state.dirty = true;
    if outcome.completed {
        state.playing = false;
        state.completed = true;
        set_status(state, events, LottieStatus::Completed);
        let _ = events.send(UiEvent::Completed);
    }

    let minimum_interval = Duration::from_secs_f64(1.0 / f64::from(state.max_frames_per_second));
    if state
        .last_render
        .is_some_and(|last| now.saturating_duration_since(last) < minimum_interval)
    {
        return;
    }
    render_dirty(renderer, state, events, now);
}

fn render_dirty(
    renderer: &mut LottieRenderer<'_>,
    state: &mut WorkerState,
    events: &UnboundedSender<UiEvent>,
    now: Instant,
) {
    if !state.dirty || !state.active || state.render_blocked {
        return;
    }
    let Some(surface) = state.surface.as_ref() else {
        set_status(state, events, LottieStatus::WaitingForSurface);
        return;
    };
    if let Err(error) = renderer.set_frame(state.frame) {
        block_render(state, events, error);
        return;
    }
    match renderer.render(surface) {
        Ok(()) => {
            state.dirty = false;
            state.last_render = Some(now);
            emit_frame(state, events, now, false);
            reconcile_status(state, events);
        }
        Err(error) => block_render(state, events, error),
    }
}

fn block_render(state: &mut WorkerState, events: &UnboundedSender<UiEvent>, error: LottieError) {
    state.render_blocked = true;
    set_status(state, events, LottieStatus::Error(error.clone()));
    let _ = events.send(UiEvent::Error(error));
}

fn reconcile_status(state: &mut WorkerState, events: &UnboundedSender<UiEvent>) {
    if state.composition.is_none() || state.render_blocked {
        return;
    }
    let next = if state.completed {
        LottieStatus::Completed
    } else if state.surface.is_none() {
        LottieStatus::WaitingForSurface
    } else if state.playing && state.active {
        LottieStatus::Playing
    } else {
        LottieStatus::Paused
    };
    set_status(state, events, next);
}

fn set_status(state: &mut WorkerState, events: &UnboundedSender<UiEvent>, status: LottieStatus) {
    if state.last_status != status {
        state.last_status = status.clone();
        let _ = events.send(UiEvent::Status(status));
    }
}

fn emit_frame(
    state: &mut WorkerState,
    events: &UnboundedSender<UiEvent>,
    now: Instant,
    force: bool,
) {
    if !force
        && state
            .last_frame_event
            .is_some_and(|last| now.saturating_duration_since(last) < FRAME_EVENT_INTERVAL)
    {
        return;
    }
    let Some(composition) = state.composition else {
        return;
    };
    state.last_frame_event = Some(now);
    let last = last_frame(composition).max(f32::EPSILON);
    let _ = events.send(UiEvent::Frame(LottieFrame {
        frame: state.frame,
        progress: (state.frame / last).clamp(0.0, 1.0),
        elapsed_seconds: state.frame / composition.frames_per_second,
    }));
}

fn emit_error(events: &UnboundedSender<UiEvent>, error: LottieError) {
    let _ = events.send(UiEvent::Status(LottieStatus::Error(error.clone())));
    let _ = events.send(UiEvent::Error(error));
}

fn validated_speed(speed: f32, events: &UnboundedSender<UiEvent>) -> f32 {
    if speed.is_finite() && speed != 0.0 {
        return speed.clamp(-16.0, 16.0);
    }
    let _ = events.send(UiEvent::Error(LottieError::invalid_configuration(
        "LottiePlayer::speed",
        "speed must be finite and non-zero; using 1.0",
    )));
    1.0
}

fn render_thread_count() -> u32 {
    std::thread::available_parallelism()
        .map(|count| count.get().saturating_sub(1).clamp(1, 4) as u32)
        .unwrap_or(1)
}

fn last_frame(composition: LottieComposition) -> f32 {
    (composition.frames - 1.0).max(0.0)
}

fn restart_at_terminal_boundary(state: &mut WorkerState) {
    let Some(composition) = state.composition else {
        state.completed = false;
        return;
    };
    let last = last_frame(composition);
    if state.speed >= 0.0 && state.frame >= last {
        state.frame = 0.0;
        state.dirty = true;
    } else if state.speed < 0.0 && state.frame <= 0.0 {
        state.frame = last;
        state.dirty = true;
    }
    state.completed = false;
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct TimelineOutcome {
    frame: f32,
    bounce_direction: f32,
    completed: bool,
}

fn advance_timeline(
    current: f32,
    advance: f32,
    composition: LottieComposition,
    repeat: LottieRepeatMode,
    bounce_direction: f32,
) -> TimelineOutcome {
    let last = last_frame(composition);
    if last <= 0.0 {
        return TimelineOutcome {
            frame: 0.0,
            bounce_direction,
            completed: repeat == LottieRepeatMode::None,
        };
    }
    match repeat {
        LottieRepeatMode::None => {
            let next = current + advance;
            TimelineOutcome {
                frame: next.clamp(0.0, last),
                bounce_direction,
                completed: next <= 0.0 || next >= last,
            }
        }
        LottieRepeatMode::Loop => TimelineOutcome {
            frame: (current + advance).rem_euclid(composition.frames),
            bounce_direction,
            completed: false,
        },
        LottieRepeatMode::Reverse => {
            let period = last * 2.0;
            let current_phase = if bounce_direction >= 0.0 {
                current
            } else {
                period - current
            };
            let phase = (current_phase + advance).rem_euclid(period);
            if phase <= last {
                TimelineOutcome {
                    frame: phase,
                    bounce_direction: 1.0,
                    completed: false,
                }
            } else {
                TimelineOutcome {
                    frame: period - phase,
                    bounce_direction: -1.0,
                    completed: false,
                }
            }
        }
    }
}

trait WorkerErrorContext {
    fn with_worker_message(self, message: String) -> Self;
}

impl WorkerErrorContext for LottieError {
    fn with_worker_message(self, message: String) -> Self {
        LottieError::new(self.kind(), self.operation(), message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn composition() -> LottieComposition {
        LottieComposition {
            width: 100.0,
            height: 100.0,
            frames: 60.0,
            duration_seconds: 2.0,
            frames_per_second: 30.0,
        }
    }

    #[test]
    fn loop_wraps_without_completing() {
        let outcome = advance_timeline(58.0, 3.0, composition(), LottieRepeatMode::Loop, 1.0);
        assert_eq!(outcome.frame, 1.0);
        assert!(!outcome.completed);
    }

    #[test]
    fn once_clamps_and_completes() {
        let outcome = advance_timeline(58.0, 3.0, composition(), LottieRepeatMode::None, 1.0);
        assert_eq!(outcome.frame, 59.0);
        assert!(outcome.completed);
    }

    #[test]
    fn reverse_reflects_at_the_end() {
        let outcome = advance_timeline(58.0, 3.0, composition(), LottieRepeatMode::Reverse, 1.0);
        assert_eq!(outcome.frame, 57.0);
        assert_eq!(outcome.bounce_direction, -1.0);
    }
}
