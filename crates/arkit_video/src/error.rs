use std::fmt::{Display, Formatter};
use std::sync::Arc;

use ohos_avplayer_binding::{AvPlayerError, AvPlayerErrorKind};

/// Stable error categories exposed by the video component.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum VideoErrorKind {
    InvalidSource,
    InvalidConfiguration,
    InvalidState,
    SurfaceUnavailable,
    Native,
    WorkerStopped,
}

/// An owned error that can cross the AVPlayer worker boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VideoError {
    kind: VideoErrorKind,
    operation: &'static str,
    native_code: Option<u32>,
    message: Arc<str>,
}

impl VideoError {
    pub(crate) fn new(
        kind: VideoErrorKind,
        operation: &'static str,
        message: impl Into<Arc<str>>,
    ) -> Self {
        Self {
            kind,
            operation,
            native_code: None,
            message: message.into(),
        }
    }

    pub(crate) fn invalid_source(operation: &'static str, message: impl Into<Arc<str>>) -> Self {
        Self::new(VideoErrorKind::InvalidSource, operation, message)
    }

    pub(crate) fn invalid_configuration(
        operation: &'static str,
        message: impl Into<Arc<str>>,
    ) -> Self {
        Self::new(VideoErrorKind::InvalidConfiguration, operation, message)
    }

    pub(crate) fn worker_stopped(operation: &'static str) -> Self {
        Self::new(
            VideoErrorKind::WorkerStopped,
            operation,
            "the video playback worker is no longer running",
        )
    }

    pub fn kind(&self) -> VideoErrorKind {
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
}

impl From<AvPlayerError> for VideoError {
    fn from(error: AvPlayerError) -> Self {
        let kind = match error.kind() {
            AvPlayerErrorKind::InvalidConfiguration => VideoErrorKind::InvalidConfiguration,
            AvPlayerErrorKind::InvalidState => VideoErrorKind::InvalidState,
            AvPlayerErrorKind::Native | AvPlayerErrorKind::Unavailable => VideoErrorKind::Native,
        };
        Self {
            kind,
            operation: error.operation(),
            native_code: error.native_code(),
            message: Arc::from(error.message()),
        }
    }
}

impl Display for VideoError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "{} failed ({:?}): {}",
            self.operation, self.kind, self.message
        )
    }
}

impl std::error::Error for VideoError {}

pub type VideoResult<T> = Result<T, VideoError>;
