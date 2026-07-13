//! Public player state shared by controls, commands, and events.

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum PlaybackState {
    #[default]
    Idle,
    Scheduled,
    Running,
    Paused,
    Completed,
    Cancelled,
    Reverted,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum PlaybackDirection {
    #[default]
    Forward,
    Reverse,
}

impl PlaybackDirection {
    pub const fn reversed(self) -> Self {
        match self {
            Self::Forward => Self::Reverse,
            Self::Reverse => Self::Forward,
        }
    }

    pub const fn sign(self) -> i8 {
        match self {
            Self::Forward => 1,
            Self::Reverse => -1,
        }
    }
}
