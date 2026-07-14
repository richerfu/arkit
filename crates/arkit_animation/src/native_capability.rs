#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AnimationBackend {
    Sampled,
    ArkUiImplicit,
    ArkUiKeyframe,
    ArkUiAnimator,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExecutionPolicy {
    Auto,
    SampledOnly,
    NativePreferred,
    NativeOnly,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CapabilityRequirements {
    pub seek: bool,
    pub pause: bool,
    pub resume: bool,
    pub reverse: bool,
    pub cancel: bool,
    pub alternate: bool,
    pub callbacks: bool,
    pub per_property_timing: bool,
    pub composition: bool,
    pub dynamic_modifier: bool,
    pub infinite: bool,
    pub layout_invalidation: bool,
    pub custom_easing: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NativeCapability {
    pub backend: AnimationBackend,
    pub seek: bool,
    pub pause: bool,
    pub resume: bool,
    pub reverse: bool,
    pub cancel: bool,
    pub alternate: bool,
    pub callbacks: bool,
    pub per_property_timing: bool,
    pub composition: bool,
    pub dynamic_modifier: bool,
    pub infinite: bool,
    pub layout_invalidation: bool,
    pub custom_easing: bool,
}

impl NativeCapability {
    pub const SAMPLED: Self = Self {
        backend: AnimationBackend::Sampled,
        seek: true,
        pause: true,
        resume: true,
        reverse: true,
        cancel: true,
        alternate: true,
        callbacks: true,
        per_property_timing: true,
        composition: true,
        dynamic_modifier: true,
        infinite: true,
        layout_invalidation: true,
        custom_easing: true,
    };

    pub const ARKUI_IMPLICIT: Self = Self {
        backend: AnimationBackend::ArkUiImplicit,
        seek: false,
        pause: false,
        resume: false,
        reverse: false,
        cancel: false,
        alternate: false,
        callbacks: false,
        per_property_timing: false,
        composition: false,
        dynamic_modifier: false,
        infinite: false,
        layout_invalidation: true,
        custom_easing: false,
    };

    pub const ARKUI_KEYFRAME: Self = Self {
        backend: AnimationBackend::ArkUiKeyframe,
        seek: false,
        pause: false,
        resume: false,
        reverse: false,
        cancel: false,
        alternate: true,
        callbacks: true,
        per_property_timing: true,
        composition: false,
        dynamic_modifier: false,
        infinite: true,
        layout_invalidation: true,
        custom_easing: false,
    };

    pub const ARKUI_ANIMATOR: Self = Self {
        backend: AnimationBackend::ArkUiAnimator,
        seek: false,
        pause: true,
        resume: true,
        // ArkUI can reverse a running Animator, but cannot preserve every
        // idle/paused/replay transition in the engine contract. Selection is
        // therefore conservative; an undeclared running reverse can still use
        // the native fast path and other states fall back atomically.
        reverse: false,
        cancel: true,
        alternate: true,
        callbacks: true,
        // The native Animator owns the root clock; compiled per-property
        // sampling and writes remain in the engine, so these semantics are
        // preserved without asking ArkUI Animator to represent each tween.
        per_property_timing: true,
        composition: true,
        dynamic_modifier: true,
        infinite: false,
        layout_invalidation: true,
        custom_easing: true,
    };

    pub const fn supports(self, requirements: CapabilityRequirements) -> bool {
        (!requirements.seek || self.seek)
            && (!requirements.pause || self.pause)
            && (!requirements.resume || self.resume)
            && (!requirements.reverse || self.reverse)
            && (!requirements.cancel || self.cancel)
            && (!requirements.alternate || self.alternate)
            && (!requirements.callbacks || self.callbacks)
            && (!requirements.per_property_timing || self.per_property_timing)
            && (!requirements.composition || self.composition)
            && (!requirements.dynamic_modifier || self.dynamic_modifier)
            && (!requirements.infinite || self.infinite)
            && (!requirements.layout_invalidation || self.layout_invalidation)
            && (!requirements.custom_easing || self.custom_easing)
    }
}
