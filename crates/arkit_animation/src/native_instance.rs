use std::cell::Cell;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::ptr::NonNull;
use std::rc::Rc;

use arkit_animation_core::{AnimationOutcome, IterationCount, TimePoint, TimeSpan};
use arkit_hooks::HostNode;
use ohos_arkui_binding::animate::animator::AnimatorController;
use ohos_arkui_binding::animate::options::{Animation as ArkUiAnimation, KeyframeAnimation};
use ohos_arkui_binding::common::ui_context::ArkUIContext;
use ohos_arkui_binding::types::animation_finish_type::AnimationFinishCallbackType;
use ohos_arkui_sys::{
    ArkUI_Animator, ArkUI_AnimatorEvent, ArkUI_AnimatorOnFrameEvent, ArkUI_AnimatorOption,
    ArkUI_NativeAPIVariantKind_ARKUI_NATIVE_ANIMATE, ArkUI_NativeAnimateAPI_1,
    OH_ArkUI_AnimatorEvent_GetUserData, OH_ArkUI_AnimatorOnFrameEvent_GetUserData,
    OH_ArkUI_AnimatorOnFrameEvent_GetValue, OH_ArkUI_AnimatorOption_Create,
    OH_ArkUI_AnimatorOption_Dispose, OH_ArkUI_AnimatorOption_RegisterOnCancelCallback,
    OH_ArkUI_AnimatorOption_RegisterOnFinishCallback,
    OH_ArkUI_AnimatorOption_RegisterOnFrameCallback, OH_ArkUI_AnimatorOption_SetDelay,
    OH_ArkUI_AnimatorOption_SetDuration, OH_ArkUI_AnimatorOption_SetIterations,
    OH_ArkUI_AnimatorOption_SetKeyframe, OH_ArkUI_Animator_Cancel, OH_ArkUI_Animator_Finish,
    OH_ArkUI_Animator_Pause, OH_ArkUI_Animator_Play, OH_ArkUI_Animator_Reverse,
    OH_ArkUI_GetContextByNode, OH_ArkUI_QueryModuleInterfaceByName,
};

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
    fn take_progress(&mut self) -> Option<f32> {
        None
    }
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
    progress: Rc<Cell<Option<f32>>>,
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
        let progress = Rc::new(Cell::new(None));
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
            .on_frame(move |event| frame_progress.set(Some(event.value())))
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
        self.progress.get().unwrap_or(0.0)
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

    fn take_progress(&mut self) -> Option<f32> {
        self.progress.take()
    }

    fn take_terminal(&mut self) -> Option<AnimationOutcome> {
        self.terminal.take()
    }
}

struct NodeAnimatorCallbacks {
    progress: Cell<Option<f32>>,
    terminal: Cell<Option<AnimationOutcome>>,
}

/// Animator clock created from a mounted ArkUI node. Unlike the public
/// N-API-facing owner above, this path obtains the native UI context directly
/// from the node and can therefore be installed by the root animation host.
pub(crate) struct ArkUiNodeAnimatorInstance {
    animator: NonNull<ArkUI_Animator>,
    option: NonNull<ArkUI_AnimatorOption>,
    dispose_animator: unsafe extern "C" fn(*mut ArkUI_Animator),
    callbacks: Box<NodeAnimatorCallbacks>,
}

