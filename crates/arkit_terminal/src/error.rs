//! Terminal errors.

use std::fmt;

/// Error surface for terminal creation, VT write, and format.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalError {
    kind: TerminalErrorKind,
    message: String,
}

/// Classifies a [`TerminalError`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalErrorKind {
    Engine,
    Format,
    InvalidSize,
    Io,
}

impl TerminalError {
    pub fn new(kind: TerminalErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    pub fn kind(&self) -> TerminalErrorKind {
        self.kind
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for TerminalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {}", self.kind, self.message)
    }
}

impl std::error::Error for TerminalError {}

/// Result alias for terminal operations.
pub type TerminalResult<T> = Result<T, TerminalError>;
