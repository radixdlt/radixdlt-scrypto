use crate::*;
use sbor::rust::fmt;
use sbor::rust::fmt::Debug;

/// Represents the level of a log message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Sbor)]
pub enum Level {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Level::Error => write!(f, "ERROR"),
            Level::Warn => write!(f, "WARN"),
            Level::Info => write!(f, "INFO"),
            Level::Debug => write!(f, "DEBUG"),
            Level::Trace => write!(f, "TRACE"),
        }
    }
}
