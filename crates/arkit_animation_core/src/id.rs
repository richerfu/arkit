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
