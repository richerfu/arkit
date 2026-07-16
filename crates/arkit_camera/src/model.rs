use crate::{CameraError, CameraPosition, CameraSessionInfo, CameraSize};

/// Declarative output-profile selection for [`crate::CameraPreview`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CameraProfileSelection {
    pub preview_size: Option<CameraSize>,
    pub photo_size: Option<CameraSize>,
}

/// Observable lifecycle state for [`crate::CameraPreview`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum CameraStatus {
    #[default]
    Idle,
    WaitingForSurface,
    Starting(CameraPosition),
    Running(CameraSessionInfo),
    Capturing(CameraSessionInfo),
    Stopped,
    PermissionDenied,
    Unavailable,
    Error(CameraError),
}

impl CameraStatus {
    pub fn session(&self) -> Option<CameraSessionInfo> {
        match self {
            Self::Running(info) | Self::Capturing(info) => Some(*info),
            _ => None,
        }
    }

    pub fn is_running(&self) -> bool {
        matches!(self, Self::Running(_) | Self::Capturing(_))
    }
}
