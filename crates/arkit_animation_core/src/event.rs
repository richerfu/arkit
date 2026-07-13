//! Events emitted after the engine releases mutable runtime state.

use crate::{AnimationRuntimeError, CallId, InstanceId, PlaybackState, TimePoint};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AnimationOutcome {
    Completed,
    Cancelled,
    Reverted,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EngineEvent {
    Begin {
        instance: InstanceId,
    },
    BeforeUpdate {
        instance: InstanceId,
        at: TimePoint,
    },
    Update {
        instance: InstanceId,
        at: TimePoint,
        progress: f32,
    },
    Render {
        instance: InstanceId,
        at: TimePoint,
    },
    Loop {
        instance: InstanceId,
        completed_iterations: u32,
    },
    Pause {
        instance: InstanceId,
    },
    RefreshRequested {
        instance: InstanceId,
        at: TimePoint,
    },
    Call {
        instance: InstanceId,
        call: CallId,
    },
    StateChanged {
        instance: InstanceId,
        state: PlaybackState,
    },
    Complete {
        instance: InstanceId,
    },
    Cancel {
        instance: InstanceId,
    },
    Revert {
        instance: InstanceId,
    },
    Settled {
        instance: InstanceId,
        outcome: AnimationOutcome,
    },
    Error {
        instance: InstanceId,
        error: AnimationRuntimeError,
    },
}
