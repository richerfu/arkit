//! Dense identifiers used by compiled plans and the runtime engine.

oxc_index::define_index_type! {
    pub struct TargetId = u32;
}

oxc_index::define_index_type! {
    pub struct TargetSetId = u32;
}

oxc_index::define_index_type! {
    pub struct PropertyId = u32;
}

oxc_index::define_index_type! {
    pub struct TweenId = u32;
}

oxc_index::define_index_type! {
    pub struct TrackId = u32;
}

oxc_index::define_index_type! {
    pub struct OutputId = u32;
}

oxc_index::define_index_type! {
    pub struct EngineOutputId = u32;
}

oxc_index::define_index_type! {
    pub struct TrackSegmentId = u32;
}

oxc_index::define_index_type! {
    pub struct TimelineNodeId = u32;
}

oxc_index::define_index_type! {
    pub struct TimeDomainId = u32;
}

oxc_index::define_index_type! {
    pub struct LabelId = u32;
}

oxc_index::define_index_type! {
    pub struct CallId = u32;
}

oxc_index::define_index_type! {
    pub struct InstanceId = u32;
}

/// Stable external handle for an animation instance.
///
/// `InstanceId` is a dense storage slot and is intentionally reused. Commands,
/// events, and snapshot lookups therefore carry a generation so delayed work
/// for a retired instance cannot affect a newer occupant of the same slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InstanceKey {
    slot: InstanceId,
    generation: u64,
}

impl InstanceKey {
    pub const fn from_parts(slot: InstanceId, generation: u64) -> Self {
        Self { slot, generation }
    }

    pub const fn slot(self) -> InstanceId {
        self.slot
    }

    pub const fn generation(self) -> u64 {
        self.generation
    }
}

oxc_index::define_index_type! {
    pub struct AdapterId = u32;
}

oxc_index::define_index_type! {
    pub struct AdapterTargetId = u32;
}

oxc_index::define_index_type! {
    pub struct AdapterPropertyId = u32;
}

oxc_index::define_index_type! {
    pub struct ValueFunctionId = u32;
}

oxc_index::define_index_type! {
    pub struct LayoutNodeId = u32;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dense_ids_preserve_distinct_domains() {
        let target = TargetId::new(7);
        let property = PropertyId::new(7);
        assert_eq!(target.index(), property.index());
        assert_eq!(target.index(), 7);
    }
}