impl ArkUiNodeAnimatorInstance {
    pub(crate) fn new(
        node: &HostNode,
        spec: NativeAnimatorSpec,
    ) -> Result<Self, NativeInstanceError> {
        let raw_node = node.borrow().raw_handle();
        // SAFETY: `raw_node` belongs to the mounted `HostNode` retained by the
        // renderer. ArkUI accepts it for context lookup on the UI thread.
        let context = unsafe { OH_ArkUI_GetContextByNode(raw_node) };
        if context.is_null() {
            return Err(NativeInstanceError::Native(
                "OH_ArkUI_GetContextByNode returned null".into(),
            ));
        }

        let keyframe_count = i32::try_from(spec.keyframes.len())
            .map_err(|_| NativeInstanceError::IterationOverflow)?;
        // SAFETY: creation has no aliasing requirements; null is handled.
        let option = NonNull::new(unsafe { OH_ArkUI_AnimatorOption_Create(keyframe_count) })
            .ok_or_else(|| {
                NativeInstanceError::Native("OH_ArkUI_AnimatorOption_Create returned null".into())
            })?;
        let mut option_guard = AnimatorOptionGuard(Some(option));
        let iterations = match spec.iterations {
            IterationCount::Finite(iterations) => i32::try_from(iterations.get())
                .map_err(|_| NativeInstanceError::IterationOverflow)?,
            IterationCount::Infinite => -1,
        };
        // SAFETY: `option` is a live ArkUI option uniquely owned by the guard.
        unsafe {
            native_status(
                "OH_ArkUI_AnimatorOption_SetDuration",
                OH_ArkUI_AnimatorOption_SetDuration(option.as_ptr(), millis_i32(spec.duration)?),
            )?;
            native_status(
                "OH_ArkUI_AnimatorOption_SetDelay",
                OH_ArkUI_AnimatorOption_SetDelay(option.as_ptr(), millis_i32(spec.delay)?),
            )?;
            native_status(
                "OH_ArkUI_AnimatorOption_SetIterations",
                OH_ArkUI_AnimatorOption_SetIterations(option.as_ptr(), iterations),
            )?;
            for (index, (time, value)) in spec.keyframes.iter().copied().enumerate() {
                native_status(
                    "OH_ArkUI_AnimatorOption_SetKeyframe",
                    OH_ArkUI_AnimatorOption_SetKeyframe(
                        option.as_ptr(),
                        time.clamp(0.0, 1.0),
                        value,
                        i32::try_from(index).map_err(|_| NativeInstanceError::IterationOverflow)?,
                    ),
                )?;
            }
        }

        let callbacks = Box::new(NodeAnimatorCallbacks {
            progress: Cell::new(None),
            terminal: Cell::new(None),
        });
        let callback_data = (&*callbacks as *const NodeAnimatorCallbacks)
            .cast_mut()
            .cast();
        // SAFETY: callback data points into a Box that remains stable until
        // callbacks are unregistered during Drop.
        unsafe {
            native_status(
                "OH_ArkUI_AnimatorOption_RegisterOnFrameCallback",
                OH_ArkUI_AnimatorOption_RegisterOnFrameCallback(
                    option.as_ptr(),
                    callback_data,
                    Some(node_animator_frame),
                ),
            )?;
            native_status(
                "OH_ArkUI_AnimatorOption_RegisterOnFinishCallback",
                OH_ArkUI_AnimatorOption_RegisterOnFinishCallback(
                    option.as_ptr(),
                    callback_data,
                    Some(node_animator_finish),
                ),
            )?;
            native_status(
                "OH_ArkUI_AnimatorOption_RegisterOnCancelCallback",
                OH_ArkUI_AnimatorOption_RegisterOnCancelCallback(
                    option.as_ptr(),
                    callback_data,
                    Some(node_animator_cancel),
                ),
            )?;
        }

        const ANIMATE_API_NAME: &[u8] = b"ArkUI_NativeAnimateAPI_1\0";
        // SAFETY: the symbol name is NUL-terminated and the returned pointer
        // is checked before being cast to the documented versioned API table.
        let api = NonNull::new(unsafe {
            OH_ArkUI_QueryModuleInterfaceByName(
                ArkUI_NativeAPIVariantKind_ARKUI_NATIVE_ANIMATE,
                ANIMATE_API_NAME.as_ptr().cast(),
            )
        })
        .ok_or_else(|| {
            NativeInstanceError::Native("ArkUI_NativeAnimateAPI_1 is unavailable".into())
        })?
        .cast::<ArkUI_NativeAnimateAPI_1>();
        // SAFETY: `api` was obtained for this exact versioned interface.
        let api = unsafe { api.as_ref() };
        let create_animator = api.createAnimator.ok_or_else(|| {
            NativeInstanceError::Native(
                "ArkUI_NativeAnimateAPI_1::createAnimator is unavailable".into(),
            )
        })?;
        let dispose_animator = api.disposeAnimator.ok_or_else(|| {
            NativeInstanceError::Native(
                "ArkUI_NativeAnimateAPI_1::disposeAnimator is unavailable".into(),
            )
        })?;
        // SAFETY: context and option are live handles created for the current
        // UI thread. Null indicates native construction failure.
        let animator = NonNull::new(unsafe { create_animator(context, option.as_ptr()) })
            .ok_or_else(|| {
                NativeInstanceError::Native(
                    "ArkUI_NativeAnimateAPI_1::createAnimator returned null".into(),
                )
            })?;
        option_guard.disarm();
        Ok(Self {
            animator,
            option,
            dispose_animator,
            callbacks,
        })
    }
}

impl NativeAnimationInstance for ArkUiNodeAnimatorInstance {
    fn backend(&self) -> AnimationBackend {
        AnimationBackend::ArkUiAnimator
    }

    fn play(&mut self) -> Result<(), NativeInstanceError> {
        self.callbacks.terminal.take();
        // SAFETY: animator is live and UI-thread confined for `self`'s life.
        native_status("OH_ArkUI_Animator_Play", unsafe {
            OH_ArkUI_Animator_Play(self.animator.as_ptr())
        })
    }

    fn pause(&mut self) -> Result<(), NativeInstanceError> {
        // SAFETY: animator is live and UI-thread confined for `self`'s life.
        native_status("OH_ArkUI_Animator_Pause", unsafe {
            OH_ArkUI_Animator_Pause(self.animator.as_ptr())
        })
    }

    fn reverse(&mut self) -> Result<(), NativeInstanceError> {
        // SAFETY: animator is live and UI-thread confined for `self`'s life.
        native_status("OH_ArkUI_Animator_Reverse", unsafe {
            OH_ArkUI_Animator_Reverse(self.animator.as_ptr())
        })
    }

