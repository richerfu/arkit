//! Adapter-snapshot resolution from symbolic targets and values to dense plan slots.

use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    AdapterId, AdapterPropertyId, AdapterTargetId, AnimationResolveError, AnimationValue,
    FromValue, IterationCount, LabelName, PlaybackSettings, PropertyId, PropertyName,
    ResolvedAnimation, ResolvedEvent, ResolvedProperty, ResolvedTarget, ResolvedTimeDomain,
    ResolvedTween, SourceAnimation, SourceSet, SourceTarget, TargetId, TargetName, TimeDomainId,
    TimeExtent, TimePoint, TimeSpan, TimelineNode, TimelinePosition, TimelineSource,
    ValueFunctionName, ValueSource,
};

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct WindowMetrics {
    pub width_vp: f32,
    pub height_vp: f32,
    pub density: f32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct TargetLayoutSnapshot {
    pub x_vp: f32,
    pub y_vp: f32,
    pub width_vp: f32,
    pub height_vp: f32,
}

#[derive(Debug, Clone)]
pub struct ResolutionTarget {
    pub name: TargetName,
    pub target: ResolvedTarget,
    pub layout: Option<TargetLayoutSnapshot>,
}

#[derive(Debug, Clone, Copy)]
pub struct TargetContext<'a> {
    pub index: usize,
    pub total: usize,
    pub target_name: &'a TargetName,
    pub layout_snapshot: Option<&'a TargetLayoutSnapshot>,
    pub window_metrics: WindowMetrics,
}

pub trait ResolutionContext {
    fn resolve_targets(
        &self,
        target: &SourceTarget,
    ) -> Result<Box<[ResolutionTarget]>, AnimationResolveError>;

    fn resolve_property(
        &self,
        target: &ResolvedTarget,
        property: &PropertyName,
    ) -> Result<ResolvedProperty, AnimationResolveError>;

    fn read_baseline(
        &self,
        target: &ResolvedTarget,
        property: &ResolvedProperty,
    ) -> Result<AnimationValue, AnimationResolveError>;

    fn resolve_value(
        &self,
        _target: &ResolvedTarget,
        _property: &ResolvedProperty,
        value: &AnimationValue,
    ) -> Result<AnimationValue, AnimationResolveError> {
        Ok(value.clone())
    }

    fn resolve_relative(
        &self,
        target: &ResolvedTarget,
        property: &ResolvedProperty,
        baseline: &AnimationValue,
        delta: &AnimationValue,
    ) -> Result<AnimationValue, AnimationResolveError>;

    fn resolve_function(
        &self,
        function: &ValueFunctionName,
        target: &ResolvedTarget,
        property: &ResolvedProperty,
        context: TargetContext<'_>,
    ) -> Result<AnimationValue, AnimationResolveError>;

    fn window_metrics(&self) -> WindowMetrics;
}

pub struct AnimationResolver<'context, Context: ResolutionContext + ?Sized> {
    context: &'context Context,
}

impl<'context, Context: ResolutionContext + ?Sized> AnimationResolver<'context, Context> {
    pub const fn new(context: &'context Context) -> Self {
        Self { context }
    }

    pub fn resolve_animation(
        &self,
        source: &SourceAnimation,
        start: TimePoint,
        settings: PlaybackSettings,
    ) -> Result<ResolvedAnimation, AnimationResolveError> {
        let mut state = ResolverState::new(settings);
        self.resolve_animation_into(&mut state, TimeDomainId::new(0), source, start)?;
        let extent = source_animation_duration(source)?;
        state.animation.domains[TimeDomainId::new(0)].extent = TimeExtent::Finite(extent);
        Ok(state.animation)
    }

    pub fn resolve_timeline(
        &self,
        source: &TimelineSource,
    ) -> Result<ResolvedAnimation, AnimationResolveError> {
        let mut state = ResolverState::new(source.settings.clone());
        let extent = self.resolve_timeline_into(&mut state, TimeDomainId::new(0), source)?;
        state.animation.domains[TimeDomainId::new(0)].extent = extent;
        Ok(state.animation)
    }

