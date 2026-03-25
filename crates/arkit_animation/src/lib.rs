use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use arkit::ohos_arkui_binding::animate::options::Animation;
use arkit::ohos_arkui_binding::animate::transition::{
    RotationOptions, ScaleOptions, TransitionEffect, TranslationOptions,
};
use arkit::ohos_arkui_binding::common::error::ArkUIResult;
use arkit::ohos_arkui_binding::common::node::ArkUINode;
use arkit::ohos_arkui_binding::component::attribute::ArkUICommonAttribute;
use arkit::ohos_arkui_binding::types::animation_finish_type::AnimationFinishCallbackType;
use arkit::ohos_arkui_binding::types::animation_mode::AnimationMode;
use arkit::ohos_arkui_binding::types::attribute::ArkUINodeAttributeType;
use arkit::ohos_arkui_binding::types::curve::Curve;
use arkit::{queue_after_mount, queue_ui_loop, ComponentElement};

#[derive(Debug, Clone, Copy)]
pub struct Motion {
    duration_ms: i32,
    delay_ms: i32,
    iterations: i32,
    tempo: f32,
    curve: Curve,
    mode: AnimationMode,
}

impl Motion {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn duration_ms(mut self, value: i32) -> Self {
        self.duration_ms = value.max(0);
        self
    }

    pub fn delay_ms(mut self, value: i32) -> Self {
        self.delay_ms = value.max(0);
        self
    }

    pub fn iterations(mut self, value: i32) -> Self {
        self.iterations = value;
        self
    }

    pub fn tempo(mut self, value: f32) -> Self {
        self.tempo = value.max(0.0);
        self
    }

    pub fn curve(mut self, value: Curve) -> Self {
        self.curve = value;
        self
    }

    pub fn mode(mut self, value: AnimationMode) -> Self {
        self.mode = value;
        self
    }

    fn build_animation(self) -> Animation {
        let animation = Animation::new();
        animation.duration(self.duration_ms);
        animation.delay(self.delay_ms);
        animation.iterations(self.iterations);
        animation.tempo(self.tempo);
        animation.curve(self.curve);
        animation.mode(self.mode);
        animation
    }
}

impl Default for Motion {
    fn default() -> Self {
        Self {
            duration_ms: 200,
            delay_ms: 0,
            iterations: 1,
            tempo: 1.0,
            curve: Curve::EaseOut,
            mode: AnimationMode::Normal,
        }
    }
}

pub struct ManagedTransition {
    effect: Option<TransitionEffect>,
    animations: Vec<Animation>,
}

impl ManagedTransition {
    pub fn opacity(opacity: f32, motion: Motion) -> ArkUIResult<Self> {
        Self::from_effect(TransitionEffect::opacity(opacity)?, motion)
    }

    pub fn translate(x: f32, y: f32, z: f32, motion: Motion) -> ArkUIResult<Self> {
        Self::from_effect(
            TransitionEffect::translation(TranslationOptions::new(x, y, z))?,
            motion,
        )
    }

    pub fn scale(
        x: f32,
        y: f32,
        z: f32,
        center_x: f32,
        center_y: f32,
        motion: Motion,
    ) -> ArkUIResult<Self> {
        Self::from_effect(
            TransitionEffect::scale(ScaleOptions::new(x, y, z, center_x, center_y))?,
            motion,
        )
    }

    pub fn rotate_z(angle: f32, center_x: f32, center_y: f32, motion: Motion) -> ArkUIResult<Self> {
        Self::from_effect(
            TransitionEffect::rotation(RotationOptions::new(
                0.0, 0.0, 1.0, angle, center_x, center_y, 0.0, 0.0,
            ))?,
            motion,
        )
    }

    pub fn combine(mut self, mut other: Self) -> ArkUIResult<Self> {
        if let (Some(effect), Some(other_effect)) = (self.effect.as_mut(), other.effect.as_ref()) {
            effect.combine(other_effect)?;
        }
        let _ = other.effect.take();
        self.animations.append(&mut other.animations);
        Ok(self)
    }

    pub fn asymmetric(mut appear: Self, mut disappear: Self) -> ArkUIResult<Self> {
        let effect = TransitionEffect::asymmetric(appear.effect(), disappear.effect())?;
        let mut animations = Vec::new();
        animations.append(&mut appear.animations);
        animations.append(&mut disappear.animations);
        let _ = appear.effect.take();
        let _ = disappear.effect.take();
        Ok(Self {
            effect: Some(effect),
            animations,
        })
    }

    fn from_effect(mut effect: TransitionEffect, motion: Motion) -> ArkUIResult<Self> {
        let animation = motion.build_animation();
        effect.set_animation(&animation)?;
        Ok(Self {
            effect: Some(effect),
            animations: vec![animation],
        })
    }

    fn effect(&self) -> &TransitionEffect {
        self.effect
            .as_ref()
            .expect("managed transition effect should exist while attached")
    }
}

impl Drop for ManagedTransition {
    fn drop(&mut self) {
        if let Some(effect) = self.effect.take() {
            effect.dispose();
        }
        self.animations.clear();
    }
}

