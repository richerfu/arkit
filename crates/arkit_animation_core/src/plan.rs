//! Immutable, dense-ID animation plan consumed by the runtime engine.

use std::num::NonZeroU32;
use std::sync::Arc;

use oxc_index::{IndexBox, IndexVec};

use crate::{
    AdapterId, AdapterPropertyId, AdapterTargetId, AnimationValue, CallId, CallPolicy, Composition,
    Easing, InvalidationClass, Modifier, OutputId, PlaybackSettings, PropertyDescriptor,
    PropertyId, TargetId, TimeDomainId, TimeExtent, TimePoint, TimeSpan, TimelineNodeId, TrackId,
    TrackSegmentId, TweenId,
};

#[derive(Debug, Clone)]
pub struct CompiledTarget {
    pub adapter: AdapterId,
    pub adapter_target: AdapterTargetId,
}

#[derive(Debug, Clone)]
pub struct CompiledProperty {
    pub adapter: AdapterId,
    pub adapter_property: AdapterPropertyId,
    pub descriptor: PropertyDescriptor,
}

#[derive(Debug, Clone)]
pub struct CompiledTween {
    pub domain: TimeDomainId,
    pub target: TargetId,
    pub property: PropertyId,
    pub start: TimePoint,
    pub end: TimePoint,
    pub priority: i32,
    pub source_order: u32,
    pub from: AnimationValue,
    pub to: AnimationValue,
    pub easing: Easing,
    pub composition: Composition,
    pub modifier: Modifier,
    pub invalidation: InvalidationClass,
}

#[derive(Debug, Clone)]
pub struct CompiledTrackSegment {
    pub start: TimePoint,
    pub end: TimePoint,
    pub replace: Option<TweenId>,
    additive: Box<[TweenId]>,
    accumulating: Box<[TweenId]>,
}

impl CompiledTrackSegment {
    pub(crate) fn new(
        start: TimePoint,
        end: TimePoint,
        replace: Option<TweenId>,
        additive: Box<[TweenId]>,
        accumulating: Box<[TweenId]>,
    ) -> Self {
        Self {
            start,
            end,
            replace,
            additive,
            accumulating,
        }
    }

    pub fn additive(&self) -> &[TweenId] {
        &self.additive
    }

    pub fn accumulating(&self) -> &[TweenId] {
        &self.accumulating
    }
}

#[derive(Debug, Clone)]
pub struct CompiledTrack {
    pub domain: TimeDomainId,
    pub target: TargetId,
    pub property: PropertyId,
    tweens: Box<[TweenId]>,
    segments: IndexBox<TrackSegmentId, [CompiledTrackSegment]>,
}

#[derive(Debug, Clone)]
pub struct CompiledOutput {
    pub target: TargetId,
    pub property: PropertyId,
    tracks: Box<[TrackId]>,
    set_events: Box<[TimelineNodeId]>,
}

impl CompiledOutput {
    pub(crate) fn new(
        target: TargetId,
        property: PropertyId,
        tracks: Box<[TrackId]>,
        set_events: Box<[TimelineNodeId]>,
    ) -> Self {
        Self {
            target,
            property,
            tracks,
            set_events,
        }
    }

    pub fn tracks(&self) -> &[TrackId] {
        &self.tracks
    }

    pub fn set_events(&self) -> &[TimelineNodeId] {
        &self.set_events
    }
}

impl CompiledTrack {
    pub(crate) fn new(
        domain: TimeDomainId,
        target: TargetId,
        property: PropertyId,
        tweens: Box<[TweenId]>,
        segments: IndexBox<TrackSegmentId, [CompiledTrackSegment]>,
    ) -> Self {
        Self {
            domain,
            target,
            property,
            tweens,
            segments,
        }
    }

    pub fn tweens(&self) -> &[TweenId] {
        &self.tweens
    }

    pub fn segments(&self) -> &oxc_index::IndexSlice<TrackSegmentId, [CompiledTrackSegment]> {
        &self.segments
    }

    pub fn seek_segment(&self, at: TimePoint) -> Option<TrackSegmentId> {
        match self
            .segments
            .binary_search_by_key(&at, |segment| segment.start)
        {
            Ok(segment) => Some(segment),
            Err(insertion) if insertion.index() > 0 => {
                Some(TrackSegmentId::new(insertion.index() - 1))
            }
            Err(_) => None,
        }
    }
}

impl CompiledTimeDomain {
    pub fn parent_extent(&self) -> TimeExtent {
        let active = match self.extent {
            TimeExtent::Infinite => return TimeExtent::Infinite,
            TimeExtent::Finite(active) => active,
        };
        let iterations = match self.settings.iterations {
            crate::IterationCount::Infinite => return TimeExtent::Infinite,
            crate::IterationCount::Finite(iterations) => iterations.get(),
        };
        let scaled = active.as_nanos() as f64 / self.settings.playback_rate.get();
        if !scaled.is_finite() || scaled > u64::MAX as f64 {
            return TimeExtent::Infinite;
        }
        let active = TimeSpan::from_nanos(scaled.round() as u64);
        let Some(active) = active.checked_mul(iterations) else {
            return TimeExtent::Infinite;
        };
        let Some(loop_delay) = self
            .settings
            .loop_delay
            .checked_mul(iterations.saturating_sub(1))
        else {
            return TimeExtent::Infinite;
        };
        self.settings
            .delay
            .checked_add(active)
            .and_then(|duration| duration.checked_add(loop_delay))
            .map_or(TimeExtent::Infinite, TimeExtent::Finite)
    }
}

impl CompiledTween {
    pub fn duration(&self) -> TimeSpan {
        self.end - self.start
    }
}

