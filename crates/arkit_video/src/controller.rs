use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::Sender;
use std::time::Duration;

use crate::worker::{PlaybackCommand, WorkerMessage};
use crate::{VideoResult, VideoSeekMode, VideoSnapshot, VideoSource, VideoStatus};

struct ControllerBinding {
    id: u64,
    sender: Sender<WorkerMessage>,
}

#[derive(Default)]
struct ControllerState {
    next_binding: u64,
    binding: Option<ControllerBinding>,
    snapshot: VideoSnapshot,
}

/// Imperative playback handle for one mounted [`crate::VideoPlayer`].
#[derive(Clone, Default)]
pub struct VideoController {
    inner: Rc<RefCell<ControllerState>>,
}

impl VideoController {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn play(&self) -> VideoResult<()> {
        self.send("VideoController::play", PlaybackCommand::Play)
    }

    pub fn pause(&self) -> VideoResult<()> {
        self.send("VideoController::pause", PlaybackCommand::Pause)
    }

    pub fn toggle(&self) -> VideoResult<()> {
        if self.status().is_playing() {
            self.pause()
        } else {
            self.play()
        }
    }

    /// Pause playback and seek to the start without discarding the source.
    pub fn stop(&self) -> VideoResult<()> {
        self.send("VideoController::stop", PlaybackCommand::Stop)
    }

    pub fn seek(&self, position: Duration) -> VideoResult<()> {
        self.seek_with_mode(position, VideoSeekMode::Closest)
    }

    pub fn seek_with_mode(&self, position: Duration, mode: VideoSeekMode) -> VideoResult<()> {
        self.send(
            "VideoController::seek_with_mode",
            PlaybackCommand::Seek(position, mode),
        )
    }

    /// Seek relative to the latest reported position. Negative values rewind.
    pub fn seek_by(&self, seconds: f64) -> VideoResult<()> {
        if !seconds.is_finite() {
            return Err(crate::VideoError::invalid_configuration(
                "VideoController::seek_by",
                "seconds must be finite",
            ));
        }
        self.send("VideoController::seek_by", PlaybackCommand::SeekBy(seconds))
    }

    pub fn set_volume(&self, volume: f32) -> VideoResult<()> {
        if !volume.is_finite() || !(0.0..=1.0).contains(&volume) {
            return Err(crate::VideoError::invalid_configuration(
                "VideoController::set_volume",
                "volume must be finite and within 0.0..=1.0",
            ));
        }
        self.send(
            "VideoController::set_volume",
            PlaybackCommand::Volume(volume),
        )
    }

    pub fn set_muted(&self, muted: bool) -> VideoResult<()> {
        self.send("VideoController::set_muted", PlaybackCommand::Muted(muted))
    }

    pub fn set_looping(&self, looping: bool) -> VideoResult<()> {
        self.send(
            "VideoController::set_looping",
            PlaybackCommand::Looping(looping),
        )
    }

    /// Present the player in the root fullscreen overlay.
    pub fn enter_fullscreen(&self) -> VideoResult<()> {
        self.send(
            "VideoController::enter_fullscreen",
            PlaybackCommand::Fullscreen(true),
        )
    }

    /// Return the player to its inline layout slot.
    pub fn exit_fullscreen(&self) -> VideoResult<()> {
        self.send(
            "VideoController::exit_fullscreen",
            PlaybackCommand::Fullscreen(false),
        )
    }

    pub fn toggle_fullscreen(&self) -> VideoResult<()> {
        self.send(
            "VideoController::toggle_fullscreen",
            PlaybackCommand::Fullscreen(!self.snapshot().fullscreen),
        )
    }

    pub fn set_playback_rate(&self, rate: f32) -> VideoResult<()> {
        if !rate.is_finite() || !(0.125..=4.0).contains(&rate) {
            return Err(crate::VideoError::invalid_configuration(
                "VideoController::set_playback_rate",
                "playback rate must be finite and within 0.125..=4.0",
            ));
        }
        self.send(
            "VideoController::set_playback_rate",
            PlaybackCommand::PlaybackRate(rate),
        )
    }

    pub fn select_bitrate(&self, bitrate: u32) -> VideoResult<()> {
        self.send(
            "VideoController::select_bitrate",
            PlaybackCommand::SelectBitrate(bitrate),
        )
    }

    pub fn select_track(&self, index: i32) -> VideoResult<()> {
        self.send(
            "VideoController::select_track",
            PlaybackCommand::SelectTrack(index),
        )
    }

    pub fn deselect_track(&self, index: i32) -> VideoResult<()> {
        self.send(
            "VideoController::deselect_track",
            PlaybackCommand::DeselectTrack(index),
        )
    }

    /// Replace the current source while keeping the current playback intent.
    pub fn replace_source(&self, source: VideoSource) -> VideoResult<()> {
        source.validate()?;
        self.send(
            "VideoController::replace_source",
            PlaybackCommand::ReplaceSource(source),
        )
    }

    pub fn snapshot(&self) -> VideoSnapshot {
        self.inner.borrow().snapshot.clone()
    }

    pub fn status(&self) -> VideoStatus {
        self.inner.borrow().snapshot.status.clone()
    }

    pub fn is_fullscreen(&self) -> bool {
        self.inner.borrow().snapshot.fullscreen
    }

    pub fn is_mounted(&self) -> bool {
        self.inner.borrow().binding.is_some()
    }

    pub(crate) fn bind(&self, sender: Sender<WorkerMessage>) -> u64 {
        let mut state = self.inner.borrow_mut();
        state.next_binding = state
            .next_binding
            .checked_add(1)
            .expect("arkit_video: controller binding id exhausted");
        let id = state.next_binding;
        state.binding = Some(ControllerBinding { id, sender });
        state.snapshot = VideoSnapshot {
            status: VideoStatus::WaitingForSurface,
            ..VideoSnapshot::default()
        };
        id
    }

    pub(crate) fn update_snapshot(&self, binding: u64, snapshot: VideoSnapshot) {
        let mut state = self.inner.borrow_mut();
        if state
            .binding
            .as_ref()
            .is_some_and(|current| current.id == binding)
        {
            state.snapshot = snapshot;
        }
    }

    pub(crate) fn unbind(&self, binding: u64) {
        let mut state = self.inner.borrow_mut();
        if state
            .binding
            .as_ref()
            .is_some_and(|current| current.id == binding)
        {
            state.binding = None;
            state.snapshot = VideoSnapshot::default();
        }
    }

    fn send(&self, operation: &'static str, command: PlaybackCommand) -> VideoResult<()> {
        let sender = self
            .inner
            .borrow()
            .binding
            .as_ref()
            .map(|binding| binding.sender.clone())
            .ok_or_else(|| crate::VideoError::worker_stopped(operation))?;
        sender
            .send(WorkerMessage::Playback(command))
            .map_err(|_| crate::VideoError::worker_stopped(operation))
    }
}

impl std::fmt::Debug for VideoController {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("VideoController")
            .field("mounted", &self.is_mounted())
            .field("snapshot", &self.snapshot())
            .finish()
    }
}

impl PartialEq for VideoController {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.inner, &other.inner)
    }
}