    fn resolve_timeline_into(
        &self,
        state: &mut ResolverState,
        domain: TimeDomainId,
        source: &TimelineSource,
    ) -> Result<TimeExtent, AnimationResolveError> {
        let mut position_state = PositionState::default();
        for node in &source.nodes {
            match node {
                TimelineNode::Animation {
                    animation,
                    position,
                } => {
                    let start = position_state.resolve(position)?;
                    self.resolve_animation_into(state, domain, animation, start)?;
                    let duration = source_animation_duration(animation)?;
                    let end = start
                        .checked_add(duration)
                        .ok_or(AnimationResolveError::TimeOverflow)?;
                    position_state.commit_node(
                        start,
                        TimeExtent::Finite(TimeSpan::from_nanos(end.as_nanos())),
                    );
                }
                TimelineNode::Timer { duration, position } => {
                    let start = position_state.resolve(position)?;
                    let end = start
                        .checked_add(*duration)
                        .ok_or(AnimationResolveError::TimeOverflow)?;
                    position_state.commit_node(
                        start,
                        TimeExtent::Finite(TimeSpan::from_nanos(end.as_nanos())),
                    );
                }
                TimelineNode::Set(set) => {
                    let at = position_state.resolve(&set.position)?;
                    self.resolve_set_into(state, domain, set, at)?;
                    position_state
                        .commit_node(at, TimeExtent::Finite(TimeSpan::from_nanos(at.as_nanos())));
                }
                TimelineNode::Call {
                    call,
                    policy,
                    position,
                } => {
                    let at = position_state.resolve(position)?;
                    state.animation.events.push(ResolvedEvent::Call {
                        domain,
                        at,
                        call: *call,
                        policy: *policy,
                    });
                    position_state
                        .commit_node(at, TimeExtent::Finite(TimeSpan::from_nanos(at.as_nanos())));
                }
                TimelineNode::Nested { timeline, position } => {
                    let start = position_state.resolve(position)?;
                    let child_domain = state.animation.domains.push(ResolvedTimeDomain {
                        parent: Some(domain),
                        offset: start,
                        extent: TimeExtent::ZERO,
                        settings: timeline.settings.clone(),
                    });
                    let child_extent = self.resolve_timeline_into(state, child_domain, timeline)?;
                    state.animation.domains[child_domain].extent = child_extent;
                    let occupied = domain_parent_extent(child_extent, &timeline.settings)?;
                    let end = add_extent(start, occupied)?;
                    position_state.commit_node(start, end);
                }
                TimelineNode::Label { name, position } => {
                    let at = position_state.resolve(position)?;
                    position_state.insert_label(name.clone(), at)?;
                }
                TimelineNode::Barrier {
                    participants,
                    position,
                } => {
                    let at = position_state.resolve(position)?;
                    state.animation.events.push(ResolvedEvent::Barrier {
                        domain,
                        at,
                        participants: *participants,
                    });
                    position_state
                        .commit_node(at, TimeExtent::Finite(TimeSpan::from_nanos(at.as_nanos())));
                }
            }
        }
        Ok(position_state.timeline_end)
    }

    fn resolve_animation_into(
        &self,
        state: &mut ResolverState,
        domain: TimeDomainId,
        source: &SourceAnimation,
        start: TimePoint,
    ) -> Result<(), AnimationResolveError> {
        let selected = self.context.resolve_targets(&source.target)?;
        if selected.is_empty() {
            return Err(AnimationResolveError::EmptyTargetSelection);
        }

        validate_unique_targets(&selected)?;

        let total = selected.len();
        let window_metrics = self.context.window_metrics();
        for (index, selected_target) in selected.iter().enumerate() {
            let target_id = state.intern_target(selected_target.target);
            let target_context = TargetContext {
                index,
                total,
                target_name: &selected_target.name,
                layout_snapshot: selected_target.layout.as_ref(),
                window_metrics,
            };
            for tween in &source.tweens {
                self.resolve_tween(
                    state,
                    TweenResolution {
                        domain,
                        selected_target,
                        target_context,
                        target_id,
                        tween,
                        start,
                    },
                )?;
            }
        }
        Ok(())
    }

