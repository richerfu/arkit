//! Commands queued for deterministic processing by the root animation engine.

use crate::{InstanceId, PlaybackRate, TimePoint, TimeSpan};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SeekMode {
    SuppressEvents,
    FireCrossingEvents,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EngineCommand {
    Play(InstanceId),
    Pause(InstanceId),
    Resume(InstanceId),
    Restart(InstanceId),
    Reverse(InstanceId),
    SetAlternate {
        instance: InstanceId,
        enabled: bool,
    },
    Seek {
        instance: InstanceId,
        position: TimePoint,
        mode: SeekMode,
    },
    Complete(InstanceId),
    Cancel(InstanceId),
    Reset(InstanceId),
    Revert(InstanceId),
    Stretch {
        instance: InstanceId,
        duration: TimeSpan,
    },
    Refresh(InstanceId),
    SetPlaybackRate {
        instance: InstanceId,
        rate: PlaybackRate,
    },
    Remove(InstanceId),
}
