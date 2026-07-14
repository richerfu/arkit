//! Deterministic lowering from adapter-resolved inputs to an immutable runtime plan.

use std::sync::Arc;

use oxc_index::IndexVec;
use rustc_hash::FxHashMap;

use crate::plan::CompiledAnimationParts;
use crate::{
    AnimationCompileError, BaselineStrategy, CompiledAnimation, CompiledEvent, CompiledOutput,
    CompiledProperty, CompiledTarget, CompiledTimeDomain, CompiledTrack, CompiledTrackSegment,
    CompiledTween, Composition, Interpolation, OutputId, PropertyDescriptor, PropertyId,
    ResolvedAnimation, ResolvedEvent, ResolvedProperty, ResolvedTimeDomain, ResolvedTween,
    TargetId, TimeDomainId, TimeExtent, TimePoint, TimeSpan, TimelineNodeId, TrackId, TweenId,
};

#[derive(Debug, Clone, Copy, Default)]
pub struct AnimationCompiler;

impl AnimationCompiler {
    pub fn compile(
        &self,
        resolved: ResolvedAnimation,
    ) -> Result<Arc<CompiledAnimation>, AnimationCompileError> {
        validate_domains(&resolved.domains)?;
        validate_properties(&resolved.properties)?;

        let mut pending_tweens = Vec::with_capacity(resolved.tweens.len());
        for (original, tween) in resolved.tweens.into_iter_enumerated() {
            let compiled = compile_tween(
                original,
                tween,
                &resolved.domains,
                &resolved.targets,
                &resolved.properties,
            )?;
            pending_tweens.push((original, compiled));
        }
        pending_tweens.sort_by_key(|(original, tween)| {
            (
                tween.target,
                tween.property,
                tween.domain,
                tween.start,
                tween.end,
                *original,
            )
        });

        let mut tweens = IndexVec::with_capacity(pending_tweens.len());
        let mut track_tweens: FxHashMap<(TargetId, PropertyId, TimeDomainId), Vec<TweenId>> =
            FxHashMap::default();
        for (_, tween) in pending_tweens {
            let key = (tween.target, tween.property, tween.domain);
            let tween_id = tweens.push(tween);
            track_tweens.entry(key).or_default().push(tween_id);
        }

        let mut track_entries: Vec<_> = track_tweens.into_iter().collect();
        track_entries.sort_unstable_by_key(|(key, _)| *key);
        let mut tracks = IndexVec::<TrackId, CompiledTrack>::with_capacity(track_entries.len());
        for ((target, property, domain), tween_ids) in track_entries {
            let segments = build_track_segments(&tween_ids, &tweens);
            tracks.push(CompiledTrack::new(
                domain,
                target,
                property,
                tween_ids.into_boxed_slice(),
                segments,
            ));
        }

        let mut pending_events = Vec::with_capacity(resolved.events.len());
        for (original, event) in resolved.events.into_iter_enumerated() {
            let event = compile_event(
                event,
                &resolved.domains,
                &resolved.targets,
                &resolved.properties,
            )?;
            pending_events.push((original, event));
        }
        pending_events.sort_by_key(|(original, event)| {
            (
                event_domain(event),
                event_time(event),
                event_kind_rank(event),
                *original,
            )
        });
        let events = IndexVec::<TimelineNodeId, CompiledEvent>::from_vec(
            pending_events.into_iter().map(|(_, event)| event).collect(),
        );

        let outputs = build_outputs(&tracks, &events);

        let root_duration_nanos = tweens
            .iter()
            .filter(|tween| tween.domain == TimeDomainId::new(0))
            .map(|tween| tween.end.as_nanos())
            .chain(
                events
                    .iter()
                    .filter(|event| event_domain(event) == TimeDomainId::new(0))
                    .map(event_time)
                    .map(TimePoint::as_nanos),
            )
            .max()
            .unwrap_or(0);
        let targets = IndexVec::from_vec(
            resolved
                .targets
                .into_iter()
                .map(|target| CompiledTarget {
                    adapter: target.adapter,
                    adapter_target: target.adapter_target,
                })
                .collect(),
        );
        let properties = IndexVec::from_vec(
            resolved
                .properties
                .into_iter()
                .map(|property| CompiledProperty {
                    adapter: property.adapter,
                    adapter_property: property.adapter_property,
                    descriptor: property.descriptor,
                })
                .collect(),
        );
        let mut domains = IndexVec::from_vec(
            resolved
                .domains
                .into_iter()
                .map(|domain| CompiledTimeDomain {
                    parent: domain.parent,
                    offset: domain.offset,
                    extent: domain.extent,
                    settings: domain.settings,
                    first_event: None,
                    event_count: 0,
                })
                .collect(),
        );
        let mut event_ranges = vec![(None, 0_u32); domains.len()];
        for (event_id, event) in events.iter_enumerated() {
            let range = &mut event_ranges[event_domain(event).index()];
            range.0.get_or_insert(event_id);
            range.1 = range.1.saturating_add(1);
        }
        for (domain_id, (first, count)) in event_ranges.into_iter().enumerate() {
            domains[TimeDomainId::new(domain_id)].set_event_range(first, count);
        }
        let root_extent = domains.raw.first().map_or(
            TimeExtent::Finite(TimeSpan::from_nanos(root_duration_nanos)),
            |domain| {
                domain.extent.max(TimeExtent::Finite(TimeSpan::from_nanos(
                    root_duration_nanos,
                )))
            },
        );
        if let Some(root) = domains.raw.first_mut() {
            root.extent = root_extent;
        }

        Ok(CompiledAnimation::from_parts(CompiledAnimationParts {
            extent: root_extent,
            settings: resolved.settings,
            targets,
            properties,
            tweens,
            tracks,
            outputs,
            events,
            domains,
        }))
    }
}