    fn resolve_set_into(
        &self,
        state: &mut ResolverState,
        domain: TimeDomainId,
        source: &SourceSet,
        at: TimePoint,
    ) -> Result<(), AnimationResolveError> {
        let selected = self.context.resolve_targets(&source.target)?;
        if selected.is_empty() {
            return Err(AnimationResolveError::EmptyTargetSelection);
        }
        validate_unique_targets(&selected)?;
        for selected_target in selected.iter() {
            let target_id = state.intern_target(selected_target.target);
            let property = self
                .context
                .resolve_property(&selected_target.target, &source.property)?;
            let property_id = state.intern_property(property.clone())?;
            let value =
                self.context
                    .resolve_value(&selected_target.target, &property, &source.value)?;
            property
                .descriptor
                .validate_value(&value)
                .map_err(|source_error| AnimationResolveError::Value {
                    property: source.property.clone(),
                    source: source_error,
                })?;
            state.animation.events.push(ResolvedEvent::Set {
                domain,
                at,
                target: target_id,
                property: property_id,
                value,
            });
        }
        Ok(())
    }

    fn resolve_tween(
        &self,
        state: &mut ResolverState,
        input: TweenResolution<'_>,
    ) -> Result<(), AnimationResolveError> {
        let TweenResolution {
            domain,
            selected_target,
            target_context,
            target_id,
            tween,
            start,
        } = input;
        let property = self
            .context
            .resolve_property(&selected_target.target, &tween.property)?;
        let property_id = state.intern_property(property.clone())?;
        let track_key = (target_id, property_id);
        let previous_key = (domain, target_id, property_id);
        let from = match &tween.from {
            FromValue::Explicit(value) => {
                self.context
                    .resolve_value(&selected_target.target, &property, value)?
            }
            FromValue::Current => {
                state.baseline(self.context, track_key, &selected_target.target, &property)?
            }
            FromValue::Previous => match state.previous_values.get(&previous_key).cloned() {
                Some(previous) => previous,
                None => {
                    state.baseline(self.context, track_key, &selected_target.target, &property)?
                }
            },
            FromValue::RelativeBaseline(delta) => {
                let baseline =
                    state.baseline(self.context, track_key, &selected_target.target, &property)?;
                self.context.resolve_relative(
                    &selected_target.target,
                    &property,
                    &baseline,
                    delta,
                )?
            }
        };
        let to = match &tween.to {
            ValueSource::Fixed(value) => {
                self.context
                    .resolve_value(&selected_target.target, &property, value)?
            }
            ValueSource::Relative(delta) => {
                self.context
                    .resolve_relative(&selected_target.target, &property, &from, delta)?
            }
            ValueSource::Function(function) => {
                let value = self.context.resolve_function(
                    function,
                    &selected_target.target,
                    &property,
                    target_context,
                )?;
                self.context
                    .resolve_value(&selected_target.target, &property, &value)?
            }
        };
        property
            .descriptor
            .validate_value(&from)
            .and_then(|()| property.descriptor.validate_value(&to))
            .map_err(|source| AnimationResolveError::Value {
                property: tween.property.clone(),
                source,
            })?;
        state.previous_values.insert(previous_key, to.clone());
        state.animation.tweens.push(ResolvedTween {
            domain,
            target: target_id,
            property: property_id,
            start,
            delay: tween.delay,
            duration: tween.duration,
            priority: tween.priority,
            from,
            to,
            easing: tween.easing.clone(),
            composition: tween.composition,
            modifier: tween.modifier.clone(),
        });
        Ok(())
    }
}

struct TweenResolution<'a> {
    domain: TimeDomainId,
    selected_target: &'a ResolutionTarget,
    target_context: TargetContext<'a>,
    target_id: TargetId,
    tween: &'a crate::TweenSpec,
    start: TimePoint,
}

fn validate_unique_targets(selected: &[ResolutionTarget]) -> Result<(), AnimationResolveError> {
    let mut selected_keys = FxHashSet::default();
    for target in selected {
        let key = (target.target.adapter, target.target.adapter_target);
        if !selected_keys.insert(key) {
            return Err(AnimationResolveError::DuplicateTarget {
                adapter: key.0,
                target: key.1,
            });
        }
    }
    Ok(())
}

