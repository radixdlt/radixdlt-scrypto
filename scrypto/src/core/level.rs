use sbor::*;

/// Represents the level of a log message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, TypeId, Encode, Decode, Describe)]
pub enum Level {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}