fn build_outputs(
    tracks: &IndexVec<TrackId, CompiledTrack>,
    events: &IndexVec<TimelineNodeId, CompiledEvent>,
) -> IndexVec<OutputId, CompiledOutput> {
    let mut groups: FxHashMap<(TargetId, PropertyId), (Vec<TrackId>, Vec<TimelineNodeId>)> =
        FxHashMap::default();
    for (track_id, track) in tracks.iter_enumerated() {
        groups
            .entry((track.target, track.property))
            .or_default()
            .0
            .push(track_id);
    }
    for (event_id, event) in events.iter_enumerated() {
        if let CompiledEvent::Set {
            target, property, ..
        } = event
        {
            groups
                .entry((*target, *property))
                .or_default()
                .1
                .push(event_id);
        }
    }

    let mut entries: Vec<_> = groups.into_iter().collect();
    entries.sort_unstable_by_key(|(key, _)| *key);
    IndexVec::from_vec(
        entries
            .into_iter()
            .map(|((target, property), (tracks, set_events))| {
                CompiledOutput::new(
                    target,
                    property,
                    tracks.into_boxed_slice(),
                    set_events.into_boxed_slice(),
                )
            })
            .collect(),
    )
}

fn validate_domains(
    domains: &IndexVec<TimeDomainId, ResolvedTimeDomain>,
) -> Result<(), AnimationCompileError> {
    let Some(root) = domains.raw.first() else {
        return Err(AnimationCompileError::MissingRootTimeDomain);
    };
    if root.parent.is_some() {
        return Err(AnimationCompileError::InvalidTimeDomainParent {
            domain: TimeDomainId::new(0),
            parent: root.parent,
        });
    }
    for (domain, resolved) in domains.iter_enumerated() {
        if domain.index() > 0
            && resolved
                .parent
                .is_none_or(|parent| parent.index() >= domain.index())
        {
            return Err(AnimationCompileError::InvalidTimeDomainParent {
                domain,
                parent: resolved.parent,
            });
        }
        resolved
            .settings
            .playback_easing
            .validate()
            .map_err(|source| AnimationCompileError::InvalidTimeDomainEasing { domain, source })?;
    }
    Ok(())
}

fn validate_properties(
    properties: &IndexVec<PropertyId, ResolvedProperty>,
) -> Result<(), AnimationCompileError> {
    for (property, binding) in properties.iter_enumerated() {
        let descriptor = &binding.descriptor;
        if !descriptor.precision.is_finite() || descriptor.precision <= 0.0 {
            return Err(AnimationCompileError::InvalidPropertyPrecision(property));
        }
        if let BaselineStrategy::Default(value) = &descriptor.baseline {
            descriptor
                .validate_value(value)
                .map_err(|source| AnimationCompileError::InvalidValue { property, source })?;
        }
    }
    Ok(())
}

