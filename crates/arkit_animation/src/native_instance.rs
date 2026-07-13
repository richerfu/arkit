use std::cell::Cell;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::rc::Rc;

use arkit_animation_core::{AnimationOutcome, IterationCount, TimePoint, TimeSpan};
use ohos_arkui_binding::animate::animator::AnimatorController;
use ohos_arkui_binding::animate::options::{Animation as ArkUiAnimation, KeyframeAnimation};
use ohos_arkui_binding::common::ui_context::ArkUIContext;
use ohos_arkui_binding::types::animation_finish_type::AnimationFinishCallbackType;

use crate::AnimationBackend;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NativeInstanceError {
    UnsupportedControl(&'static str),
    DurationOverflow(TimeSpan),
    IterationOverflow,
    Native(Box<str>),
}

impl Display for NativeInstanceError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl Error for NativeInstanceError {}

pub trait NativeAnimationInstance {
    fn backend(&self) -> AnimationBackend;
    fn play(&mut self) -> Result<(), NativeInstanceError>;
    fn pause(&mut self) -> Result<(), NativeInstanceError>;
    fn reverse(&mut self) -> Result<(), NativeInstanceError>;
    fn seek(&mut self, position: TimePoint) -> Result<(), NativeInstanceError>;
    fn complete(&mut self) -> Result<(), NativeInstanceError>;
    fn cancel(&mut self) -> Result<(), NativeInstanceError>;
    fn take_terminal(&mut self) -> Option<AnimationOutcome>;
}

#[derive(Debug, Clone)]
pub struct NativeAnimatorSpec {
    pub duration: TimeSpan,
    pub delay: TimeSpan,
    pub iterations: IterationCount,
    pub keyframes: Rc<[(f32, f32)]>,
}

impl NativeAnimatorSpec {
    pub fn progress(duration: TimeSpan) -> Self {
        Self {
            duration,
            delay: TimeSpan::ZERO,
            iterations: IterationCount::ONCE,
            keyframes: Rc::from([(0.0, 0.0), (1.0, 1.0)]),
        }
    }
}

/// Owns the real ArkUI Animator and its callback-bearing option until
/// terminal completion, cancellation, or drop. No fixed retention timer is
/// involved.
pub struct ArkUiAnimatorInstance {
    controller: AnimatorController,
    progress: Rc<Cell<f32>>,
    terminal: Rc<Cell<Option<AnimationOutcome>>>,
}

pub struct ArkUiImplicitInstance {
    context: ArkUIContext,
    animation: ArkUiAnimation,
    terminal: Rc<Cell<Option<AnimationOutcome>>>,
    started: bool,
}

impl ArkUiImplicitInstance {
    pub fn new(
        context: ArkUIContext,
        duration: TimeSpan,
        delay: TimeSpan,
        update: impl Fn() + 'static,
    ) -> Result<Self, NativeInstanceError> {
        let animation = ArkUiAnimation::new();
        animation.duration(millis_i32(duration)?);
        animation.delay(millis_i32(delay)?);
        animation.update(update);
        let terminal = Rc::new(Cell::new(None));
        let finish_terminal = terminal.clone();
        animation.finish(AnimationFinishCallbackType::Logically, move || {
            finish_terminal.set(Some(AnimationOutcome::Completed));
        });
        Ok(Self {
            context,
            animation,
            terminal,
            started: false,
        })
    }
}

impl NativeAnimationInstance for ArkUiImplicitInstance {
    fn backend(&self) -> AnimationBackend {
        AnimationBackend::ArkUiImplicit
    }

    fn play(&mut self) -> Result<(), NativeInstanceError> {
        if self.started {
            return Err(NativeInstanceError::UnsupportedControl("replay"));
        }
        self.started = true;
        self.animation.animate_to(self.context).map_err(native)
    }

    fn pause(&mut self) -> Result<(), NativeInstanceError> {
        Err(NativeInstanceError::UnsupportedControl("pause"))
    }

    fn reverse(&mut self) -> Result<(), NativeInstanceError> {
        Err(NativeInstanceError::UnsupportedControl("reverse"))
    }

    fn seek(&mut self, _position: TimePoint) -> Result<(), NativeInstanceError> {
        Err(NativeInstanceError::UnsupportedControl("seek"))
    }

    fn complete(&mut self) -> Result<(), NativeInstanceError> {
        Err(NativeInstanceError::UnsupportedControl("complete"))
    }

    fn cancel(&mut self) -> Result<(), NativeInstanceError> {
        Err(NativeInstanceError::UnsupportedControl("cancel"))
    }

    fn take_terminal(&mut self) -> Option<AnimationOutcome> {
        self.terminal.take()
    }
}

#[derive(Clone)]
pub struct NativeKeyframe {
    pub duration: TimeSpan,
    pub update: Rc<dyn Fn()>,
}

pub struct ArkUiKeyframeInstance {
    context: ArkUIContext,
    animation: KeyframeAnimation,
    terminal: Rc<Cell<Option<AnimationOutcome>>>,
    started: bool,
}

impl ArkUiKeyframeInstance {
    pub fn new(
        context: ArkUIContext,
        delay: TimeSpan,
        iterations: IterationCount,
        keyframes: impl IntoIterator<Item = NativeKeyframe>,
    ) -> Result<Self, NativeInstanceError> {
        let keyframes = keyframes.into_iter().collect::<Vec<_>>();
        let animation = KeyframeAnimation::new(
            i32::try_from(keyframes.len()).map_err(|_| NativeInstanceError::IterationOverflow)?,
        )
        .map_err(native)?;
        animation.delay(millis_i32(delay)?).map_err(native)?;
        let iterations = match iterations {
            IterationCount::Finite(iterations) => i32::try_from(iterations.get())
                .map_err(|_| NativeInstanceError::IterationOverflow)?,
            IterationCount::Infinite => -1,
        };
        animation.iterations(iterations).map_err(native)?;
        for (index, keyframe) in keyframes.into_iter().enumerate() {
            let index = i32::try_from(index).map_err(|_| NativeInstanceError::IterationOverflow)?;
            animation
                .duration(millis_i32(keyframe.duration)?, index)
                .map_err(native)?;
            let update = keyframe.update;
            animation
                .on_event_callback(index, move || update())
                .map_err(native)?;
        }
        let terminal = Rc::new(Cell::new(None));
        let finish_terminal = terminal.clone();
        animation
            .on_finish_callback(move || {
                finish_terminal.set(Some(AnimationOutcome::Completed));
            })
            .map_err(native)?;
        Ok(Self {
            context,
            animation,
            terminal,
            started: false,
        })
    }
}

impl NativeAnimationInstance for ArkUiKeyframeInstance {
    fn backend(&self) -> AnimationBackend {
        AnimationBackend::ArkUiKeyframe
    }

    fn play(&mut self) -> Result<(), NativeInstanceError> {
        if self.started {
            return Err(NativeInstanceError::UnsupportedControl("replay"));
        }
        self.started = true;
        self.animation.animate_to(self.context).map_err(native)
    }

    fn pause(&mut self) -> Result<(), NativeInstanceError> {
        Err(NativeInstanceError::UnsupportedControl("pause"))
    }

    fn reverse(&mut self) -> Result<(), NativeInstanceError> {
        Err(NativeInstanceError::UnsupportedControl("reverse"))
    }

    fn seek(&mut self, _position: TimePoint) -> Result<(), NativeInstanceError> {
        Err(NativeInstanceError::UnsupportedControl("seek"))
    }

    fn complete(&mut self) -> Result<(), NativeInstanceError> {
        Err(NativeInstanceError::UnsupportedControl("complete"))
    }

    fn cancel(&mut self) -> Result<(), NativeInstanceError> {
        Err(NativeInstanceError::UnsupportedControl("cancel"))
    }

    fn take_terminal(&mut self) -> Option<AnimationOutcome> {
        self.terminal.take()
    }
}

impl ArkUiAnimatorInstance {
    pub fn new(
        context: ArkUIContext,
        spec: NativeAnimatorSpec,
    ) -> Result<Self, NativeInstanceError> {
        let keyframe_count = i32::try_from(spec.keyframes.len())
            .map_err(|_| NativeInstanceError::IterationOverflow)?;
        let mut controller = AnimatorController::new(context, keyframe_count).map_err(native)?;
        let duration = millis_i32(spec.duration)?;
        let delay = millis_i32(spec.delay)?;
        let iterations = match spec.iterations {
            IterationCount::Finite(iterations) => i32::try_from(iterations.get())
                .map_err(|_| NativeInstanceError::IterationOverflow)?,
            IterationCount::Infinite => -1,
        };
        let progress = Rc::new(Cell::new(0.0));
        let terminal = Rc::new(Cell::new(None));
        let frame_progress = progress.clone();
        let finish_terminal = terminal.clone();
        let cancel_terminal = terminal.clone();
        let option = controller.option_mut();
        option
            .duration(duration)
            .and_then(|option| option.delay(delay))
            .and_then(|option| option.iterations(iterations))
            .map_err(native)?;
        for (index, (time, value)) in spec.keyframes.iter().copied().enumerate() {
            option
                .keyframe(
                    time.clamp(0.0, 1.0),
                    value,
                    i32::try_from(index).map_err(|_| NativeInstanceError::IterationOverflow)?,
                )
                .map_err(native)?;
        }
        option
            .on_frame(move |event| frame_progress.set(event.value()))
            .and_then(|option| {
                option.on_finish(move |_| {
                    finish_terminal.set(Some(AnimationOutcome::Completed));
                })
            })
            .and_then(|option| {
                option.on_cancel(move |_| {
                    cancel_terminal.set(Some(AnimationOutcome::Cancelled));
                })
            })
            .map_err(native)?;
        controller.commit_option().map_err(native)?;
        Ok(Self {
            controller,
            progress,
            terminal,
        })
    }

    pub fn progress(&self) -> f32 {
        self.progress.get()
    }
}

impl NativeAnimationInstance for ArkUiAnimatorInstance {
    fn backend(&self) -> AnimationBackend {
        AnimationBackend::ArkUiAnimator
    }

    fn play(&mut self) -> Result<(), NativeInstanceError> {
        self.controller.play().map_err(native)
    }

    fn pause(&mut self) -> Result<(), NativeInstanceError> {
        self.controller.pause().map_err(native)
    }

    fn reverse(&mut self) -> Result<(), NativeInstanceError> {
        self.controller.reverse().map_err(native)
    }

    fn seek(&mut self, _position: TimePoint) -> Result<(), NativeInstanceError> {
        Err(NativeInstanceError::UnsupportedControl("seek"))
    }

    fn complete(&mut self) -> Result<(), NativeInstanceError> {
        self.controller.finish().map_err(native)
    }

    fn cancel(&mut self) -> Result<(), NativeInstanceError> {
        self.controller.cancel().map_err(native)
    }

    fn take_terminal(&mut self) -> Option<AnimationOutcome> {
        self.terminal.take()
    }
}

fn millis_i32(duration: TimeSpan) -> Result<i32, NativeInstanceError> {
    let millis = duration.as_nanos() / arkit_animation_core::NANOS_PER_MILLISECOND;
    i32::try_from(millis).map_err(|_| NativeInstanceError::DurationOverflow(duration))
}

fn native(error: impl Display) -> NativeInstanceError {
    NativeInstanceError::Native(error.to_string().into_boxed_str())
}