#[derive(Debug, Clone)]
pub enum CompiledEvent {
    Call {
        domain: TimeDomainId,
        at: TimePoint,
        call: CallId,
        policy: CallPolicy,
    },
    Set {
        domain: TimeDomainId,
        at: TimePoint,
        target: TargetId,
        property: PropertyId,
        value: AnimationValue,
    },
    Barrier {
        domain: TimeDomainId,
        at: TimePoint,
        participants: NonZeroU32,
    },
}

#[derive(Debug, Clone)]
pub struct CompiledTimeDomain {
    pub parent: Option<TimeDomainId>,
    pub offset: TimePoint,
    pub extent: TimeExtent,
    pub settings: PlaybackSettings,
    pub(crate) first_event: Option<TimelineNodeId>,
    pub(crate) event_count: u32,
}

impl CompiledTimeDomain {
    pub(crate) fn set_event_range(&mut self, first: Option<TimelineNodeId>, count: u32) {
        self.first_event = first;
        self.event_count = count;
    }

    pub fn event_ids(&self) -> impl Iterator<Item = TimelineNodeId> {
        let start = self.first_event.map_or(0, TimelineNodeId::index);
        let count = self.event_count as usize;
        (start..start + count).map(TimelineNodeId::new)
    }

    pub(crate) fn event_index_range(&self) -> std::ops::Range<usize> {
        let start = self.first_event.map_or(0, TimelineNodeId::index);
        start..start + self.event_count as usize
    }
}

#[derive(Debug, Clone)]
pub struct CompiledAnimation {
    extent: TimeExtent,
    settings: PlaybackSettings,
    targets: IndexVec<TargetId, CompiledTarget>,
    properties: IndexVec<PropertyId, CompiledProperty>,
    tweens: IndexVec<TweenId, CompiledTween>,
    tracks: IndexVec<TrackId, CompiledTrack>,
    outputs: IndexVec<OutputId, CompiledOutput>,
    events: IndexVec<TimelineNodeId, CompiledEvent>,
    domains: IndexVec<TimeDomainId, CompiledTimeDomain>,
}

pub(crate) struct CompiledAnimationParts {
    pub extent: TimeExtent,
    pub settings: PlaybackSettings,
    pub targets: IndexVec<TargetId, CompiledTarget>,
    pub properties: IndexVec<PropertyId, CompiledProperty>,
    pub tweens: IndexVec<TweenId, CompiledTween>,
    pub tracks: IndexVec<TrackId, CompiledTrack>,
    pub outputs: IndexVec<OutputId, CompiledOutput>,
    pub events: IndexVec<TimelineNodeId, CompiledEvent>,
    pub domains: IndexVec<TimeDomainId, CompiledTimeDomain>,
}

impl CompiledAnimation {
    pub const fn extent(&self) -> TimeExtent {
        self.extent
    }

    pub const fn settings(&self) -> &PlaybackSettings {
        &self.settings
    }

    pub fn targets(&self) -> &oxc_index::IndexSlice<TargetId, [CompiledTarget]> {
        &self.targets
    }

    pub fn properties(&self) -> &oxc_index::IndexSlice<PropertyId, [CompiledProperty]> {
        &self.properties
    }

    pub fn tweens(&self) -> &oxc_index::IndexSlice<TweenId, [CompiledTween]> {
        &self.tweens
    }

    pub fn tracks(&self) -> &oxc_index::IndexSlice<TrackId, [CompiledTrack]> {
        &self.tracks
    }

    pub fn outputs(&self) -> &oxc_index::IndexSlice<OutputId, [CompiledOutput]> {
        &self.outputs
    }

    pub fn events(&self) -> &oxc_index::IndexSlice<TimelineNodeId, [CompiledEvent]> {
        &self.events
    }

    pub fn domains(&self) -> &oxc_index::IndexSlice<TimeDomainId, [CompiledTimeDomain]> {
        &self.domains
    }

    pub fn deterministic_trace(&self) -> Box<str> {
        format!("arkit-animation-plan-v1\n{self:#?}").into_boxed_str()
    }

    pub(crate) fn from_parts(parts: CompiledAnimationParts) -> Arc<Self> {
        Arc::new(Self {
            extent: parts.extent,
            settings: parts.settings,
            targets: parts.targets,
            properties: parts.properties,
            tweens: parts.tweens,
            tracks: parts.tracks,
            outputs: parts.outputs,
            events: parts.events,
            domains: parts.domains,
        })
    }

    #[cfg(test)]
    fn test_empty(extent: TimeExtent) -> Self {
        Self {
            extent,
            settings: PlaybackSettings::default(),
            targets: IndexVec::new(),
            properties: IndexVec::new(),
            tweens: IndexVec::new(),
            tracks: IndexVec::new(),
            outputs: IndexVec::new(),
            events: IndexVec::new(),
            domains: IndexVec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compiled_plan_exposes_read_only_dense_slices() {
        let plan = CompiledAnimation::test_empty(TimeExtent::Finite(TimeSpan::from_millis(300)));
        assert_eq!(
            plan.extent(),
            TimeExtent::Finite(TimeSpan::from_millis(300))
        );
        assert!(plan.targets().is_empty());
        assert!(plan.properties().is_empty());
        assert!(plan.tweens().is_empty());
        assert!(plan.tracks().is_empty());
        assert!(plan.outputs().is_empty());
        assert!(plan.events().is_empty());
        assert!(plan.domains().is_empty());
    }

    #[test]
    fn trace_has_a_versioned_stable_prefix() {
        let plan = CompiledAnimation::test_empty(TimeExtent::ZERO);
        assert!(plan
            .deterministic_trace()
            .starts_with("arkit-animation-plan-v1\n"));
    }
}
