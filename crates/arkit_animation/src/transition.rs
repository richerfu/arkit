use arkit_animation_core::{Angle, BuiltinEase, Easing, Length, TimeSpan, TimelinePosition};
use arkit_prelude::*;

use crate::api::{Animation, Timeline};
use crate::properties::{OPACITY, ROTATION, SCALE_X, SCALE_Y, TRANSLATE_X, TRANSLATE_Y};
use crate::{use_animation, use_animation_target, AnimationSelector};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TransitionPreset {
    Fade,
    #[default]
    SlideUp,
    SlideDown,
    SlideLeft,
    SlideRight,
    ZoomIn,
    ZoomOut,
    RotateClockwise,
    RotateCounterClockwise,
}

#[derive(Debug, Clone, Copy)]
struct TransitionValues {
    opacity: f32,
    translate_x: f32,
    translate_y: f32,
    scale: f32,
    rotation: f32,
}

impl TransitionPreset {
    fn values(self) -> TransitionValues {
        const DISTANCE: f32 = 48.0;
        const ROTATION: f32 = 32.0;
        match self {
            Self::Fade => TransitionValues::new(),
            Self::SlideUp => TransitionValues::motion().translate(0.0, DISTANCE),
            Self::SlideDown => TransitionValues::motion().translate(0.0, -DISTANCE),
            Self::SlideLeft => TransitionValues::motion().translate(DISTANCE, 0.0),
            Self::SlideRight => TransitionValues::motion().translate(-DISTANCE, 0.0),
            Self::ZoomIn => TransitionValues::motion().scale(0.68),
            Self::ZoomOut => TransitionValues::motion().scale(1.32),
            Self::RotateClockwise => TransitionValues::motion().scale(0.82).rotate(-ROTATION),
            Self::RotateCounterClockwise => TransitionValues::motion().scale(0.82).rotate(ROTATION),
        }
    }
}

impl TransitionValues {
    const fn new() -> Self {
        Self {
            opacity: 0.0,
            translate_x: 0.0,
            translate_y: 0.0,
            scale: 1.0,
            rotation: 0.0,
        }
    }

    const fn motion() -> Self {
        Self {
            opacity: 0.35,
            ..Self::new()
        }
    }

    const fn translate(mut self, x: f32, y: f32) -> Self {
        self.translate_x = x;
        self.translate_y = y;
        self
    }

    const fn scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }

    const fn rotate(mut self, rotation: f32) -> Self {
        self.rotation = rotation;
        self
    }
}

fn transition_timeline(
    target: arkit_animation_core::TargetName,
    preset: TransitionPreset,
    duration: TimeSpan,
    delay: TimeSpan,
) -> Timeline {
    let values = preset.values();
    let easing = Easing::Builtin(BuiltinEase::Cubic(arkit_animation_core::EaseDirection::Out));
    let animation = Animation::new(AnimationSelector::Target(target))
        .tween(&OPACITY, values.opacity, 1.0, duration)
        .configure_last(
            easing.clone(),
            Default::default(),
            Default::default(),
            delay,
            0,
        )
        .tween(
            &TRANSLATE_X,
            Length::vp(values.translate_x),
            Length::vp(0.0),
            duration,
        )
        .configure_last(
            easing.clone(),
            Default::default(),
            Default::default(),
            delay,
            0,
        )
        .tween(
            &TRANSLATE_Y,
            Length::vp(values.translate_y),
            Length::vp(0.0),
            duration,
        )
        .configure_last(
            easing.clone(),
            Default::default(),
            Default::default(),
            delay,
            0,
        )
        .tween(&SCALE_X, values.scale, 1.0, duration)
        .configure_last(
            easing.clone(),
            Default::default(),
            Default::default(),
            delay,
            0,
        )
        .tween(&SCALE_Y, values.scale, 1.0, duration)
        .configure_last(
            easing.clone(),
            Default::default(),
            Default::default(),
            delay,
            0,
        )
        .tween(
            &ROTATION,
            Angle::degrees(values.rotation),
            Angle::degrees(0.0),
            duration,
        )
        .configure_last(easing, Default::default(), Default::default(), delay, 0);
    Timeline::new().add(animation, TimelinePosition::START)
}

#[component]
pub fn MountTransition(
    children: Element,
    #[props(default)] preset: Option<TransitionPreset>,
    #[props(default)] duration_ms: Option<i32>,
    #[props(default)] delay_ms: Option<i32>,
    #[props(default)] replay_id: Option<u64>,
    #[props(default)] fill: Option<bool>,
) -> Element {
    let name = use_hook(|| {
        arkit_animation_core::TargetName::owned(format!(
            "mount-transition-{:?}",
            current_scope_id()
        ))
    });
    let target = use_animation_target(name.as_str().to_owned());
    let duration = TimeSpan::from_millis(duration_ms.unwrap_or(180).max(0) as u64);
    let delay = TimeSpan::from_millis(delay_ms.unwrap_or(0).max(0) as u64);
    let selected_preset = preset.unwrap_or_default();
    let controls = use_animation(transition_timeline(
        name.clone(),
        selected_preset,
        duration,
        delay,
    ));
    let request = (selected_preset, duration, delay, replay_id.unwrap_or(0));
    let mut active = use_signal(|| None);
    use_effect(use_reactive((&request,), move |(request,)| {
        if !target.is_ready() || !controls.is_ready() || *active.peek() == Some(request) {
            return;
        }
        controls.set_timeline(transition_timeline(
            name.clone(),
            request.0,
            request.1,
            request.2,
        ));
        active.set(Some(request));
        controls.restart();
    }));
    if fill.unwrap_or(false) {
        rsx! {
            column {
                percent_width: 1.0,
                percent_height: 1.0,
                align_items: "start",
                {children}
            }
        }
    } else {
        rsx! {
            column {
                align_items: "start",
                {children}
            }
        }
    }
}
