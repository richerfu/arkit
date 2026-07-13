//! Allocation-free runtime cursors over precompiled property-track segments.

use crate::{CompiledTrack, TimePoint, TrackSegmentId};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TrackCursor {
    segment: Option<TrackSegmentId>,
}

impl TrackCursor {
    pub const fn segment(self) -> Option<TrackSegmentId> {
        self.segment
    }

    pub fn seek(&mut self, track: &CompiledTrack, at: TimePoint) -> Option<TrackSegmentId> {
        self.segment = track.seek_segment(at);
        self.segment
    }

    pub fn advance(&mut self, track: &CompiledTrack, at: TimePoint) -> Option<TrackSegmentId> {
        let Some(mut current) = self.segment else {
            return self.seek(track, at);
        };
        if track.segments()[current].start > at {
            return self.seek(track, at);
        }

        loop {
            let next_index = current.index() + 1;
            let Some(next) = track.segments().raw.get(next_index) else {
                break;
            };
            if next.start > at {
                break;
            }
            current = TrackSegmentId::new(next_index);
        }
        self.segment = Some(current);
        self.segment
    }

    pub fn reset(&mut self) {
        self.segment = None;
    }
}

#[cfg(test)]
mod tests {
    use oxc_index::IndexVec;

    use super::*;
    use crate::{CompiledTrackSegment, PropertyId, TargetId, TimeDomainId, TweenId};

    fn point(nanos: u64) -> TimePoint {
        TimePoint::from_nanos(nanos)
    }

    fn track() -> CompiledTrack {
        let segments = IndexVec::from_vec(vec![
            CompiledTrackSegment::new(
                point(10),
                point(20),
                Some(TweenId::new(0)),
                Box::new([]),
                Box::new([]),
            ),
            CompiledTrackSegment::new(
                point(20),
                point(30),
                Some(TweenId::new(0)),
                Box::new([]),
                Box::new([]),
            ),
            CompiledTrackSegment::new(
                point(30),
                point(30),
                Some(TweenId::new(1)),
                Box::new([]),
                Box::new([]),
            ),
        ]);
        CompiledTrack::new(
            TimeDomainId::new(0),
            TargetId::new(0),
            PropertyId::new(0),
            Box::new([TweenId::new(0), TweenId::new(1)]),
            segments.into_boxed_slice(),
        )
    }

    #[test]
    fn cursor_seeks_and_advances_without_researching_forward_frames() {
        let track = track();
        let mut cursor = TrackCursor::default();
        assert_eq!(cursor.seek(&track, point(5)), None);
        assert_eq!(
            cursor.advance(&track, point(10)),
            Some(TrackSegmentId::new(0))
        );
        assert_eq!(
            cursor.advance(&track, point(29)),
            Some(TrackSegmentId::new(1))
        );
        assert_eq!(
            cursor.advance(&track, point(30)),
            Some(TrackSegmentId::new(2))
        );
        assert_eq!(
            cursor.advance(&track, point(15)),
            Some(TrackSegmentId::new(0))
        );
        cursor.reset();
        assert_eq!(cursor.segment(), None);
    }
}
