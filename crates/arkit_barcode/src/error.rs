//! Error types for barcode encoding and export.

use std::error::Error;
use std::fmt;
use std::path::PathBuf;

/// Convenient result alias for barcode APIs.
pub type BarcodeResult<T> = Result<T, BarcodeError>;

/// Stable error classification for barcode operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BarcodeErrorKind {
    EmptyContents,
    InvalidContents,
    UnsupportedFormat,
    InvalidDimensions,
    EncodeFailed,
    RenderFailed,
    Io,
}

/// Public barcode error with a stable [`BarcodeErrorKind`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BarcodeError {
    pub kind: BarcodeErrorKind,
    message: String,
}

impl BarcodeError {
    pub fn new(kind: BarcodeErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    pub fn empty_contents() -> Self {
        Self::new(
            BarcodeErrorKind::EmptyContents,
            "barcode contents are empty",
        )
    }

    pub fn invalid_contents(message: impl Into<String>) -> Self {
        Self::new(BarcodeErrorKind::InvalidContents, message)
    }

    pub fn invalid_dimensions(message: impl Into<String>) -> Self {
        Self::new(BarcodeErrorKind::InvalidDimensions, message)
    }

    pub fn encode_failed(message: impl Into<String>) -> Self {
        Self::new(BarcodeErrorKind::EncodeFailed, message)
    }

    pub fn render_failed(message: impl Into<String>) -> Self {
        Self::new(BarcodeErrorKind::RenderFailed, message)
    }

    pub fn io(operation: &'static str, path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        let path = path.into();
        Self::new(
            BarcodeErrorKind::Io,
            format!("{operation} {}: {source}", path.display()),
        )
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for BarcodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for BarcodeError {}
