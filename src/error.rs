/*! Custom error for this crate. */

use std::{error::Error, fmt, io};

/// Represent all possible error that can happen when opening segment.
#[derive(Debug, PartialEq, Eq)]
pub enum MmapVecError {
    /// Segment was open without any path.
    MissingSegmentPath,

    /// I/O error.
    Io(String),
}

impl From<io::Error> for MmapVecError {
    fn from(value: io::Error) -> Self {
        Self::Io(value.to_string())
    }
}

impl fmt::Display for MmapVecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingSegmentPath => write!(f, "missing segment path"),
            Self::Io(msg) => write!(f, "I/O: {}", msg),
        }
    }
}

impl Error for MmapVecError {}
