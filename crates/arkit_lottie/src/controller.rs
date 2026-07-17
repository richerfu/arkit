use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::Sender;

use crate::worker::{PlaybackCommand, WorkerMessage};
use crate::{LottieComposition, LottieFrame, LottieRepeatMode, LottieResult, LottieStatus};

struct ControllerBinding {
    id: u64,
    sender: Sender<WorkerMessage>,
}

#[derive(Default)]
struct ControllerState {
    next_binding: u64,
    binding: Option<ControllerBinding>,
    status: LottieStatus,
    composition: Option<LottieComposition>,
    frame: LottieFrame,
}

/// Imperative playback handle for a mounted [`crate::LottiePlayer`].
#[derive(Clone, Default)]
pub struct LottieController {
    inner: Rc<RefCell<ControllerState>>,
}

impl LottieController {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn play(&self) -> LottieResult<()> {
        self.send("LottieController::play", PlaybackCommand::Play)
    }

    pub fn pause(&self) -> LottieResult<()> {
        self.send("LottieController::pause", PlaybackCommand::Pause)
    }

    pub fn toggle(&self) -> LottieResult<()> {
        if self.status().is_playing() {
            self.pause()
        } else {
            self.play()
        }
    }

    /// Pause and return to the first frame.
    pub fn stop(&self) -> LottieResult<()> {
        self.send("LottieController::stop", PlaybackCommand::Stop)
    }

    /// Seek to a normalized position in `0.0..=1.0`.
    pub fn seek(&self, progress: f32) -> LottieResult<()> {
        if !progress.is_finite() {
            return Err(crate::LottieError::invalid_configuration(
                "LottieController::seek",
                "progress must be finite",
            ));
        }
        self.send(
            "LottieController::seek",
            PlaybackCommand::SeekProgress(progress),
        )
    }

    pub fn seek_frame(&self, frame: f32) -> LottieResult<()> {
        if !frame.is_finite() {
            return Err(crate::LottieError::invalid_configuration(
                "LottieController::seek_frame",
                "frame must be finite",
            ));
        }
        self.send(
            "LottieController::seek_frame",
            PlaybackCommand::SeekFrame(frame),
        )
    }

    /// Set signed playback speed. Negative values play backwards.
    pub fn set_speed(&self, speed: f32) -> LottieResult<()> {
        if !speed.is_finite() || speed == 0.0 {
            return Err(crate::LottieError::invalid_configuration(
                "LottieController::set_speed",
                "speed must be finite and non-zero",
            ));
        }
        self.send("LottieController::set_speed", PlaybackCommand::Speed(speed))
    }

    pub fn set_repeat_mode(&self, repeat: LottieRepeatMode) -> LottieResult<()> {
        self.send(
            "LottieController::set_repeat_mode",
            PlaybackCommand::Repeat(repeat),
        )
    }

    pub fn status(&self) -> LottieStatus {
        self.inner.borrow().status.clone()
    }

    pub fn composition(&self) -> Option<LottieComposition> {
        self.inner.borrow().composition
    }

    pub fn frame(&self) -> LottieFrame {
        self.inner.borrow().frame
    }

    pub fn is_mounted(&self) -> bool {
        self.inner.borrow().binding.is_some()
    }

    pub(crate) fn bind(&self, sender: Sender<WorkerMessage>) -> u64 {
        let mut state = self.inner.borrow_mut();
        state.next_binding = state
            .next_binding
            .checked_add(1)
            .expect("arkit_lottie: controller binding id exhausted");
        let id = state.next_binding;
        state.binding = Some(ControllerBinding { id, sender });
        state.status = LottieStatus::WaitingForSurface;
        state.composition = None;
        state.frame = LottieFrame::default();
        id
    }

    pub(crate) fn update_status(&self, binding: u64, status: LottieStatus) {
        let mut state = self.inner.borrow_mut();
        if state
            .binding
            .as_ref()
            .is_some_and(|current| current.id == binding)
        {
            state.status = status;
        }
    }

    pub(crate) fn update_composition(&self, binding: u64, composition: LottieComposition) {
        let mut state = self.inner.borrow_mut();
        if state
            .binding
            .as_ref()
            .is_some_and(|current| current.id == binding)
        {
            state.composition = Some(composition);
        }
    }

    pub(crate) fn update_frame(&self, binding: u64, frame: LottieFrame) {
        let mut state = self.inner.borrow_mut();
        if state
            .binding
            .as_ref()
            .is_some_and(|current| current.id == binding)
        {
            state.frame = frame;
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
            state.status = LottieStatus::Idle;
            state.composition = None;
            state.frame = LottieFrame::default();
        }
    }

    fn send(&self, operation: &'static str, command: PlaybackCommand) -> LottieResult<()> {
        let sender = self
            .inner
            .borrow()
            .binding
            .as_ref()
            .map(|binding| binding.sender.clone())
            .ok_or_else(|| crate::LottieError::worker_stopped(operation))?;
        sender
            .send(WorkerMessage::Playback(command))
            .map_err(|_| crate::LottieError::worker_stopped(operation))
    }
}

impl std::fmt::Debug for LottieController {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("LottieController")
            .field("mounted", &self.is_mounted())
            .field("status", &self.status())
            .field("composition", &self.composition())
            .field("frame", &self.frame())
            .finish()
    }
}

impl PartialEq for LottieController {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.inner, &other.inner)
    }
}