fn compile_tween(
    tween_id: TweenId,
    tween: ResolvedTween,
    domains: &IndexVec<TimeDomainId, ResolvedTimeDomain>,
    targets: &IndexVec<TargetId, crate::ResolvedTarget>,
    properties: &IndexVec<PropertyId, ResolvedProperty>,
) -> Result<CompiledTween, AnimationCompileError> {
    if domains.raw.get(tween.domain.index()).is_none() {
        return Err(AnimationCompileError::UnknownTimeDomain(tween.domain));
    }
    let target = targets
        .raw
        .get(tween.target.index())
        .ok_or(AnimationCompileError::UnknownTarget(tween.target))?;
    let property = properties
        .raw
        .get(tween.property.index())
        .ok_or(AnimationCompileError::UnknownProperty(tween.property))?;
    if target.adapter != property.adapter {
        return Err(AnimationCompileError::AdapterMismatch {
            target: tween.target,
            property: tween.property,
        });
    }
    let descriptor = &property.descriptor;
    validate_property_write(tween.property, descriptor)?;
    validate_tween_values(tween.property, descriptor, &tween.from, &tween.to)?;
    validate_composition(tween.property, descriptor, tween.composition)?;
    tween
        .easing
        .validate()
        .map_err(|source| AnimationCompileError::InvalidEasing {
            tween: tween_id,
            source,
        })?;
    tween
        .modifier
        .validate_for_kind(descriptor.value_kind)
        .map_err(|source| AnimationCompileError::InvalidModifier {
            property: tween.property,
            source,
        })?;

    let start = tween
        .start
        .checked_add(tween.delay)
        .ok_or(AnimationCompileError::TimeOverflow(tween_id))?;
    let end = start
        .checked_add(tween.duration)
        .ok_or(AnimationCompileError::TimeOverflow(tween_id))?;

    Ok(CompiledTween {
        domain: tween.domain,
        target: tween.target,
        property: tween.property,
        start,
        end,
        priority: tween.priority,
        source_order: tween_id.index() as u32,
        from: tween.from,
        to: tween.to,
        easing: tween.easing,
        composition: tween.composition,
        modifier: tween.modifier,
        invalidation: descriptor.invalidation,
    })
}

fn build_track_segments(
    tween_ids: &[TweenId],
    tweens: &IndexVec<TweenId, CompiledTween>,
) -> oxc_index::IndexBox<crate::TrackSegmentId, [CompiledTrackSegment]> {
    let mut boundaries = Vec::with_capacity(tween_ids.len() * 2);
    for tween_id in tween_ids {
        let tween = &tweens[*tween_id];
        boundaries.push(tween.start);
        boundaries.push(tween.end);
    }
    boundaries.sort_unstable();
    boundaries.dedup();

    let mut segments = IndexVec::with_capacity(boundaries.len());
    for (boundary_index, start) in boundaries.iter().copied().enumerate() {
        let end = boundaries.get(boundary_index + 1).copied().unwrap_or(start);
        let mut replace: Option<TweenId> = None;
        let mut additive = Vec::new();
        let mut accumulating = Vec::new();

        for tween_id in tween_ids.iter().copied() {
            let tween = &tweens[tween_id];
            if tween.start > start {
                break;
            }
            match tween.composition {
                Composition::Replace => {
                    let precedence = (tween.start, tween.priority, tween.source_order);
                    let wins = replace.is_none_or(|winner| {
                        let winner = &tweens[winner];
                        precedence > (winner.start, winner.priority, winner.source_order)
                    });
                    if wins {
                        replace = Some(tween_id);
                    }
                }
                Composition::Add => additive.push(tween_id),
                Composition::Accumulate => accumulating.push(tween_id),
            }
        }
        additive.sort_unstable_by_key(|tween_id| {
            let tween = &tweens[*tween_id];
            (tween.start, tween.priority, tween.source_order)
        });
        accumulating.sort_unstable_by_key(|tween_id| {
            let tween = &tweens[*tween_id];
            (tween.start, tween.priority, tween.source_order)
        });
        segments.push(CompiledTrackSegment::new(
            start,
            end,
            replace,
            additive.into_boxed_slice(),
            accumulating.into_boxed_slice(),
        ));
    }
    segments.into_boxed_slice()
}

