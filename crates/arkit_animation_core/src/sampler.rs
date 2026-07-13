//! Allocation-free sampling and composition of one precompiled property track.

use crate::{
    AnimationSampleError, AnimationValue, CompiledAnimation, CompiledTrack, TimePoint,
    TrackSegmentId, TweenId,
};

#[derive(Debug, Clone, Copy)]
pub struct TrackSampleContext {
    pub local_time: TimePoint,
    pub completed_iterations: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SampledReplace {
    pub tween: TweenId,
    pub value: AnimationValue,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SampledTrack {
    pub replace: Option<SampledReplace>,
    pub additive: Option<AnimationValue>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct AnimationSampler;

impl AnimationSampler {
    pub fn sample_track(
        plan: &CompiledAnimation,
        track: &CompiledTrack,
        segment: TrackSegmentId,
        context: TrackSampleContext,
    ) -> Result<SampledTrack, AnimationSampleError> {
        let segment = &track.segments()[segment];
        let replace = match segment.replace {
            Some(tween) => Some(SampledReplace {
                tween,
                value: sample_tween(plan, tween, context.local_time)?,
            }),
            None => None,
        };
        let mut additive: Option<AnimationValue> = None;

        for tween in segment.additive().iter().copied() {
            let contribution = sample_tween(plan, tween, context.local_time)?;
            additive = Some(match additive {
                Some(value) => value
                    .compose_add(&contribution)
                    .map_err(|source| AnimationSampleError::Value { tween, source })?,
                None => contribution,
            });
        }
        for tween in segment.accumulating().iter().copied() {
            let compiled = &plan.tweens()[tween];
            let current = sample_tween(plan, tween, context.local_time)?;
            let delta = compiled
                .to
                .delta_from(&compiled.from)
                .and_then(|delta| delta.scale(context.completed_iterations as f32))
                .and_then(|accumulated| current.compose_add(&accumulated))
                .map_err(|source| AnimationSampleError::Value { tween, source })?;
            additive = Some(match additive {
                Some(value) => value
                    .compose_add(&delta)
                    .map_err(|source| AnimationSampleError::Value { tween, source })?,
                None => delta,
            });
        }
        Ok(SampledTrack { replace, additive })
    }
}

fn sample_tween(
    plan: &CompiledAnimation,
    tween_id: TweenId,
    local_time: TimePoint,
) -> Result<AnimationValue, AnimationSampleError> {
    let tween = &plan.tweens()[tween_id];
    let progress = if local_time <= tween.start {
        0.0
    } else if local_time >= tween.end || tween.start == tween.end {
        1.0
    } else {
        let elapsed = local_time - tween.start;
        elapsed.as_nanos() as f32 / tween.duration().as_nanos() as f32
    };
    let eased = tween.easing.sample(progress);
    let value =
        tween
            .from
            .interpolate(&tween.to, eased)
            .map_err(|source| AnimationSampleError::Value {
                tween: tween_id,
                source,
            })?;
    tween
        .modifier
        .apply(value)
        .map_err(|source| AnimationSampleError::Modifier {
            tween: tween_id,
            source,
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AdapterId, AdapterPropertyId, AdapterTargetId, AnimationCompiler, Composition,
        CompositionSupport, Easing, Modifier, Property, PropertyDescriptor, ResolvedAnimation,
        ResolvedProperty, ResolvedTarget, ResolvedTween, TargetId, TimeDomainId, TimeSpan,
    };

    const VALUE: Property<f32> = Property::static_name("value");

    fn plan() -> std::sync::Arc<CompiledAnimation> {
        let mut resolved = ResolvedAnimation::default();
        resolved.targets.push(ResolvedTarget {
            adapter: AdapterId::new(0),
            adapter_target: AdapterTargetId::new(0),
        });
        let mut descriptor = PropertyDescriptor::new(&VALUE);
        descriptor.composition = CompositionSupport::NUMERIC;
        resolved.properties.push(ResolvedProperty {
            adapter: AdapterId::new(0),
            adapter_property: AdapterPropertyId::new(0),
            descriptor,
        });
        let tween = |from, to, composition| ResolvedTween {
            domain: TimeDomainId::new(0),
            target: TargetId::new(0),
            property: crate::PropertyId::new(0),
            start: TimePoint::ZERO,
            delay: TimeSpan::ZERO,
            duration: TimeSpan::from_nanos(100),
            priority: 0,
            from: AnimationValue::Scalar(from),
            to: AnimationValue::Scalar(to),
            easing: Easing::Linear,
            composition,
            modifier: Modifier::Identity,
        };
        resolved.tweens.push(tween(0.0, 10.0, Composition::Replace));
        resolved.tweens.push(tween(0.0, 2.0, Composition::Add));
        resolved
            .tweens
            .push(tween(0.0, 1.0, Composition::Accumulate));
        AnimationCompiler.compile(resolved).unwrap()
    }

    #[test]
    fn sampler_composes_replace_add_and_accumulate_without_allocating_scratch() {
        let plan = plan();
        let track = &plan.tracks()[crate::TrackId::new(0)];
        let segment = track.seek_segment(TimePoint::from_nanos(50)).unwrap();
        let sampled = AnimationSampler::sample_track(
            &plan,
            track,
            segment,
            TrackSampleContext {
                local_time: TimePoint::from_nanos(50),
                completed_iterations: 2,
            },
        )
        .unwrap();
        let output = sampled
            .replace
            .unwrap()
            .value
            .compose_add(&sampled.additive.unwrap())
            .unwrap();
        assert_eq!(output, AnimationValue::Scalar(8.5));
    }

    #[test]
    fn arithmetic_composition_rejects_discrete_values() {
        let value = AnimationValue::Discrete(crate::DiscreteValue::new("a"));
        assert!(value.compose_add(&value).is_err());
    }
}
