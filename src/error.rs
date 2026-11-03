//! Error categories shared by desktop and headless consumers.
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    Invalid(String),
    StaleRevision { expected: u64, actual: u64 },
    RevisionExhausted,
    FileChanged,
    UnsupportedFormat(String),
    Io(String),
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid(message) | Self::Io(message) => f.write_str(message),
            Self::StaleRevision { expected, actual } => write!(
                f,
                "Project changed from revision {expected} to {actual}; retry from the current revision."
            ),
            Self::RevisionExhausted => f.write_str("Revision limit reached."),
            Self::FileChanged => {
                f.write_str("The file changed outside PCBcraft. Save As to a different file.")
            }
            Self::UnsupportedFormat(format) => write!(
                f,
                "Unsupported format '{format}'; the existing project was kept."
            ),
        }
    }
}
impl std::error::Error for Error {}
impl From<&str> for Error {
    fn from(value: &str) -> Self {
        Self::Invalid(value.into())
    }
}
impl From<String> for Error {
    fn from(value: String) -> Self {
        Self::Invalid(value)
    }
}
impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value.to_string())
    }
}
