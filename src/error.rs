//! Typed errors. The adapter maps each variant onto a Python exception kind name.

use std::fmt;

/// Everything the library can reject.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MazeError {
    /// A precondition on the caller's input failed (Python: `AssertionError`).
    InvalidInput(String),
    /// A value is out of its domain, e.g. a seed or an empty random range (Python: `ValueError`).
    InvalidValue(String),
    /// Choosing from an empty sequence (Python: `IndexError`).
    Empty(String),
}

impl MazeError {
    pub(crate) fn input(msg: impl Into<String>) -> Self {
        MazeError::InvalidInput(msg.into())
    }

    pub(crate) fn value(msg: impl Into<String>) -> Self {
        MazeError::InvalidValue(msg.into())
    }

    pub(crate) fn empty(msg: impl Into<String>) -> Self {
        MazeError::Empty(msg.into())
    }

    /// The human-readable message.
    pub fn message(&self) -> &str {
        match self {
            MazeError::InvalidInput(m) | MazeError::InvalidValue(m) | MazeError::Empty(m) => m,
        }
    }
}

impl fmt::Display for MazeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.message())
    }
}

impl std::error::Error for MazeError {}

/// Library result type.
pub type Result<T> = std::result::Result<T, MazeError>;