fn source_animation_duration(
    animation: &SourceAnimation,
) -> Result<TimeSpan, AnimationResolveError> {
    animation
        .tweens
        .iter()
        .map(|tween| {
            tween
                .delay
                .checked_add(tween.duration)
                .ok_or(AnimationResolveError::TimeOverflow)
        })
        .try_fold(TimeSpan::ZERO, |duration, tween| {
            tween.map(|tween| duration.max(tween))
        })
}

fn domain_parent_extent(
    active: TimeExtent,
    settings: &PlaybackSettings,
) -> Result<TimeExtent, AnimationResolveError> {
    let active = match active {
        TimeExtent::Infinite => return Ok(TimeExtent::Infinite),
        TimeExtent::Finite(active) => active,
    };
    let iterations = match settings.iterations {
        IterationCount::Infinite => return Ok(TimeExtent::Infinite),
        IterationCount::Finite(iterations) => iterations.get(),
    };
    let scaled_nanos = active.as_nanos() as f64 / settings.playback_rate.get();
    if !scaled_nanos.is_finite() || scaled_nanos > u64::MAX as f64 {
        return Err(AnimationResolveError::TimeOverflow);
    }
    let iteration_duration = TimeSpan::from_nanos(scaled_nanos.round() as u64);
    let active_total = iteration_duration
        .checked_mul(iterations)
        .ok_or(AnimationResolveError::TimeOverflow)?;
    let loop_total = settings
        .loop_delay
        .checked_mul(iterations.saturating_sub(1))
        .ok_or(AnimationResolveError::TimeOverflow)?;
    let occupied = settings
        .delay
        .checked_add(active_total)
        .and_then(|duration| duration.checked_add(loop_total))
        .ok_or(AnimationResolveError::TimeOverflow)?;
    Ok(TimeExtent::Finite(occupied))
}

fn add_extent(start: TimePoint, duration: TimeExtent) -> Result<TimeExtent, AnimationResolveError> {
    match duration {
        TimeExtent::Infinite => Ok(TimeExtent::Infinite),
        TimeExtent::Finite(duration) => start
            .checked_add(duration)
            .map(|end| TimeExtent::Finite(TimeSpan::from_nanos(end.as_nanos())))
            .ok_or(AnimationResolveError::TimeOverflow),
    }
}

#[derive(Debug, Clone, Copy)]
struct NodeRange {
    start: TimePoint,
    end: TimeExtent,
}

#[derive(Default)]
struct PositionState {
    labels: FxHashMap<LabelName, TimePoint>,
    previous: Option<NodeRange>,
    timeline_end: TimeExtent,
}

impl PositionState {
    fn resolve(&self, position: &TimelinePosition) -> Result<TimePoint, AnimationResolveError> {
        match position {
            TimelinePosition::Absolute(at) => Ok(*at),
            TimelinePosition::Label { label, offset } => {
                let at = self
                    .labels
                    .get(label)
                    .copied()
                    .ok_or_else(|| AnimationResolveError::UnknownLabel(label.clone()))?;
                offset
                    .checked_apply(at)
                    .ok_or(AnimationResolveError::TimeOverflow)
            }
            TimelinePosition::PreviousStart(offset) => offset
                .checked_apply(self.previous.map_or(TimePoint::ZERO, |range| range.start))
                .ok_or(AnimationResolveError::TimeOverflow),
            TimelinePosition::PreviousEnd(offset) => offset
                .checked_apply(match self.previous.map(|range| range.end) {
                    None => TimePoint::ZERO,
                    Some(TimeExtent::Finite(end)) => TimePoint::from_nanos(end.as_nanos()),
                    Some(TimeExtent::Infinite) => {
                        return Err(AnimationResolveError::PositionAfterInfiniteTimeline);
                    }
                })
                .ok_or(AnimationResolveError::TimeOverflow),
            TimelinePosition::TimelineEnd(offset) => match self.timeline_end {
                TimeExtent::Finite(end) => offset
                    .checked_apply(TimePoint::from_nanos(end.as_nanos()))
                    .ok_or(AnimationResolveError::TimeOverflow),
                TimeExtent::Infinite => Err(AnimationResolveError::PositionAfterInfiniteTimeline),
            },
            TimelinePosition::Percentage(percentage) => {
                if !percentage.is_finite() || !(0.0..=1.0).contains(percentage) {
                    return Err(AnimationResolveError::InvalidPercentage(*percentage));
                }
                let TimeExtent::Finite(timeline_end) = self.timeline_end else {
                    return Err(AnimationResolveError::PositionAfterInfiniteTimeline);
                };
                let nanos = (timeline_end.as_nanos() as f64 * f64::from(*percentage)).round();
                Ok(TimePoint::from_nanos(nanos as u64))
            }
        }
    }

