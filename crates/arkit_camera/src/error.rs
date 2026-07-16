use std::fmt;

/// Stable error categories exposed by the camera component.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraErrorKind {
    PermissionDenied,
    SurfaceUnavailable,
    NoCamera,
    Unsupported,
    InvalidState,
    Native,
    Image,
    WorkerStopped,
}

/// A CameraKit, surface, image, or lifecycle failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CameraError {
    kind: CameraErrorKind,
    operation: &'static str,
    native_code: Option<u32>,
    message: String,
}

impl CameraError {
    pub fn kind(&self) -> CameraErrorKind {
        self.kind
    }

    pub fn operation(&self) -> &'static str {
        self.operation
    }

    pub fn native_code(&self) -> Option<u32> {
        self.native_code
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub(crate) fn permission_denied() -> Self {
        Self {
            kind: CameraErrorKind::PermissionDenied,
            operation: "OH_AT_CheckSelfPermission",
            native_code: None,
            message: "ohos.permission.CAMERA has not been granted".into(),
        }
    }

    pub(crate) fn invalid_state(operation: &'static str, message: impl Into<String>) -> Self {
        Self {
            kind: CameraErrorKind::InvalidState,
            operation,
            native_code: None,
            message: message.into(),
        }
    }

    pub(crate) fn unsupported(operation: &'static str, message: impl Into<String>) -> Self {
        Self {
            kind: CameraErrorKind::Unsupported,
            operation,
            native_code: None,
            message: message.into(),
        }
    }

    pub(crate) fn worker_stopped(operation: &'static str) -> Self {
        Self {
            kind: CameraErrorKind::WorkerStopped,
            operation,
            native_code: None,
            message: "camera worker has stopped".into(),
        }
    }

    pub(crate) fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }
}

impl fmt::Display for CameraError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for CameraError {}

impl From<ohos_camera_binding::CameraError> for CameraError {
    fn from(error: ohos_camera_binding::CameraError) -> Self {
        let kind = match error.kind() {
            ohos_camera_binding::CameraErrorKind::NoCamera => CameraErrorKind::NoCamera,
            ohos_camera_binding::CameraErrorKind::Unsupported => CameraErrorKind::Unsupported,
            ohos_camera_binding::CameraErrorKind::InvalidState => CameraErrorKind::InvalidState,
            ohos_camera_binding::CameraErrorKind::Native => CameraErrorKind::Native,
            ohos_camera_binding::CameraErrorKind::Image => CameraErrorKind::Image,
            ohos_camera_binding::CameraErrorKind::Surface => CameraErrorKind::SurfaceUnavailable,
        };
        Self {
            kind,
            operation: error.operation(),
            native_code: error.native_code(),
            message: error.message().to_owned(),
        }
    }
}

pub type CameraResult<T> = Result<T, CameraError>;