fn log_animation_error(message: impl AsRef<str>) {
    ohos_hilog_binding::error(message.as_ref());
}

pub trait MotionExt<T>
where
    T: ArkUICommonAttribute + 'static,
{
    fn with_mount_motion(
        self,
        motion: Motion,
        initial: impl Fn(&mut ArkUINode) -> ArkUIResult<()> + 'static,
        target: impl Fn(&mut ArkUINode) -> ArkUIResult<()> + 'static,
    ) -> Self;

    fn with_mount_motion_finish(
        self,
        motion: Motion,
        initial: impl Fn(&mut ArkUINode) -> ArkUIResult<()> + 'static,
        target: impl Fn(&mut ArkUINode) -> ArkUIResult<()> + 'static,
        on_finish: impl Fn() + 'static,
    ) -> Self;
}

impl<T> MotionExt<T> for ComponentElement<T>
where
    T: ArkUICommonAttribute + 'static,
{
    fn with_mount_motion(
        self,
        motion: Motion,
        initial: impl Fn(&mut ArkUINode) -> ArkUIResult<()> + 'static,
        target: impl Fn(&mut ArkUINode) -> ArkUIResult<()> + 'static,
    ) -> Self {
        self.with_mount_motion_finish(motion, initial, target, || {})
    }

    fn with_mount_motion_finish(
        self,
        motion: Motion,
        initial: impl Fn(&mut ArkUINode) -> ArkUIResult<()> + 'static,
        target: impl Fn(&mut ArkUINode) -> ArkUIResult<()> + 'static,
        on_finish: impl Fn() + 'static,
    ) -> Self {
        let initial = Rc::new(initial);
        let target = Rc::new(target);
        let on_finish = Rc::new(on_finish);

        self.native_with_cleanup(move |node| {
            initial(node.borrow_mut())?;
            let animated_node = Rc::new(RefCell::new(node.borrow_mut().clone()));
            let animation_slot = Rc::new(RefCell::new(None::<Animation>));
            let is_active = Rc::new(Cell::new(true));
            let queued_node = animated_node.clone();
            let queued_slot = animation_slot.clone();
            let queued_active = is_active.clone();
            let queued_target = target.clone();
            let queued_finish = on_finish.clone();

            queue_after_mount(move || {
                if !queued_active.get() {
                    return;
                }

                let animation = motion.build_animation();
                let update_node = queued_node.clone();
                let update_target = queued_target.clone();
                let update_active = queued_active.clone();
                animation.update(move || {
                    if !update_active.get() {
                        return;
                    }
                    let mut node = update_node.borrow_mut();
                    if let Err(error) = update_target(&mut node) {
                        log_animation_error(format!(
                            "animation error: failed to apply animated node update: {error}"
                        ));
                    }
                });

                let finish_slot = queued_slot.clone();
                let finish_active = queued_active.clone();
                let finish_callback = queued_finish.clone();
                animation.finish(AnimationFinishCallbackType::Logically, move || {
                    let release_slot = finish_slot.clone();
                    queue_ui_loop(move || {
                        let _ = release_slot.borrow_mut().take();
                    });
                    if finish_active.get() {
                        finish_callback();
                    }
                });

                if !queued_active.get() {
                    return;
                }

                let animation_node = queued_node.borrow().clone();
                if let Err(error) = animation_node.animate_to(&animation) {
                    log_animation_error(format!(
                        "animation error: failed to start mount animation: {error}"
                    ));
                    return;
                }

                *queued_slot.borrow_mut() = Some(animation);
            });

            Ok(move || {
                is_active.set(false);
            })
        })
    }
}

pub trait TransitionExt<T>
where
    T: ArkUICommonAttribute + 'static,
{
    fn with_transition_attr(
        self,
        attr: ArkUINodeAttributeType,
        transition: ManagedTransition,
    ) -> Self;

    fn with_transition(self, transition: ManagedTransition) -> Self;

    fn with_opacity_transition(self, transition: ManagedTransition) -> Self;

    fn with_rotate_transition(self, transition: ManagedTransition) -> Self;

    fn with_translate_transition(self, transition: ManagedTransition) -> Self;
}

impl<T> TransitionExt<T> for ComponentElement<T>
where
    T: ArkUICommonAttribute + 'static,
{
    fn with_transition_attr(
        self,
        attr: ArkUINodeAttributeType,
        transition: ManagedTransition,
    ) -> Self {
        self.native_with_cleanup(move |node| {
            node.set_attribute(attr, transition.effect().into())?;
            Ok(move || drop(transition))
        })
    }

    fn with_transition(self, transition: ManagedTransition) -> Self {
        self.with_transition_attr(ArkUINodeAttributeType::Transition, transition)
    }

    fn with_opacity_transition(self, transition: ManagedTransition) -> Self {
        self.with_transition(transition)
    }

    fn with_rotate_transition(self, transition: ManagedTransition) -> Self {
        self.with_transition(transition)
    }

    fn with_translate_transition(self, transition: ManagedTransition) -> Self {
        self.with_transition(transition)
    }
}