    fn commit_node(&mut self, start: TimePoint, end: TimeExtent) {
        self.previous = Some(NodeRange { start, end });
        self.timeline_end = self.timeline_end.max(end);
    }

    fn insert_label(
        &mut self,
        label: LabelName,
        at: TimePoint,
    ) -> Result<(), AnimationResolveError> {
        if self.labels.insert(label.clone(), at).is_some() {
            Err(AnimationResolveError::DuplicateLabel(label))
        } else {
            Ok(())
        }
    }
}

struct ResolverState {
    animation: ResolvedAnimation,
    targets: FxHashMap<(AdapterId, AdapterTargetId), TargetId>,
    properties: FxHashMap<(AdapterId, AdapterPropertyId), PropertyId>,
    baselines: FxHashMap<(TargetId, PropertyId), AnimationValue>,
    previous_values: FxHashMap<(TimeDomainId, TargetId, PropertyId), AnimationValue>,
}

impl ResolverState {
    fn new(settings: PlaybackSettings) -> Self {
        let mut domains = oxc_index::IndexVec::new();
        domains.push(ResolvedTimeDomain {
            parent: None,
            offset: TimePoint::ZERO,
            extent: TimeExtent::ZERO,
            settings: settings.clone(),
        });
        let animation = ResolvedAnimation {
            domains,
            targets: oxc_index::IndexVec::new(),
            properties: oxc_index::IndexVec::new(),
            tweens: oxc_index::IndexVec::new(),
            events: oxc_index::IndexVec::new(),
            settings,
        };
        Self {
            animation,
            targets: FxHashMap::default(),
            properties: FxHashMap::default(),
            baselines: FxHashMap::default(),
            previous_values: FxHashMap::default(),
        }
    }

    fn intern_target(&mut self, target: ResolvedTarget) -> TargetId {
        let key = (target.adapter, target.adapter_target);
        *self
            .targets
            .entry(key)
            .or_insert_with(|| self.animation.targets.push(target))
    }

    fn intern_property(
        &mut self,
        property: ResolvedProperty,
    ) -> Result<PropertyId, AnimationResolveError> {
        let key = (property.adapter, property.adapter_property);
        if let Some(property_id) = self.properties.get(&key).copied() {
            if self.animation.properties[property_id] != property {
                return Err(AnimationResolveError::PropertyBindingConflict {
                    adapter: key.0,
                    property: key.1,
                });
            }
            return Ok(property_id);
        }
        let property_id = self.animation.properties.push(property);
        self.properties.insert(key, property_id);
        Ok(property_id)
    }

