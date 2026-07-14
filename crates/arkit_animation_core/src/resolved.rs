//! Adapter-resolved animation inputs with dense target and property identity.

use oxc_index::IndexVec;

use std::num::NonZeroU32;

use crate::{
    AdapterId, AdapterPropertyId, AdapterTargetId, AnimationValue, CallId, CallPolicy, Composition,
    Easing, Modifier, PlaybackSettings, PropertyDescriptor, PropertyId, TargetId, TimeDomainId,
    TimeExtent, TimePoint, TimeSpan, TimelineNodeId, TweenId,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResolvedTarget {
    pub adapter: AdapterId,
    pub adapter_target: AdapterTargetId,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedProperty {
    pub adapter: AdapterId,
    pub adapter_property: AdapterPropertyId,
    pub descriptor: PropertyDescriptor,
}

#[derive(Debug, Clone)]
pub struct ResolvedTween {
    pub domain: TimeDomainId,
    pub target: TargetId,
    pub property: PropertyId,
    pub start: TimePoint,
    pub delay: TimeSpan,
    pub duration: TimeSpan,
    pub priority: i32,
    pub from: AnimationValue,
    pub to: AnimationValue,
    pub easing: Easing,
    pub composition: Composition,
    pub modifier: Modifier,
}

#[derive(Debug, Clone)]
pub enum ResolvedEvent {
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
pub struct ResolvedTimeDomain {
    pub parent: Option<TimeDomainId>,
    pub offset: TimePoint,
    pub extent: TimeExtent,
    pub settings: PlaybackSettings,
}

#[derive(Debug, Clone)]
pub struct ResolvedAnimation {
    pub domains: IndexVec<TimeDomainId, ResolvedTimeDomain>,
    pub targets: IndexVec<TargetId, ResolvedTarget>,
    pub properties: IndexVec<PropertyId, ResolvedProperty>,
    pub tweens: IndexVec<TweenId, ResolvedTween>,
    pub events: IndexVec<TimelineNodeId, ResolvedEvent>,
    pub settings: PlaybackSettings,
}

impl Default for ResolvedAnimation {
    fn default() -> Self {
        let settings = PlaybackSettings::default();
        let mut domains = IndexVec::new();
        domains.push(ResolvedTimeDomain {
            parent: None,
            offset: TimePoint::ZERO,
            extent: TimeExtent::ZERO,
            settings: settings.clone(),
        });
        Self {
            domains,
            targets: IndexVec::new(),
            properties: IndexVec::new(),
            tweens: IndexVec::new(),
            events: IndexVec::new(),
            settings,
        }
    }
}
