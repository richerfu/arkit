//! Commands queued for deterministic processing by the root animation engine.

use crate::{
    AdapterId, AdapterPropertyId, AdapterTargetId, InstanceKey, PlaybackRate, TimePoint, TimeSpan,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SeekMode {
    SuppressEvents,
    FireCrossingEvents,
}

/// One adapter output sampled at an independent timeline position.
///
/// This keeps two-dimensional gesture mapping inside the root Engine without
/// rebuilding a timeline or writing native properties from the input event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OutputSeek {
    pub adapter: AdapterId,
    pub target: AdapterTargetId,
    pub property: AdapterPropertyId,
    pub position: TimePoint,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EngineCommand {
    Play(InstanceKey),
    Pause(InstanceKey),
    Resume(InstanceKey),
    Restart(InstanceKey),
    Reverse(InstanceKey),
    SetAlternate {
        instance: InstanceKey,
        enabled: bool,
    },
    Seek {
        instance: InstanceKey,
        position: TimePoint,
        mode: SeekMode,
    },
    /// Advances a running instance from a platform-owned clock while keeping
    /// sampling, crossing events, loops, and terminal state in the engine.
    AdvanceExternal {
        instance: InstanceKey,
        position: TimePoint,
    },
    SeekOutputs {
        instance: InstanceKey,
        first: OutputSeek,
        second: Option<OutputSeek>,
    },
    Complete(InstanceKey),
    Cancel(InstanceKey),
    Reset(InstanceKey),
    Revert(InstanceKey),
    Stretch {
        instance: InstanceKey,
        duration: TimeSpan,
    },
    Refresh(InstanceKey),
    SetPlaybackRate {
        instance: InstanceKey,
        rate: PlaybackRate,
    },
    Remove(InstanceKey),
}