    fn baseline<Context: ResolutionContext + ?Sized>(
        &mut self,
        context: &Context,
        track: (TargetId, PropertyId),
        target: &ResolvedTarget,
        property: &ResolvedProperty,
    ) -> Result<AnimationValue, AnimationResolveError> {
        if let Some(value) = self.baselines.get(&track) {
            return Ok(value.clone());
        }
        let value = context.read_baseline(target, property)?;
        self.baselines.insert(track, value.clone());
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::num::NonZeroU32;

    use super::*;
    use crate::{
        AnimationCompiler, CallId, CallPolicy, Easing, FromValue, Modifier, Property,
        PropertyDescriptor, SourceSet, SourceTarget, TargetSetName, TimeOffset, TimeSpan,
        TimelineNode, TimelinePosition, TimelineSource, TweenSpec, ValueSource,
    };

    const OPACITY: Property<f32> = Property::static_name("opacity");

    struct FakeContext {
        duplicate: bool,
        baseline_reads: Cell<usize>,
    }

    impl FakeContext {
        fn target(index: u32) -> ResolutionTarget {
            ResolutionTarget {
                name: TargetName::owned(format!("target-{index}")),
                target: ResolvedTarget {
                    adapter: AdapterId::new(0),
                    adapter_target: AdapterTargetId::new(index as usize),
                },
                layout: Some(TargetLayoutSnapshot {
                    width_vp: 100.0 + index as f32,
                    ..TargetLayoutSnapshot::default()
                }),
            }
        }
    }

    impl ResolutionContext for FakeContext {
        fn resolve_targets(
            &self,
            _target: &SourceTarget,
        ) -> Result<Box<[ResolutionTarget]>, AnimationResolveError> {
            if self.duplicate {
                Ok(Box::new([Self::target(0), Self::target(0)]))
            } else {
                Ok(Box::new([Self::target(0), Self::target(1)]))
            }
        }

        fn resolve_property(
            &self,
            target: &ResolvedTarget,
            _property: &PropertyName,
        ) -> Result<ResolvedProperty, AnimationResolveError> {
            Ok(ResolvedProperty {
                adapter: target.adapter,
                adapter_property: AdapterPropertyId::new(0),
                descriptor: PropertyDescriptor::new(&OPACITY),
            })
        }

        fn read_baseline(
            &self,
            target: &ResolvedTarget,
            _property: &ResolvedProperty,
        ) -> Result<AnimationValue, AnimationResolveError> {
            self.baseline_reads.set(self.baseline_reads.get() + 1);
            Ok(AnimationValue::Scalar(target.adapter_target.index() as f32))
        }

        fn resolve_relative(
            &self,
            _target: &ResolvedTarget,
            _property: &ResolvedProperty,
            baseline: &AnimationValue,
            delta: &AnimationValue,
        ) -> Result<AnimationValue, AnimationResolveError> {
            match (baseline, delta) {
                (AnimationValue::Scalar(baseline), AnimationValue::Scalar(delta)) => {
                    Ok(AnimationValue::Scalar(baseline + delta))
                }
                _ => Err(AnimationResolveError::context("unsupported relative value")),
            }
        }

        fn resolve_function(
            &self,
            function: &ValueFunctionName,
            _target: &ResolvedTarget,
            _property: &ResolvedProperty,
            context: TargetContext<'_>,
        ) -> Result<AnimationValue, AnimationResolveError> {
            if function.as_str() != "by-index" {
                return Err(AnimationResolveError::context("unknown function"));
            }
            assert_eq!(context.total, 2);
            assert!(context.layout_snapshot.is_some());
            Ok(AnimationValue::Scalar((context.index + 1) as f32 * 10.0))
        }

        fn window_metrics(&self) -> WindowMetrics {
            WindowMetrics {
                width_vp: 360.0,
                height_vp: 800.0,
                density: 3.0,
            }
        }
    }

    fn source() -> SourceAnimation {
        let mut source = SourceAnimation::new(SourceTarget::Set(TargetSetName::static_name("all")));
        source.push(TweenSpec {
            property: OPACITY.name().clone(),
            from: FromValue::Current,
            to: ValueSource::Function(ValueFunctionName::static_name("by-index")),
            delay: TimeSpan::ZERO,
            duration: TimeSpan::from_millis(100),
            priority: 0,
            easing: Easing::Linear,
            composition: crate::Composition::Replace,
            modifier: Modifier::Identity,
        });
        source.push(TweenSpec {
            property: OPACITY.name().clone(),
            from: FromValue::Previous,
            to: ValueSource::Relative(AnimationValue::Scalar(5.0)),
            delay: TimeSpan::ZERO,
            duration: TimeSpan::from_millis(50),
            priority: 1,
            easing: Easing::Linear,
            composition: crate::Composition::Replace,
            modifier: Modifier::Identity,
        });
        source
    }

    #[test]
    fn resolver_expands_target_functions_and_previous_values_into_dense_slots() {
        let context = FakeContext {
            duplicate: false,
            baseline_reads: Cell::new(0),
        };
        let resolved = AnimationResolver::new(&context)
            .resolve_animation(
                &source(),
                TimePoint::from_nanos(7),
                PlaybackSettings::default(),
            )
            .unwrap();

        assert_eq!(resolved.targets.len(), 2);
        assert_eq!(resolved.properties.len(), 1);
        assert_eq!(resolved.tweens.len(), 4);
        assert_eq!(
            resolved.tweens[crate::TweenId::new(0)].from,
            AnimationValue::Scalar(0.0)
        );
        assert_eq!(
            resolved.tweens[crate::TweenId::new(0)].to,
            AnimationValue::Scalar(10.0)
        );
        assert_eq!(
            resolved.tweens[crate::TweenId::new(1)].from,
            AnimationValue::Scalar(10.0)
        );
        assert_eq!(
            resolved.tweens[crate::TweenId::new(1)].to,
            AnimationValue::Scalar(15.0)
        );
        assert_eq!(
            resolved.tweens[crate::TweenId::new(2)].from,
            AnimationValue::Scalar(1.0)
        );
        assert_eq!(
            resolved.tweens[crate::TweenId::new(3)].to,
            AnimationValue::Scalar(25.0)
        );
        assert_eq!(context.baseline_reads.get(), 2);
        assert!(AnimationCompiler.compile(resolved).is_ok());
    }

    #[test]
    fn resolver_rejects_duplicate_adapter_targets() {
        let context = FakeContext {
            duplicate: true,
            baseline_reads: Cell::new(0),
        };
        assert_eq!(
            AnimationResolver::new(&context)
                .resolve_animation(&source(), TimePoint::ZERO, PlaybackSettings::default(),)
                .unwrap_err(),
            AnimationResolveError::DuplicateTarget {
                adapter: AdapterId::new(0),
                target: AdapterTargetId::new(0),
            }
        );
    }

    #[test]
    fn timeline_resolver_compiles_positions_labels_timers_sets_and_barriers() {
        let context = FakeContext {
            duplicate: false,
            baseline_reads: Cell::new(0),
        };
        let mut timeline = TimelineSource::default();
        let mut animation = source();
        animation.tweens[0].duration = TimeSpan::from_nanos(100);
        animation.tweens[1].duration = TimeSpan::from_nanos(50);
        timeline.push(TimelineNode::Animation {
            animation,
            position: TimelinePosition::Absolute(TimePoint::from_nanos(10)),
        });
        timeline.push(TimelineNode::Timer {
            duration: TimeSpan::from_nanos(50),
            position: TimelinePosition::PreviousEnd(TimeOffset::from_nanos(20)),
        });
        timeline.push(TimelineNode::Label {
            name: LabelName::static_name("ready"),
            position: TimelinePosition::TimelineEnd(TimeOffset::ZERO),
        });
        timeline.push(TimelineNode::Call {
            call: CallId::new(3),
            policy: CallPolicy::BothDirections,
            position: TimelinePosition::Label {
                label: LabelName::static_name("ready"),
                offset: TimeOffset::from_nanos(10),
            },
        });
        timeline.push(TimelineNode::Set(SourceSet {
            target: SourceTarget::Set(TargetSetName::static_name("all")),
            property: OPACITY.name().clone(),
            value: AnimationValue::Scalar(0.5),
            position: TimelinePosition::Percentage(0.5),
        }));
        timeline.push(TimelineNode::Barrier {
            participants: NonZeroU32::new(2).unwrap(),
            position: TimelinePosition::PreviousStart(TimeOffset::from_nanos(5)),
        });

        let resolved = AnimationResolver::new(&context)
            .resolve_timeline(&timeline)
            .unwrap();

        assert_eq!(resolved.tweens.len(), 4);
        assert!(resolved
            .tweens
            .iter()
            .all(|tween| tween.start == TimePoint::from_nanos(10)));
        assert!(matches!(
            resolved.events[crate::TimelineNodeId::new(0)],
            ResolvedEvent::Call { at, .. } if at == TimePoint::from_nanos(190)
        ));
        assert!(matches!(
            resolved.events[crate::TimelineNodeId::new(1)],
            ResolvedEvent::Set { at, .. } if at == TimePoint::from_nanos(95)
        ));
        assert!(matches!(
            resolved.events[crate::TimelineNodeId::new(3)],
            ResolvedEvent::Barrier { at, .. } if at == TimePoint::from_nanos(100)
        ));
        let plan = AnimationCompiler.compile(resolved).unwrap();
        assert_eq!(plan.extent(), TimeExtent::Finite(TimeSpan::from_nanos(190)));
    }

    #[test]
    fn timeline_resolver_preserves_nested_playback_as_a_time_domain() {
        let context = FakeContext {
            duplicate: false,
            baseline_reads: Cell::new(0),
        };
        let mut child = TimelineSource::default();
        child.settings.delay = TimeSpan::from_nanos(10);
        child.settings.loop_delay = TimeSpan::from_nanos(5);
        child.settings.iterations = IterationCount::finite(2).unwrap();
        child.settings.playback_rate = crate::PlaybackRate::new(2.0).unwrap();
        child.settings.reversed = true;
        child.settings.alternate = true;
        let mut animation = source();
        animation.tweens[0].duration = TimeSpan::from_nanos(100);
        animation.tweens[1].duration = TimeSpan::from_nanos(50);
        child.push(TimelineNode::Animation {
            animation,
            position: TimelinePosition::START,
        });

        let mut timeline = TimelineSource::default();
        timeline.push(TimelineNode::Nested {
            timeline: Box::new(child),
            position: TimelinePosition::Absolute(TimePoint::from_nanos(20)),
        });
        timeline.push(TimelineNode::Call {
            call: CallId::new(9),
            policy: CallPolicy::ForwardOnly,
            position: TimelinePosition::PreviousEnd(TimeOffset::ZERO),
        });

        let resolved = AnimationResolver::new(&context)
            .resolve_timeline(&timeline)
            .unwrap();
        assert_eq!(resolved.domains.len(), 2);
        assert_eq!(
            resolved.domains[TimeDomainId::new(1)].extent,
            TimeExtent::Finite(TimeSpan::from_nanos(100))
        );
        assert!(resolved.domains[TimeDomainId::new(1)].settings.reversed);
        assert!(resolved.domains[TimeDomainId::new(1)].settings.alternate);
        assert!(resolved
            .tweens
            .iter()
            .all(|tween| tween.domain == TimeDomainId::new(1)));
        assert!(matches!(
            resolved.events[crate::TimelineNodeId::new(0)],
            ResolvedEvent::Call { domain, at, .. }
                if domain == TimeDomainId::new(0) && at == TimePoint::from_nanos(135)
        ));

        let plan = AnimationCompiler.compile(resolved).unwrap();
        assert_eq!(plan.extent(), TimeExtent::Finite(TimeSpan::from_nanos(135)));
        assert_eq!(plan.domains().len(), 2);
        assert!(plan
            .tracks()
            .iter()
            .all(|track| track.domain == TimeDomainId::new(1)));
    }

    #[test]
    fn infinite_nested_iterations_produce_an_infinite_parent_extent() {
        let context = FakeContext {
            duplicate: false,
            baseline_reads: Cell::new(0),
        };
        let mut child = TimelineSource::default();
        child.settings.iterations = IterationCount::Infinite;
        let mut animation = source();
        animation.tweens[0].duration = TimeSpan::from_nanos(100);
        animation.tweens[1].duration = TimeSpan::from_nanos(50);
        child.push(TimelineNode::Animation {
            animation,
            position: TimelinePosition::START,
        });
        let mut timeline = TimelineSource::default();
        timeline.push(TimelineNode::Nested {
            timeline: Box::new(child),
            position: TimelinePosition::START,
        });

        let resolved = AnimationResolver::new(&context)
            .resolve_timeline(&timeline)
            .unwrap();
        assert_eq!(
            resolved.domains[TimeDomainId::new(0)].extent,
            TimeExtent::Infinite
        );
        assert_eq!(
            AnimationCompiler.compile(resolved).unwrap().extent(),
            TimeExtent::Infinite
        );

        timeline.push(TimelineNode::Call {
            call: CallId::new(1),
            policy: CallPolicy::ForwardOnly,
            position: TimelinePosition::TimelineEnd(TimeOffset::ZERO),
        });
        assert_eq!(
            AnimationResolver::new(&context)
                .resolve_timeline(&timeline)
                .unwrap_err(),
            AnimationResolveError::PositionAfterInfiniteTimeline
        );
    }
}