fn compile_event(
    event: ResolvedEvent,
    domains: &IndexVec<TimeDomainId, ResolvedTimeDomain>,
    targets: &IndexVec<TargetId, crate::ResolvedTarget>,
    properties: &IndexVec<PropertyId, ResolvedProperty>,
) -> Result<CompiledEvent, AnimationCompileError> {
    match event {
        ResolvedEvent::Call {
            domain,
            at,
            call,
            policy,
        } => {
            validate_domain(domains, domain)?;
            Ok(CompiledEvent::Call {
                domain,
                at,
                call,
                policy,
            })
        }
        ResolvedEvent::Set {
            domain,
            at,
            target,
            property,
            value,
        } => {
            validate_domain(domains, domain)?;
            let target_binding = targets
                .raw
                .get(target.index())
                .ok_or(AnimationCompileError::UnknownTarget(target))?;
            let property_binding = properties
                .raw
                .get(property.index())
                .ok_or(AnimationCompileError::UnknownProperty(property))?;
            if target_binding.adapter != property_binding.adapter {
                return Err(AnimationCompileError::AdapterMismatch { target, property });
            }
            let descriptor = &property_binding.descriptor;
            validate_property_write(property, descriptor)?;
            descriptor
                .validate_value(&value)
                .map_err(|source| AnimationCompileError::InvalidValue { property, source })?;
            Ok(CompiledEvent::Set {
                domain,
                at,
                target,
                property,
                value,
            })
        }
        ResolvedEvent::Barrier {
            domain,
            at,
            participants,
        } => {
            validate_domain(domains, domain)?;
            Ok(CompiledEvent::Barrier {
                domain,
                at,
                participants,
            })
        }
    }
}

fn validate_domain(
    domains: &IndexVec<TimeDomainId, ResolvedTimeDomain>,
    domain: TimeDomainId,
) -> Result<(), AnimationCompileError> {
    domains
        .raw
        .get(domain.index())
        .map(|_| ())
        .ok_or(AnimationCompileError::UnknownTimeDomain(domain))
}

fn validate_property_write(
    property: PropertyId,
    descriptor: &PropertyDescriptor,
) -> Result<(), AnimationCompileError> {
    if descriptor.writable {
        Ok(())
    } else {
        Err(AnimationCompileError::PropertyNotWritable(property))
    }
}

fn validate_tween_values(
    property: PropertyId,
    descriptor: &PropertyDescriptor,
    from: &crate::AnimationValue,
    to: &crate::AnimationValue,
) -> Result<(), AnimationCompileError> {
    descriptor
        .validate_value(from)
        .and_then(|()| descriptor.validate_value(to))
        .map_err(|source| AnimationCompileError::InvalidValue { property, source })?;

    if descriptor.interpolation == Interpolation::Linear {
        from.interpolate(to, 0.5)
            .map(drop)
            .map_err(|source| AnimationCompileError::InvalidValue { property, source })?;
    }
    Ok(())
}

fn validate_composition(
    property: PropertyId,
    descriptor: &PropertyDescriptor,
    composition: Composition,
) -> Result<(), AnimationCompileError> {
    let supported = match composition {
        Composition::Replace => descriptor.composition.replace,
        Composition::Add => descriptor.composition.add,
        Composition::Accumulate => descriptor.composition.accumulate,
    };
    if supported {
        Ok(())
    } else {
        Err(AnimationCompileError::UnsupportedComposition {
            property,
            composition,
        })
    }
}

const fn event_time(event: &CompiledEvent) -> TimePoint {
    match event {
        CompiledEvent::Call { at, .. }
        | CompiledEvent::Set { at, .. }
        | CompiledEvent::Barrier { at, .. } => *at,
    }
}

const fn event_domain(event: &CompiledEvent) -> TimeDomainId {
    match event {
        CompiledEvent::Call { domain, .. }
        | CompiledEvent::Set { domain, .. }
        | CompiledEvent::Barrier { domain, .. } => *domain,
    }
}

