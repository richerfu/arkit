use std::error::Error;
use std::fmt::{Display, Formatter};

use arkit_animation_core::{
    CompiledAnimation, Composition, Easing, InvalidationClass, IterationCount, Modifier,
    NativeSupport, TimeExtent,
};

use crate::{AnimationBackend, CapabilityRequirements, ExecutionPolicy, NativeCapability};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnsupportedFeature {
    Seek,
    Pause,
    Resume,
    Reverse,
    Cancel,
    Alternate,
    Callbacks,
    PerPropertyTiming,
    Composition,
    DynamicModifier,
    InfiniteIterations,
    LayoutInvalidation,
    CustomEasing,
    Property,
    BackendUnavailable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendRejection {
    pub backend: AnimationBackend,
    pub unsupported: Vec<UnsupportedFeature>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoweringReport {
    pub requested: ExecutionPolicy,
    pub selected: AnimationBackend,
    pub rejected_native: Vec<AnimationBackend>,
    pub rejections: Vec<BackendRejection>,
    pub fallback_reason: Option<Box<str>>,
    pub target_count: usize,
    pub property_count: usize,
    pub tween_count: usize,
    pub layout_property_count: usize,
    pub estimated_per_frame_work: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NativeLoweringError {
    NativeOnlyUnsupported {
        requirements: CapabilityRequirements,
        rejections: Vec<BackendRejection>,
    },
    BackendUnavailable {
        backend: AnimationBackend,
    },
}

impl Display for NativeLoweringError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl Error for NativeLoweringError {}

#[derive(Debug, Clone, Copy, Default)]
pub struct NativeLowerer;

impl NativeLowerer {
    pub fn lower(
        self,
        policy: ExecutionPolicy,
        requirements: CapabilityRequirements,
    ) -> Result<LoweringReport, NativeLoweringError> {
        self.select(policy, requirements, |_| true, PlanCounts::default())
    }

    pub fn lower_plan(
        self,
        policy: ExecutionPolicy,
        plan: &CompiledAnimation,
        controls: CapabilityRequirements,
    ) -> Result<LoweringReport, NativeLoweringError> {
        let requirements = merge_requirements(derive_requirements(plan), controls);
        let counts = PlanCounts {
            targets: plan.targets().len(),
            properties: plan.properties().len(),
            tweens: plan.tweens().len(),
            layout_properties: plan
                .properties()
                .iter()
                .filter(|property| {
                    matches!(
                        property.descriptor.invalidation,
                        InvalidationClass::Layout | InvalidationClass::Measure
                    )
                })
                .count(),
            outputs: plan.outputs().len(),
        };
        self.select(
            policy,
            requirements,
            |backend| {
                plan.properties().iter().all(|property| {
                    property_supported(property.descriptor.native, backend)
                        || backend == AnimationBackend::ArkUiAnimator
                })
            },
            counts,
        )
    }

    fn select(
        self,
        policy: ExecutionPolicy,
        requirements: CapabilityRequirements,
        properties_supported: impl Fn(AnimationBackend) -> bool,
        counts: PlanCounts,
    ) -> Result<LoweringReport, NativeLoweringError> {
        if policy == ExecutionPolicy::SampledOnly {
            return Ok(report(
                policy,
                AnimationBackend::Sampled,
                Vec::new(),
                None,
                counts,
            ));
        }
        let native = match policy {
            ExecutionPolicy::Auto
            | ExecutionPolicy::NativePreferred
            | ExecutionPolicy::NativeOnly => [
                NativeCapability::ARKUI_ANIMATOR,
                NativeCapability::ARKUI_KEYFRAME,
                NativeCapability::ARKUI_IMPLICIT,
            ],
            ExecutionPolicy::SampledOnly => unreachable!(),
        };
        let mut rejections = Vec::new();
        for capability in native {
            let mut unsupported = unsupported(capability, requirements);
            if !properties_supported(capability.backend) {
                unsupported.push(UnsupportedFeature::Property);
            }
            if unsupported.is_empty() {
                return Ok(report(policy, capability.backend, rejections, None, counts));
            }
            rejections.push(BackendRejection {
                backend: capability.backend,
                unsupported,
            });
        }
        if policy == ExecutionPolicy::NativeOnly {
            return Err(NativeLoweringError::NativeOnlyUnsupported {
                requirements,
                rejections,
            });
        }
        Ok(report(
            policy,
            AnimationBackend::Sampled,
            rejections,
            Some("native backends cannot preserve requested semantics".into()),
            counts,
        ))
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct PlanCounts {
    targets: usize,
    properties: usize,
    tweens: usize,
    layout_properties: usize,
    outputs: usize,
}

fn report(
    requested: ExecutionPolicy,
    selected: AnimationBackend,
    rejections: Vec<BackendRejection>,
    fallback_reason: Option<Box<str>>,
    counts: PlanCounts,
) -> LoweringReport {
    LoweringReport {
        requested,
        selected,
        rejected_native: rejections
            .iter()
            .map(|rejection| rejection.backend)
            .collect(),
        rejections,
        fallback_reason,
        target_count: counts.targets,
        property_count: counts.properties,
        tween_count: counts.tweens,
        layout_property_count: counts.layout_properties,
        estimated_per_frame_work: counts.tweens.saturating_add(counts.outputs),
    }
}

fn derive_requirements(plan: &CompiledAnimation) -> CapabilityRequirements {
    CapabilityRequirements {
        alternate: plan.settings().alternate,
        per_property_timing: plan.tracks().iter().any(|track| track.tweens().len() > 1),
        composition: plan
            .tweens()
            .iter()
            .any(|tween| tween.composition != Composition::Replace),
        dynamic_modifier: plan
            .tweens()
            .iter()
            .any(|tween| !matches!(tween.modifier, Modifier::Identity)),
        infinite: matches!(plan.extent(), TimeExtent::Infinite)
            || matches!(plan.settings().iterations, IterationCount::Infinite),
        layout_invalidation: plan.properties().iter().any(|property| {
            matches!(
                property.descriptor.invalidation,
                InvalidationClass::Layout | InvalidationClass::Measure
            )
        }),
        custom_easing: plan
            .tweens()
            .iter()
            .any(|tween| matches!(tween.easing, Easing::Custom { .. })),
        callbacks: !plan.events().is_empty(),
        ..CapabilityRequirements::default()
    }
}

fn merge_requirements(
    left: CapabilityRequirements,
    right: CapabilityRequirements,
) -> CapabilityRequirements {
    CapabilityRequirements {
        seek: left.seek || right.seek,
        pause: left.pause || right.pause,
        resume: left.resume || right.resume,
        reverse: left.reverse || right.reverse,
        cancel: left.cancel || right.cancel,
        alternate: left.alternate || right.alternate,
        callbacks: left.callbacks || right.callbacks,
        per_property_timing: left.per_property_timing || right.per_property_timing,
        composition: left.composition || right.composition,
        dynamic_modifier: left.dynamic_modifier || right.dynamic_modifier,
        infinite: left.infinite || right.infinite,
        layout_invalidation: left.layout_invalidation || right.layout_invalidation,
        custom_easing: left.custom_easing || right.custom_easing,
    }
}

fn property_supported(support: NativeSupport, backend: AnimationBackend) -> bool {
    match backend {
        AnimationBackend::Sampled => true,
        AnimationBackend::ArkUiImplicit => support.implicit,
        AnimationBackend::ArkUiKeyframe => support.keyframe,
        AnimationBackend::ArkUiAnimator => support.animator,
    }
}

fn unsupported(
    capability: NativeCapability,
    requirements: CapabilityRequirements,
) -> Vec<UnsupportedFeature> {
    let mut output = Vec::new();
    let checks = [
        (
            requirements.seek && !capability.seek,
            UnsupportedFeature::Seek,
        ),
        (
            requirements.pause && !capability.pause,
            UnsupportedFeature::Pause,
        ),
        (
            requirements.resume && !capability.resume,
            UnsupportedFeature::Resume,
        ),
        (
            requirements.reverse && !capability.reverse,
            UnsupportedFeature::Reverse,
        ),
        (
            requirements.cancel && !capability.cancel,
            UnsupportedFeature::Cancel,
        ),
        (
            requirements.alternate && !capability.alternate,
            UnsupportedFeature::Alternate,
        ),
        (
            requirements.callbacks && !capability.callbacks,
            UnsupportedFeature::Callbacks,
        ),
        (
            requirements.per_property_timing && !capability.per_property_timing,
            UnsupportedFeature::PerPropertyTiming,
        ),
        (
            requirements.composition && !capability.composition,
            UnsupportedFeature::Composition,
        ),
        (
            requirements.dynamic_modifier && !capability.dynamic_modifier,
            UnsupportedFeature::DynamicModifier,
        ),
        (
            requirements.infinite && !capability.infinite,
            UnsupportedFeature::InfiniteIterations,
        ),
        (
            requirements.layout_invalidation && !capability.layout_invalidation,
            UnsupportedFeature::LayoutInvalidation,
        ),
        (
            requirements.custom_easing && !capability.custom_easing,
            UnsupportedFeature::CustomEasing,
        ),
    ];
    output.extend(
        checks
            .into_iter()
            .filter_map(|(missing, feature)| missing.then_some(feature)),
    );
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn composition_forces_a_reported_sampled_fallback() {
        let report = NativeLowerer
            .lower(
                ExecutionPolicy::Auto,
                CapabilityRequirements {
                    composition: true,
                    ..CapabilityRequirements::default()
                },
            )
            .unwrap();
        assert_eq!(report.selected, AnimationBackend::Sampled);
        assert!(report.fallback_reason.is_some());
        assert!(report.rejections.iter().all(|rejection| {
            rejection
                .unsupported
                .contains(&UnsupportedFeature::Composition)
        }));
    }

    #[test]
    fn native_only_rejects_unrepresentable_controls() {
        assert!(NativeLowerer
            .lower(
                ExecutionPolicy::NativeOnly,
                CapabilityRequirements {
                    dynamic_modifier: true,
                    ..CapabilityRequirements::default()
                },
            )
            .is_err());
    }
}
