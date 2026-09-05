use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender, TryRecvError};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use ohos_avplayer_binding::{AvPlayer, AvPlayerEvent, AvPlayerState, NativeWindow};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    VideoBuffering, VideoError, VideoMetadata, VideoProgress, VideoResizeMode, VideoSeekMode,
    VideoSnapshot, VideoSource, VideoStatus, VideoSubtitleCue, VideoSubtitleSource,
};

const WORKER_POLL_INTERVAL: Duration = Duration::from_millis(20);
const CONTROL_TICK_INTERVAL: Duration = Duration::from_millis(250);
const MIN_PROGRESS_INTERVAL: Duration = Duration::from_millis(50);
const MAX_PROGRESS_INTERVAL: Duration = Duration::from_secs(10);

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PlayerConfiguration {
    pub(crate) source: VideoSource,
    pub(crate) active: bool,
    pub(crate) autoplay: bool,
    pub(crate) looping: bool,
    pub(crate) muted: bool,
    pub(crate) volume: f32,
    pub(crate) playback_rate: f32,
    pub(crate) initial_position: Duration,
    pub(crate) resize_mode: VideoResizeMode,
    pub(crate) progress_interval: Duration,
    pub(crate) subtitles: Vec<VideoSubtitleSource>,
}

#[derive(Debug)]
pub(crate) enum PlaybackCommand {
    Play,
    Pause,
    Stop,
    Seek(Duration, VideoSeekMode),
    SeekBy(f64),
    Volume(f32),
    Muted(bool),
    Looping(bool),
    Fullscreen(bool),
    PlaybackRate(f32),
    SelectBitrate(u32),
    SelectTrack(i32),
    DeselectTrack(i32),
    ReplaceSource(VideoSource),
}

pub(crate) enum WorkerMessage {
    Configure(PlayerConfiguration),
    SurfaceAvailable {
        registration: u64,
        surface: NativeWindow,
    },
    SurfaceLost(u64),
    Playback(PlaybackCommand),
    Shutdown,
}

#[derive(Debug)]
pub(crate) enum UiEvent {
    Snapshot(VideoSnapshot),
    Status(VideoStatus),
    LoadStart,
    Loaded(VideoMetadata),
    Progress(VideoProgress),
    Buffering(VideoBuffering),
    SeekCompleted(Duration),
    PlaybackRateChanged(f32),
    VolumeChanged(f32),
    BitrateChanged(u32),
    AvailableBitrates(Vec<u32>),
    ReadyForDisplay,
    TracksChanged(Vec<crate::VideoTrack>),
    Subtitle(VideoSubtitleCue),
    AudioInterrupted,
    ControlTick,
    FullscreenChanged(bool),
    Ended,
    Error(VideoError),
}

pub(crate) struct WorkerHandle {
    sender: Sender<WorkerMessage>,
    thread: Option<JoinHandle<()>>,
}

impl WorkerHandle {
    pub(crate) fn spawn(events: UnboundedSender<UiEvent>) -> crate::VideoResult<Self> {
        let (sender, receiver) = mpsc::channel();
        let thread = thread::Builder::new()
            .name("arkit-video-player".into())
            .spawn(move || worker_main(receiver, events))
            .map_err(|error| {
                VideoError::new(
                    crate::VideoErrorKind::WorkerStopped,
                    "WorkerHandle::spawn",
                    error.to_string(),
                )
            })?;
        Ok(Self {
            sender,
            thread: Some(thread),
        })
    }

    pub(crate) fn sender(&self) -> Sender<WorkerMessage> {
        self.sender.clone()
    }

    pub(crate) fn send(&self, message: WorkerMessage) -> crate::VideoResult<()> {
        self.sender
            .send(message)
            .map_err(|_| VideoError::worker_stopped("WorkerHandle::send"))
    }
}