const fn event_kind_rank(event: &CompiledEvent) -> u8 {
    match event {
        CompiledEvent::Set { .. } => 0,
        CompiledEvent::Call { .. } => 1,
        CompiledEvent::Barrier { .. } => 2,
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU32;

    use super::*;
    use crate::{
        AdapterId, AdapterPropertyId, AdapterTargetId, AnimationValue, CallId, CallPolicy,
        CompositionSupport, Easing, InvalidationClass, Length, Modifier, Property, ResolvedTarget,
        UnitDomain, ValueError, NANOS_PER_MILLISECOND,
    };

    const OPACITY: Property<f32> = Property::static_name("opacity");
    const WIDTH: Property<Length> = Property::static_name("width");

    fn point(milliseconds: u64) -> TimePoint {
        TimePoint::from_nanos(milliseconds * NANOS_PER_MILLISECOND)
    }

    fn resolved_with_property(descriptor: PropertyDescriptor) -> ResolvedAnimation {
        let mut resolved = ResolvedAnimation::default();
        resolved.targets.push(ResolvedTarget {
            adapter: AdapterId::new(1),
            adapter_target: AdapterTargetId::new(2),
        });
        resolved.properties.push(ResolvedProperty {
            adapter: AdapterId::new(1),
            adapter_property: AdapterPropertyId::new(0),
            descriptor,
        });
        resolved
    }

    fn tween(start_ms: u64, delay_ms: u64, duration_ms: u64) -> ResolvedTween {
        ResolvedTween {
            domain: TimeDomainId::new(0),
            target: TargetId::new(0),
            property: PropertyId::new(0),
            start: point(start_ms),
            delay: TimeSpan::from_millis(delay_ms),
            duration: TimeSpan::from_millis(duration_ms),
            priority: 0,
            from: AnimationValue::Scalar(0.0),
            to: AnimationValue::Scalar(1.0),
            easing: Easing::Linear,
            composition: Composition::Replace,
            modifier: Modifier::Identity,
        }
    }

    #[test]
    fn compiler_folds_delay_and_builds_property_tracks() {
        let mut resolved = resolved_with_property(PropertyDescriptor::new(&OPACITY));
        resolved.tweens.push(tween(40, 10, 100));
        resolved.tweens.push(tween(0, 0, 20));

        let plan = AnimationCompiler.compile(resolved).unwrap();

        assert_eq!(
            plan.extent(),
            TimeExtent::Finite(TimeSpan::from_millis(150))
        );
        assert_eq!(plan.tweens()[TweenId::new(0)].start, point(0));
        assert_eq!(plan.tweens()[TweenId::new(1)].start, point(50));
        assert_eq!(plan.tracks().len(), 1);
        assert_eq!(
            plan.tracks()[TrackId::new(0)].tweens(),
            &[TweenId::new(0), TweenId::new(1)]
        );
    }

    #[test]
    fn compiler_sorts_events_and_includes_them_in_duration() {
        let mut resolved = resolved_with_property(PropertyDescriptor::new(&OPACITY));
        resolved.events.push(ResolvedEvent::Barrier {
            domain: TimeDomainId::new(0),
            at: point(80),
            participants: NonZeroU32::new(2).unwrap(),
        });
        resolved.events.push(ResolvedEvent::Call {
            domain: TimeDomainId::new(0),
            at: point(10),
            call: CallId::new(4),
            policy: CallPolicy::BothDirections,
        });
        resolved.events.push(ResolvedEvent::Set {
            domain: TimeDomainId::new(0),
            at: point(10),
            target: TargetId::new(0),
            property: PropertyId::new(0),
            value: AnimationValue::Scalar(0.5),
        });

        let plan = AnimationCompiler.compile(resolved).unwrap();

        assert_eq!(plan.extent(), TimeExtent::Finite(TimeSpan::from_millis(80)));
        assert!(matches!(
            plan.events()[TimelineNodeId::new(0)],
            CompiledEvent::Set { .. }
        ));
        assert!(matches!(
            plan.events()[TimelineNodeId::new(1)],
            CompiledEvent::Call { .. }
        ));
        assert!(matches!(
            plan.events()[TimelineNodeId::new(2)],
            CompiledEvent::Barrier { .. }
        ));
        assert_eq!(plan.outputs().len(), 1);
        assert!(plan.outputs()[OutputId::new(0)].tracks().is_empty());
        assert_eq!(
            plan.outputs()[OutputId::new(0)].set_events(),
            &[TimelineNodeId::new(0)]
        );
    }

    #[test]
    fn compiler_groups_tracks_from_multiple_domains_into_one_output() {
        let mut resolved = resolved_with_property(PropertyDescriptor::new(&OPACITY));
        resolved.domains.push(ResolvedTimeDomain {
            parent: Some(TimeDomainId::new(0)),
            offset: TimePoint::ZERO,
            extent: TimeExtent::Finite(TimeSpan::from_millis(100)),
            settings: Default::default(),
        });
        resolved.tweens.push(tween(0, 0, 100));
        let mut nested = tween(0, 0, 100);
        nested.domain = TimeDomainId::new(1);
        resolved.tweens.push(nested);

        let plan = AnimationCompiler.compile(resolved).unwrap();

        assert_eq!(plan.tracks().len(), 2);
        assert_eq!(plan.outputs().len(), 1);
        assert_eq!(
            plan.outputs()[OutputId::new(0)].tracks(),
            &[TrackId::new(0), TrackId::new(1)]
        );
    }

    #[test]
    fn compiler_rejects_unknown_ids_and_non_writable_properties() {
        let mut resolved = resolved_with_property(PropertyDescriptor::new(&OPACITY));
        let mut invalid = tween(0, 0, 10);
        invalid.target = TargetId::new(7);
        resolved.tweens.push(invalid);
        assert_eq!(
            AnimationCompiler.compile(resolved).unwrap_err(),
            AnimationCompileError::UnknownTarget(TargetId::new(7))
        );

        let mut descriptor = PropertyDescriptor::new(&OPACITY);
        descriptor.writable = false;
        let mut resolved = resolved_with_property(descriptor);
        resolved.tweens.push(tween(0, 0, 10));
        assert_eq!(
            AnimationCompiler.compile(resolved).unwrap_err(),
            AnimationCompileError::PropertyNotWritable(PropertyId::new(0))
        );

        let mut resolved = resolved_with_property(PropertyDescriptor::new(&OPACITY));
        resolved.properties[PropertyId::new(0)].adapter = AdapterId::new(9);
        resolved.tweens.push(tween(0, 0, 10));
        assert_eq!(
            AnimationCompiler.compile(resolved).unwrap_err(),
            AnimationCompileError::AdapterMismatch {
                target: TargetId::new(0),
                property: PropertyId::new(0),
            }
        );
    }

    #[test]
    fn compiler_rejects_value_unit_composition_and_modifier_mismatches() {
        let mut descriptor = PropertyDescriptor::new(&WIDTH);
        descriptor.unit_domain = UnitDomain::Length {
            vp: true,
            px: false,
            percent: false,
        };
        descriptor.invalidation = InvalidationClass::Layout;
        let mut resolved = resolved_with_property(descriptor);
        let mut invalid = tween(0, 0, 10);
        invalid.from = AnimationValue::Length(Length::vp(0.0));
        invalid.to = AnimationValue::Length(Length::px(10.0));
        resolved.tweens.push(invalid);
        assert_eq!(
            AnimationCompiler.compile(resolved).unwrap_err(),
            AnimationCompileError::InvalidValue {
                property: PropertyId::new(0),
                source: ValueError::UnitNotSupported,
            }
        );

        let mut descriptor = PropertyDescriptor::new(&WIDTH);
        descriptor.unit_domain = UnitDomain::ALL_LENGTHS;
        let mut resolved = resolved_with_property(descriptor);
        let mut invalid = tween(0, 0, 10);
        invalid.from = AnimationValue::Length(Length::vp(0.0));
        invalid.to = AnimationValue::Length(Length::px(10.0));
        resolved.tweens.push(invalid);
        assert_eq!(
            AnimationCompiler.compile(resolved).unwrap_err(),
            AnimationCompileError::InvalidValue {
                property: PropertyId::new(0),
                source: ValueError::UnitMismatch,
            }
        );

        let mut resolved = resolved_with_property(PropertyDescriptor::new(&OPACITY));
        let mut invalid = tween(0, 0, 10);
        invalid.composition = Composition::Add;
        resolved.tweens.push(invalid);
        assert_eq!(
            AnimationCompiler.compile(resolved).unwrap_err(),
            AnimationCompileError::UnsupportedComposition {
                property: PropertyId::new(0),
                composition: Composition::Add,
            }
        );

        let mut descriptor = PropertyDescriptor::new(&WIDTH);
        descriptor.unit_domain = UnitDomain::ALL_LENGTHS;
        let mut resolved = resolved_with_property(descriptor);
        let mut invalid = tween(0, 0, 10);
        invalid.from = AnimationValue::Length(Length::vp(0.0));
        invalid.to = AnimationValue::Length(Length::vp(10.0));
        invalid.modifier = Modifier::Round { decimal_places: 0 };
        resolved.tweens.push(invalid);
        assert!(matches!(
            AnimationCompiler.compile(resolved),
            Err(AnimationCompileError::InvalidModifier { .. })
        ));
    }

    #[test]
    fn compiler_reports_time_overflow_instead_of_saturating() {
        let mut resolved = resolved_with_property(PropertyDescriptor::new(&OPACITY));
        let mut invalid = tween(0, 0, 1);
        invalid.start = TimePoint::from_nanos(u64::MAX);
        resolved.tweens.push(invalid);
        assert_eq!(
            AnimationCompiler.compile(resolved).unwrap_err(),
            AnimationCompileError::TimeOverflow(TweenId::new(0))
        );
    }

    #[test]
    fn compiler_validates_public_easing_variants() {
        let mut resolved = resolved_with_property(PropertyDescriptor::new(&OPACITY));
        let mut invalid = tween(0, 0, 10);
        invalid.easing = Easing::CubicBezier {
            x1: -1.0,
            y1: 0.0,
            x2: 1.0,
            y2: 1.0,
        };
        resolved.tweens.push(invalid);
        assert!(matches!(
            AnimationCompiler.compile(resolved),
            Err(AnimationCompileError::InvalidEasing { .. })
        ));
    }

    #[test]
    fn compiler_builds_deterministic_composition_segments() {
        let mut descriptor = PropertyDescriptor::new(&OPACITY);
        descriptor.composition = CompositionSupport::NUMERIC;
        let mut resolved = resolved_with_property(descriptor);

        let first_replace = resolved.tweens.push(tween(0, 0, 30));
        let mut additive = tween(5, 0, 20);
        additive.composition = Composition::Add;
        let additive = resolved.tweens.push(additive);
        let mut accumulating = tween(8, 0, 20);
        accumulating.composition = Composition::Accumulate;
        let accumulating = resolved.tweens.push(accumulating);
        let mut lower_priority = tween(10, 0, 20);
        lower_priority.priority = -1;
        resolved.tweens.push(lower_priority);
        let mut higher_priority = tween(10, 0, 10);
        higher_priority.priority = 5;
        let higher_priority = resolved.tweens.push(higher_priority);

        let plan = AnimationCompiler.compile(resolved).unwrap();
        let track = &plan.tracks()[TrackId::new(0)];
        let segment_id = track.seek_segment(point(10)).unwrap();
        let segment = &track.segments()[segment_id];
        let source_order = |compiled: TweenId| plan.tweens()[compiled].source_order;

        assert_eq!(
            source_order(segment.replace.unwrap()),
            higher_priority.index() as u32
        );
        assert_eq!(
            segment
                .additive()
                .iter()
                .map(|id| source_order(*id))
                .collect::<Vec<_>>(),
            [additive.index() as u32]
        );
        assert_eq!(
            segment
                .accumulating()
                .iter()
                .map(|id| source_order(*id))
                .collect::<Vec<_>>(),
            [accumulating.index() as u32]
        );
        assert_ne!(
            source_order(segment.replace.unwrap()),
            first_replace.index() as u32
        );
    }

    #[test]
    fn identical_resolved_inputs_produce_byte_identical_plan_traces() {
        fn input() -> ResolvedAnimation {
            let mut resolved = resolved_with_property(PropertyDescriptor::new(&OPACITY));
            resolved.tweens.push(tween(40, 10, 100));
            resolved.tweens.push(tween(0, 0, 20));
            resolved.events.push(ResolvedEvent::Call {
                domain: TimeDomainId::new(0),
                at: point(25),
                call: CallId::new(1),
                policy: CallPolicy::Once,
            });
            resolved
        }

        let first = AnimationCompiler.compile(input()).unwrap();
        let second = AnimationCompiler.compile(input()).unwrap();
        assert_eq!(
            first.deterministic_trace().as_bytes(),
            second.deterministic_trace().as_bytes()
        );
    }
}