    fn seek(&mut self, _position: TimePoint) -> Result<(), NativeInstanceError> {
        Err(NativeInstanceError::UnsupportedControl("seek"))
    }

    fn complete(&mut self) -> Result<(), NativeInstanceError> {
        // SAFETY: animator is live and UI-thread confined for `self`'s life.
        native_status("OH_ArkUI_Animator_Finish", unsafe {
            OH_ArkUI_Animator_Finish(self.animator.as_ptr())
        })
    }

    fn cancel(&mut self) -> Result<(), NativeInstanceError> {
        // SAFETY: animator is live and UI-thread confined for `self`'s life.
        native_status("OH_ArkUI_Animator_Cancel", unsafe {
            OH_ArkUI_Animator_Cancel(self.animator.as_ptr())
        })
    }

    fn take_progress(&mut self) -> Option<f32> {
        self.callbacks.progress.take()
    }

    fn take_terminal(&mut self) -> Option<AnimationOutcome> {
        self.callbacks.terminal.take()
    }
}

impl Drop for ArkUiNodeAnimatorInstance {
    fn drop(&mut self) {
        // SAFETY: unregistering first prevents native callbacks from observing
        // `callbacks` after it is freed. Handles remain live until the end of
        // this block and are disposed exactly once.
        unsafe {
            let option = self.option.as_ptr();
            let _ =
                OH_ArkUI_AnimatorOption_RegisterOnFrameCallback(option, std::ptr::null_mut(), None);
            let _ = OH_ArkUI_AnimatorOption_RegisterOnFinishCallback(
                option,
                std::ptr::null_mut(),
                None,
            );
            let _ = OH_ArkUI_AnimatorOption_RegisterOnCancelCallback(
                option,
                std::ptr::null_mut(),
                None,
            );
            (self.dispose_animator)(self.animator.as_ptr());
            OH_ArkUI_AnimatorOption_Dispose(option);
        }
    }
}

struct AnimatorOptionGuard(Option<NonNull<ArkUI_AnimatorOption>>);

impl AnimatorOptionGuard {
    fn disarm(&mut self) {
        self.0 = None;
    }
}

impl Drop for AnimatorOptionGuard {
    fn drop(&mut self) {
        let Some(option) = self.0.take() else {
            return;
        };
        // SAFETY: the guard uniquely owns this option until disarmed.
        unsafe { OH_ArkUI_AnimatorOption_Dispose(option.as_ptr()) };
    }
}

unsafe extern "C" fn node_animator_frame(event: *mut ArkUI_AnimatorOnFrameEvent) {
    // SAFETY: ArkUI invokes this callback with the event registered above.
    let data = unsafe { OH_ArkUI_AnimatorOnFrameEvent_GetUserData(event) };
    let Some(callbacks) = NonNull::new(data.cast::<NodeAnimatorCallbacks>()) else {
        return;
    };
    // SAFETY: callback unregistration precedes freeing the boxed context.
    let callbacks = unsafe { callbacks.as_ref() };
    // SAFETY: `event` is live for the callback invocation.
    callbacks.progress.set(Some(unsafe {
        OH_ArkUI_AnimatorOnFrameEvent_GetValue(event)
    }));
}

unsafe extern "C" fn node_animator_finish(event: *mut ArkUI_AnimatorEvent) {
    set_node_animator_terminal(event, AnimationOutcome::Completed);
}

unsafe extern "C" fn node_animator_cancel(event: *mut ArkUI_AnimatorEvent) {
    set_node_animator_terminal(event, AnimationOutcome::Cancelled);
}

fn set_node_animator_terminal(event: *mut ArkUI_AnimatorEvent, outcome: AnimationOutcome) {
    // SAFETY: this function is called only from ArkUI animator callbacks.
    let data = unsafe { OH_ArkUI_AnimatorEvent_GetUserData(event) };
    let Some(callbacks) = NonNull::new(data.cast::<NodeAnimatorCallbacks>()) else {
        return;
    };
    // SAFETY: callback unregistration precedes freeing the boxed context.
    unsafe { callbacks.as_ref() }.terminal.set(Some(outcome));
}

fn native_status(operation: &'static str, status: i32) -> Result<(), NativeInstanceError> {
    if status == 0 {
        Ok(())
    } else {
        Err(NativeInstanceError::Native(
            format!("{operation} failed with ArkUI status {status}").into_boxed_str(),
        ))
    }
}

fn millis_i32(duration: TimeSpan) -> Result<i32, NativeInstanceError> {
    let millis = duration.as_nanos() / arkit_animation_core::NANOS_PER_MILLISECOND;
    i32::try_from(millis).map_err(|_| NativeInstanceError::DurationOverflow(duration))
}

fn native(error: impl Display) -> NativeInstanceError {
    NativeInstanceError::Native(error.to_string().into_boxed_str())
}
