//! Events emitted after the engine releases mutable runtime state.

use crate::{AnimationRuntimeError, CallId, InstanceKey, PlaybackState, TimePoint};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AnimationOutcome {
    Completed,
    Cancelled,
    Reverted,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EngineEvent {
    Begin {
        instance: InstanceKey,
    },
    BeforeUpdate {
        instance: InstanceKey,
        at: TimePoint,
    },
    Update {
        instance: InstanceKey,
        at: TimePoint,
        progress: f32,
    },
    Render {
        instance: InstanceKey,
        at: TimePoint,
    },
    Loop {
        instance: InstanceKey,
        completed_iterations: u32,
    },
    Pause {
        instance: InstanceKey,
    },
    RefreshRequested {
        instance: InstanceKey,
        at: TimePoint,
    },
    Call {
        instance: InstanceKey,
        call: CallId,
    },
    StateChanged {
        instance: InstanceKey,
        state: PlaybackState,
    },
    Complete {
        instance: InstanceKey,
    },
    Cancel {
        instance: InstanceKey,
    },
    Revert {
        instance: InstanceKey,
    },
    Settled {
        instance: InstanceKey,
        outcome: AnimationOutcome,
    },
    /// The instance no longer exists in the engine. Consumers must invalidate
    /// cached handles before the dense slot can be reused.
    Removed {
        instance: InstanceKey,
    },
    Error {
        instance: InstanceKey,
        error: AnimationRuntimeError,
    },
}
