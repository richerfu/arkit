use std::fmt::{Display, Formatter};
use std::sync::Arc;

/// Stable error categories exposed by the Lottie component.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum LottieErrorKind {
    InvalidSource,
    InvalidConfiguration,
    Network,
    SurfaceUnavailable,
    UnsupportedPixelFormat,
    Render,
    WorkerStopped,
}

/// An owned Lottie error that can cross the render-worker boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LottieError {
    kind: LottieErrorKind,
    operation: &'static str,
    message: Arc<str>,
}

impl LottieError {
    pub(crate) fn new(
        kind: LottieErrorKind,
        operation: &'static str,
        message: impl Into<Arc<str>>,
    ) -> Self {
        Self {
            kind,
            operation,
            message: message.into(),
        }
    }

    #[cfg_attr(not(target_env = "ohos"), allow(dead_code))]
    pub(crate) fn invalid_source(operation: &'static str, message: impl Into<Arc<str>>) -> Self {
        Self::new(LottieErrorKind::InvalidSource, operation, message)
    }

    #[cfg_attr(not(target_env = "ohos"), allow(dead_code))]
    pub(crate) fn invalid_configuration(
        operation: &'static str,
        message: impl Into<Arc<str>>,
    ) -> Self {
        Self::new(LottieErrorKind::InvalidConfiguration, operation, message)
    }

    pub(crate) fn render(operation: &'static str, message: impl Into<Arc<str>>) -> Self {
        Self::new(LottieErrorKind::Render, operation, message)
    }

    pub(crate) fn network(operation: &'static str, message: impl Into<Arc<str>>) -> Self {
        Self::new(LottieErrorKind::Network, operation, message)
    }

    pub(crate) fn worker_stopped(operation: &'static str) -> Self {
        Self::new(
            LottieErrorKind::WorkerStopped,
            operation,
            "the Lottie render worker is no longer running",
        )
    }

    pub fn kind(&self) -> LottieErrorKind {
        self.kind
    }

    pub fn operation(&self) -> &'static str {
        self.operation
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl Display for LottieError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "{} failed ({:?}): {}",
            self.operation, self.kind, self.message
        )
    }
}

impl std::error::Error for LottieError {}

pub type LottieResult<T> = Result<T, LottieError>;