impl Drop for WorkerHandle {
    fn drop(&mut self) {
        let _ = self.sender.send(WorkerMessage::Shutdown);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

struct WorkerState {
    configuration: Option<PlayerConfiguration>,
    surface: Option<NativeWindow>,
    surface_registration: Option<u64>,
    player: Option<AvPlayer>,
    native_state: AvPlayerState,
    snapshot: VideoSnapshot,
    desired_playing: bool,
    ready: bool,
    buffering: bool,
    pending_initial_seek: bool,
    ready_for_display_emitted: bool,
    ended_emitted: bool,
    pending_resume_position: Option<Duration>,
    last_progress_event: Option<Instant>,
}

impl Default for WorkerState {
    fn default() -> Self {
        Self {
            configuration: None,
            surface: None,
            surface_registration: None,
            player: None,
            native_state: AvPlayerState::Idle,
            snapshot: VideoSnapshot::default(),
            desired_playing: false,
            ready: false,
            buffering: false,
            pending_initial_seek: false,
            ready_for_display_emitted: false,
            ended_emitted: false,
            pending_resume_position: None,
            last_progress_event: None,
        }
    }
}

fn worker_main(receiver: Receiver<WorkerMessage>, events: UnboundedSender<UiEvent>) {
    let (native_sender, native_receiver) = mpsc::channel();
    let mut state = WorkerState::default();
    let mut last_control_tick = Instant::now();
    loop {
        match receiver.recv_timeout(WORKER_POLL_INTERVAL) {
            Ok(WorkerMessage::Configure(configuration)) => {
                configure(&mut state, configuration, &native_sender, &events);
            }
            Ok(WorkerMessage::SurfaceAvailable {
                registration,
                surface,
            }) => {
                surface_available(&mut state, registration, surface, &native_sender, &events);
            }
            Ok(WorkerMessage::SurfaceLost(registration)) => {
                surface_lost(&mut state, registration, &events)
            }
            Ok(WorkerMessage::Playback(command)) => {
                playback(&mut state, command, &native_sender, &events)
            }
            Ok(WorkerMessage::Shutdown) | Err(RecvTimeoutError::Disconnected) => break,
            Err(RecvTimeoutError::Timeout) => {}
        }

        loop {
            match native_receiver.try_recv() {
                Ok(event) => handle_native_event(&mut state, event, &events),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => break,
            }
        }
        emit_polled_progress(&mut state, &events, Instant::now());
        if last_control_tick.elapsed() >= CONTROL_TICK_INTERVAL {
            last_control_tick = Instant::now();
            let _ = events.send(UiEvent::ControlTick);
        }
    }
    state.player.take();
    state.surface.take();
}

fn configure(
    state: &mut WorkerState,
    mut configuration: PlayerConfiguration,
    native_sender: &Sender<AvPlayerEvent>,
    events: &UnboundedSender<UiEvent>,
) {
    if let Err(error) = configuration.source.validate() {
        fail(state, events, error);
        return;
    }
    if !configuration.volume.is_finite() || !(0.0..=1.0).contains(&configuration.volume) {
        report_error(
            events,
            VideoError::invalid_configuration(
                "VideoPlayerProps::volume",
                "volume must be finite and within 0.0..=1.0",
            ),
        );
        configuration.volume = 1.0;
    }
    if !configuration.playback_rate.is_finite()
        || !(0.125..=4.0).contains(&configuration.playback_rate)
    {
        report_error(
            events,
            VideoError::invalid_configuration(
                "VideoPlayerProps::playback_rate",
                "playback rate must be finite and within 0.125..=4.0",
            ),
        );
        configuration.playback_rate = 1.0;
    }
    configuration.progress_interval = configuration
        .progress_interval
        .clamp(MIN_PROGRESS_INTERVAL, MAX_PROGRESS_INTERVAL);

    let previous = state.configuration.as_ref();
    let source_changed = previous.is_none_or(|old| old.source != configuration.source);
    let autoplay_changed = previous.is_some_and(|old| old.autoplay != configuration.autoplay);
    let active_changed = previous.is_some_and(|old| old.active != configuration.active);
    let resize_changed = previous.is_some_and(|old| old.resize_mode != configuration.resize_mode);
    let volume_changed = previous.is_some_and(|old| old.volume != configuration.volume);
    let muted_changed = previous.is_some_and(|old| old.muted != configuration.muted);
    let looping_changed = previous.is_some_and(|old| old.looping != configuration.looping);
    let rate_changed = previous.is_some_and(|old| old.playback_rate != configuration.playback_rate);
    let subtitles_changed = previous.is_some_and(|old| old.subtitles != configuration.subtitles);

    if source_changed || autoplay_changed {
        state.desired_playing = configuration.autoplay;
    }
    state.snapshot.volume = configuration.volume;
    state.snapshot.muted = configuration.muted;
    state.snapshot.looping = configuration.looping;
    state.snapshot.playback_rate = configuration.playback_rate;
    state.configuration = Some(configuration);

    if source_changed {
        state.pending_resume_position = None;
        restart_player(state, native_sender, events);
        return;
    }
    if subtitles_changed {
        preserve_resume_position(state);
        restart_player(state, native_sender, events);
        return;
    }
    if resize_changed {
        if let (Some(surface), Some(configuration)) = (&state.surface, &state.configuration) {
            if let Err(error) = surface.set_scaling_mode(configuration.resize_mode.native()) {
                report_error(
                    events,
                    VideoError::new(
                        crate::VideoErrorKind::SurfaceUnavailable,
                        "NativeWindow::set_scaling_mode",
                        format!("{error:?}"),
                    ),
                );
            }
        }
    }
    if state.ready {
        if volume_changed || muted_changed {
            apply_volume(state, events);
        }
        if looping_changed {
            apply_looping(state, events);
        }
        if rate_changed {
            apply_rate(state, events);
        }
        if autoplay_changed || active_changed {
            reconcile_playback(state, events);
        }
    }
    emit_snapshot(state, events);
}

fn surface_available(
    state: &mut WorkerState,
    registration: u64,
    surface: NativeWindow,
    native_sender: &Sender<AvPlayerEvent>,
    events: &UnboundedSender<UiEvent>,
) {
    if let Some(configuration) = &state.configuration {
        if let Err(error) = surface.set_scaling_mode(configuration.resize_mode.native()) {
            fail(
                state,
                events,
                VideoError::new(
                    crate::VideoErrorKind::SurfaceUnavailable,
                    "NativeWindow::set_scaling_mode",
                    format!("{error:?}"),
                ),
            );
            return;
        }
    }
    let same_surface = state
        .surface
        .as_ref()
        .and_then(|current| current.surface_id().ok())
        .zip(surface.surface_id().ok())
        .is_some_and(|(current, next)| current == next);
    state.surface = Some(surface);
    state.surface_registration = Some(registration);
    // ArkUI reports size changes through `on_surface_changed` with the same
    // native window. AVPlayer already follows that queue's new geometry, so a
    // resize must not reset playback or reload the source.
    if same_surface && state.player.is_some() {
        return;
    }
    preserve_resume_position(state);
    restart_player(state, native_sender, events);
}

fn surface_lost(state: &mut WorkerState, registration: u64, events: &UnboundedSender<UiEvent>) {
    if state.surface_registration != Some(registration) {
        return;
    }
    preserve_resume_position(state);
    state.player.take();
    state.surface.take();
    state.surface_registration = None;
    state.ready = false;
    state.buffering = false;
    state.native_state = AvPlayerState::Idle;
    set_status(state, events, VideoStatus::WaitingForSurface);
}

fn restart_player(
    state: &mut WorkerState,
    native_sender: &Sender<AvPlayerEvent>,
    events: &UnboundedSender<UiEvent>,
) {
    state.player.take();
    reset_media_state(state);
    if state.configuration.is_none() || state.surface.is_none() {
        set_status(state, events, VideoStatus::WaitingForSurface);
        return;
    }
    set_status(state, events, VideoStatus::Loading);
    let _ = events.send(UiEvent::LoadStart);
    match create_player(state, native_sender.clone()) {
        Ok(player) => state.player = Some(player),
        Err(error) => fail(state, events, error),
    }
}

fn create_player(
    state: &WorkerState,
    native_sender: Sender<AvPlayerEvent>,
) -> crate::VideoResult<AvPlayer> {
    let configuration = state.configuration.as_ref().ok_or_else(|| {
        VideoError::invalid_configuration("create_player", "missing player configuration")
    })?;
    let surface = state.surface.as_ref().ok_or_else(|| {
        VideoError::new(
            crate::VideoErrorKind::SurfaceUnavailable,
            "create_player",
            "missing video surface",
        )
    })?;
    let mut player = AvPlayer::new(native_sender)?;
    match &configuration.source {
        VideoSource::Network(source) if source.headers().is_empty() => {
            player.set_url_source(source.url())?;
        }
        VideoSource::Network(source) => {
            let headers = source
                .headers()
                .iter()
                .map(|(name, value)| (name.to_string(), value.to_string()))
                .collect::<Vec<_>>();
            player.set_url_source_with_headers(source.url(), &headers)?;
        }
        VideoSource::File(source) => {
            player.set_fd_source(source.raw_fd(), source.offset(), source.size())?;
        }
    }
    player.set_video_surface(surface.clone())?;
    // AVPlayer configures the producer queue while attaching the surface and
    // can replace a scaling mode set before that call. Apply presentation
    // semantics after attachment so contain/cover survive player recreation,
    // including the inline-to-fullscreen Portal move.
    surface
        .set_scaling_mode(configuration.resize_mode.native())
        .map_err(|error| {
            VideoError::new(
                crate::VideoErrorKind::SurfaceUnavailable,
                "NativeWindow::set_scaling_mode",
                format!("{error:?}"),
            )
        })?;
    for subtitle in &configuration.subtitles {
        if subtitle.url_value().trim().is_empty() {
            return Err(VideoError::invalid_source(
                "VideoSubtitleSource::url",
                "subtitle URL must not be empty",
            ));
        }
        player.add_url_subtitle(subtitle.url_value())?;
    }
    player.prepare()?;
    Ok(player)
}

fn reset_media_state(state: &mut WorkerState) {
    state.native_state = AvPlayerState::Idle;
    state.ready = false;
    state.buffering = false;
    state.pending_initial_seek = true;
    state.ready_for_display_emitted = false;
    state.ended_emitted = false;
    state.last_progress_event = None;
    state.snapshot.progress = VideoProgress::default();
    state.snapshot.size = crate::VideoSize::default();
    state.snapshot.is_live = false;
    state.snapshot.tracks.clear();
    state.snapshot.available_bitrates.clear();
}

fn playback(
    state: &mut WorkerState,
    command: PlaybackCommand,
    native_sender: &Sender<AvPlayerEvent>,
    events: &UnboundedSender<UiEvent>,
) {
    match command {
        PlaybackCommand::Play => {
            state.desired_playing = true;
            if state.snapshot.status == VideoStatus::Completed {
                seek_to(state, Duration::ZERO, VideoSeekMode::Closest, events);
            }
            reconcile_playback(state, events);
        }
        PlaybackCommand::Pause => {
            state.desired_playing = false;
            reconcile_playback(state, events);
        }
        PlaybackCommand::Stop => {
            state.desired_playing = false;
            if matches!(state.native_state, AvPlayerState::Playing) {
                call_player(state, events, AvPlayer::pause);
            }
            seek_to(state, Duration::ZERO, VideoSeekMode::Closest, events);
            if state.ready {
                set_status(state, events, VideoStatus::Stopped);
            }
        }
        PlaybackCommand::Seek(position, mode) => seek_to(state, position, mode, events),
        PlaybackCommand::SeekBy(seconds) => {
            let current = state.snapshot.progress.position.as_secs_f64();
            let duration = state.snapshot.progress.duration.as_secs_f64();
            let target = (current + seconds).clamp(0.0, duration.max(0.0));
            seek_to(
                state,
                Duration::from_secs_f64(target),
                VideoSeekMode::Closest,
                events,
            );
        }
        PlaybackCommand::Volume(volume) => {
            if let Some(configuration) = &mut state.configuration {
                configuration.volume = volume;
            }
            state.snapshot.volume = volume;
            apply_volume(state, events);
        }
        PlaybackCommand::Muted(muted) => {
            if let Some(configuration) = &mut state.configuration {
                configuration.muted = muted;
            }
            state.snapshot.muted = muted;
            apply_volume(state, events);
        }
        PlaybackCommand::Looping(looping) => {
            if let Some(configuration) = &mut state.configuration {
                configuration.looping = looping;
            }
            state.snapshot.looping = looping;
            apply_looping(state, events);
        }
        PlaybackCommand::Fullscreen(fullscreen) => {
            state.snapshot.fullscreen = fullscreen;
            let _ = events.send(UiEvent::FullscreenChanged(fullscreen));
        }
        PlaybackCommand::PlaybackRate(rate) => {
            if let Some(configuration) = &mut state.configuration {
                configuration.playback_rate = rate;
            }
            state.snapshot.playback_rate = rate;
            apply_rate(state, events);
        }
        PlaybackCommand::SelectBitrate(bitrate) => {
            call_player_with(state, events, |player| player.select_bitrate(bitrate));
        }
        PlaybackCommand::SelectTrack(index) => {
            call_player_with(state, events, |player| player.select_track(index));
        }
        PlaybackCommand::DeselectTrack(index) => {
            call_player_with(state, events, |player| player.deselect_track(index));
        }
        PlaybackCommand::ReplaceSource(source) => {
            if let Some(configuration) = &mut state.configuration {
                configuration.source = source;
                state.pending_resume_position = None;
                restart_player(state, native_sender, events);
            } else {
                report_error(
                    events,
                    VideoError::invalid_configuration(
                        "VideoController::replace_source",
                        "player is not configured",
                    ),
                );
            }
        }
    }
    emit_snapshot(state, events);
}

fn handle_native_event(
    state: &mut WorkerState,
    event: AvPlayerEvent,
    events: &UnboundedSender<UiEvent>,
) {
    match event {
        AvPlayerEvent::StateChanged(native_state) => {
            state.native_state = native_state;
            match native_state {
                AvPlayerState::Prepared => prepared(state, events),
                AvPlayerState::Playing => set_status(state, events, VideoStatus::Playing),
                AvPlayerState::Paused if state.snapshot.status != VideoStatus::Stopped => {
                    set_status(state, events, VideoStatus::Paused);
                }
                AvPlayerState::Stopped => set_status(state, events, VideoStatus::Stopped),
                AvPlayerState::Completed => complete(state, events),
                AvPlayerState::Error => {
                    let error = VideoError::new(
                        crate::VideoErrorKind::Native,
                        "OH_AVPlayerOnInfoCallback",
                        "AVPlayer entered the error state",
                    );
                    fail(state, events, error);
                }
                _ => {}
            }
        }
        AvPlayerEvent::Position(position) => {
            state.snapshot.progress.position = position;
            emit_progress_if_due(state, events, Instant::now(), false);
        }
        AvPlayerEvent::Duration(duration) => {
            state.snapshot.progress.duration = duration;
            emit_snapshot(state, events);
        }
        AvPlayerEvent::Resolution(size) => {
            state.snapshot.size = size;
            emit_ready_for_display(state, events);
            emit_snapshot(state, events);
        }
        AvPlayerEvent::Buffering(buffering) => {
            match buffering {
                VideoBuffering::Started => {
                    state.buffering = true;
                    if state.desired_playing {
                        set_status(state, events, VideoStatus::Buffering);
                    }
                }
                VideoBuffering::Ended => {
                    state.buffering = false;
                    if state.native_state == AvPlayerState::Playing {
                        set_status(state, events, VideoStatus::Playing);
                    }
                }
                VideoBuffering::CachedDuration(duration) => {
                    state.snapshot.progress.buffered = duration;
                }
                VideoBuffering::Percent(_) | VideoBuffering::Unknown { .. } => {}
                _ => {}
            }
            let _ = events.send(UiEvent::Buffering(buffering));
            emit_snapshot(state, events);
        }
        AvPlayerEvent::SeekCompleted(position) => {
            state.snapshot.progress.position = position;
            let _ = events.send(UiEvent::SeekCompleted(position));
            emit_progress_if_due(state, events, Instant::now(), true);
        }
        AvPlayerEvent::PlaybackRateChanged(rate) => {
            state.snapshot.playback_rate = rate;
            let _ = events.send(UiEvent::PlaybackRateChanged(rate));
            emit_snapshot(state, events);
        }
        AvPlayerEvent::VolumeChanged(volume) => {
            if !state.snapshot.muted {
                state.snapshot.volume = volume;
            }
            let _ = events.send(UiEvent::VolumeChanged(volume));
            emit_snapshot(state, events);
        }
        AvPlayerEvent::BitrateChanged(bitrate) => {
            let _ = events.send(UiEvent::BitrateChanged(bitrate));
        }
        AvPlayerEvent::AvailableBitrates(bitrates) => {
            state.snapshot.available_bitrates = bitrates.clone();
            let _ = events.send(UiEvent::AvailableBitrates(bitrates));
            emit_snapshot(state, events);
        }
        AvPlayerEvent::LiveChanged(is_live) => {
            state.snapshot.is_live = is_live;
            emit_snapshot(state, events);
        }
        AvPlayerEvent::TrackChanged { .. } | AvPlayerEvent::TracksChanged => {
            refresh_tracks(state, events);
        }
        AvPlayerEvent::Subtitle {
            text,
            start,
            duration,
        } => {
            let _ = events.send(UiEvent::Subtitle(VideoSubtitleCue {
                text,
                start,
                duration,
            }));
        }
        AvPlayerEvent::Ended => complete(state, events),
        AvPlayerEvent::AudioInterrupted => {
            state.desired_playing = false;
            reconcile_playback(state, events);
            let _ = events.send(UiEvent::AudioInterrupted);
        }
        AvPlayerEvent::Error(error) => fail(state, events, error.into()),
        _ => {}
    }
}

fn prepared(state: &mut WorkerState, events: &UnboundedSender<UiEvent>) {
    state.ready = true;
    state.ended_emitted = false;
    // Preparing the decoder negotiates its output buffer geometry and may
    // reset producer-queue scaling. Reapply the requested presentation mode
    // after that negotiation has completed.
    if let (Some(surface), Some(configuration)) = (&state.surface, &state.configuration) {
        if let Err(error) = surface.set_scaling_mode(configuration.resize_mode.native()) {
            report_error(
                events,
                VideoError::new(
                    crate::VideoErrorKind::SurfaceUnavailable,
                    "NativeWindow::set_scaling_mode",
                    format!("{error:?}"),
                ),
            );
        }
    }
    if let Some(player) = state.player.as_ref() {
        if let Ok(duration) = player.duration() {
            state.snapshot.progress.duration = duration;
        }
        if let Ok(size) = player.video_size() {
            state.snapshot.size = size;
        }
        state.snapshot.tracks = player.tracks();
    }
    apply_volume(state, events);
    apply_looping(state, events);
    apply_rate(state, events);
    set_status(state, events, VideoStatus::Ready);

    if state.pending_initial_seek {
        state.pending_initial_seek = false;
        let initial = state.pending_resume_position.take().unwrap_or_else(|| {
            state
                .configuration
                .as_ref()
                .map_or(Duration::ZERO, |configuration| {
                    configuration.initial_position
                })
        });
        if !initial.is_zero() {
            seek_to(state, initial, VideoSeekMode::Closest, events);
        }
    }
    let metadata = metadata(state);
    let _ = events.send(UiEvent::Loaded(metadata));
    let _ = events.send(UiEvent::TracksChanged(state.snapshot.tracks.clone()));
    emit_ready_for_display(state, events);
    reconcile_playback(state, events);
}

fn preserve_resume_position(state: &mut WorkerState) {
    if state.ready && !state.snapshot.progress.position.is_zero() {
        state.pending_resume_position = Some(state.snapshot.progress.position);
    }
}

fn complete(state: &mut WorkerState, events: &UnboundedSender<UiEvent>) {
    if state.snapshot.looping {
        state.ended_emitted = false;
        return;
    }
    state.snapshot.progress.position = state.snapshot.progress.duration;
    set_status(state, events, VideoStatus::Completed);
    emit_progress_if_due(state, events, Instant::now(), true);
    if !state.ended_emitted {
        state.ended_emitted = true;
        let _ = events.send(UiEvent::Ended);
    }
}

fn reconcile_playback(state: &mut WorkerState, events: &UnboundedSender<UiEvent>) {
    if !state.ready {
        return;
    }
    let active = state
        .configuration
        .as_ref()
        .is_some_and(|configuration| configuration.active);
    if state.desired_playing && active {
        if !matches!(state.native_state, AvPlayerState::Playing) {
            call_player(state, events, AvPlayer::play);
        }
    } else if matches!(state.native_state, AvPlayerState::Playing) {
        call_player(state, events, AvPlayer::pause);
    } else if state.snapshot.status != VideoStatus::Stopped {
        set_status(state, events, VideoStatus::Paused);
    }
}

fn apply_volume(state: &mut WorkerState, events: &UnboundedSender<UiEvent>) {
    if !state.ready {
        return;
    }
    let volume = if state.snapshot.muted {
        0.0
    } else {
        state.snapshot.volume
    };
    call_player_with(state, events, |player| player.set_volume(volume));
}

fn apply_looping(state: &mut WorkerState, events: &UnboundedSender<UiEvent>) {
    if state.ready {
        let looping = state.snapshot.looping;
        call_player_with(state, events, |player| player.set_looping(looping));
    }
}

fn apply_rate(state: &mut WorkerState, events: &UnboundedSender<UiEvent>) {
    if state.ready {
        let rate = state.snapshot.playback_rate;
        call_player_with(state, events, |player| player.set_playback_rate(rate));
    }
}

fn seek_to(
    state: &mut WorkerState,
    position: Duration,
    mode: VideoSeekMode,
    events: &UnboundedSender<UiEvent>,
) {
    if !state.ready {
        report_error(
            events,
            VideoError::new(
                crate::VideoErrorKind::InvalidState,
                "VideoController::seek",
                "video must be prepared before seeking",
            ),
        );
        return;
    }
    let duration = state.snapshot.progress.duration;
    let position = if duration.is_zero() {
        position
    } else {
        position.min(duration)
    };
    call_player_with(state, events, |player| player.seek(position, mode));
}

fn refresh_tracks(state: &mut WorkerState, events: &UnboundedSender<UiEvent>) {
    if let Some(player) = state.player.as_ref() {
        state.snapshot.tracks = player.tracks();
        let _ = events.send(UiEvent::TracksChanged(state.snapshot.tracks.clone()));
        emit_snapshot(state, events);
    }
}

fn emit_polled_progress(state: &mut WorkerState, events: &UnboundedSender<UiEvent>, now: Instant) {
    if !state.ready {
        return;
    }
    let interval = state
        .configuration
        .as_ref()
        .map_or(Duration::from_millis(250), |configuration| {
            configuration.progress_interval
        });
    if state
        .last_progress_event
        .is_some_and(|last| now.duration_since(last) < interval)
    {
        return;
    }
    if let Some(player) = state.player.as_ref() {
        if let Ok(position) = player.current_time() {
            state.snapshot.progress.position = position;
        }
        if state.snapshot.progress.duration.is_zero() {
            if let Ok(duration) = player.duration() {
                state.snapshot.progress.duration = duration;
            }
        }
    }
    emit_progress_if_due(state, events, now, true);
}

fn emit_progress_if_due(
    state: &mut WorkerState,
    events: &UnboundedSender<UiEvent>,
    now: Instant,
    force: bool,
) {
    let interval = state
        .configuration
        .as_ref()
        .map_or(Duration::from_millis(250), |configuration| {
            configuration.progress_interval
        });
    if !force
        && state
            .last_progress_event
            .is_some_and(|last| now.duration_since(last) < interval)
    {
        return;
    }
    state.last_progress_event = Some(now);
    let progress = state.snapshot.progress;
    let _ = events.send(UiEvent::Progress(progress));
    emit_snapshot(state, events);
}

fn emit_ready_for_display(state: &mut WorkerState, events: &UnboundedSender<UiEvent>) {
    if state.ready && !state.snapshot.size.is_empty() && !state.ready_for_display_emitted {
        state.ready_for_display_emitted = true;
        let _ = events.send(UiEvent::ReadyForDisplay);
    }
}

fn metadata(state: &WorkerState) -> VideoMetadata {
    VideoMetadata {
        duration: state.snapshot.progress.duration,
        size: state.snapshot.size,
        is_live: state.snapshot.is_live,
        tracks: state.snapshot.tracks.clone(),
        available_bitrates: state.snapshot.available_bitrates.clone(),
    }
}

fn set_status(state: &mut WorkerState, events: &UnboundedSender<UiEvent>, status: VideoStatus) {
    if state.snapshot.status == status {
        return;
    }
    state.snapshot.status = status.clone();
    let _ = events.send(UiEvent::Status(status));
    emit_snapshot(state, events);
}

fn emit_snapshot(state: &WorkerState, events: &UnboundedSender<UiEvent>) {
    let _ = events.send(UiEvent::Snapshot(state.snapshot.clone()));
}

fn report_error(events: &UnboundedSender<UiEvent>, error: VideoError) {
    let _ = events.send(UiEvent::Error(error));
}

fn fail(state: &mut WorkerState, events: &UnboundedSender<UiEvent>, error: VideoError) {
    state.player.take();
    state.ready = false;
    state.buffering = false;
    set_status(state, events, VideoStatus::Error(error.clone()));
    report_error(events, error);
}

fn call_player(
    state: &mut WorkerState,
    events: &UnboundedSender<UiEvent>,
    operation: fn(&mut AvPlayer) -> ohos_avplayer_binding::AvPlayerResult<()>,
) {
    call_player_with(state, events, operation);
}

fn call_player_with(
    state: &mut WorkerState,
    events: &UnboundedSender<UiEvent>,
    operation: impl FnOnce(&mut AvPlayer) -> ohos_avplayer_binding::AvPlayerResult<()>,
) {
    let result = state.player.as_mut().map(operation);
    match result {
        Some(Ok(())) => {}
        Some(Err(error)) => report_error(events, error.into()),
        None => report_error(
            events,
            VideoError::new(
                crate::VideoErrorKind::InvalidState,
                "VideoController",
                "video player is not initialized",
            ),
        ),
    }
}
