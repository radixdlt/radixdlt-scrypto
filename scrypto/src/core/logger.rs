use crate::engine::*;
use crate::rust::string::String;

/// A utility for logging messages.
#[derive(Debug)]
pub struct Logger {}

impl Logger {
    /// Emits a log to console.
    pub fn log(level: LogLevel, message: String) {
        let input = EmitLogInput { level, message };
        let _: EmitLogOutput = call_engine(EMIT_LOG, input);
    }

    /// Emits a trace message.
    pub fn trace(message: String) {
        Self::log(LogLevel::Trace, message);
    }

    /// Emits a debug message.
    pub fn debug(message: String) {
        Self::log(LogLevel::Debug, message);
    }

    /// Emits an info message.
    pub fn info(message: String) {
        Self::log(LogLevel::Info, message);
    }

    /// Emits a warn message.
    pub fn warn(message: String) {
        Self::log(LogLevel::Warn, message);
    }

    /// Emits an error message.
    pub fn error(message: String) {
        Self::log(LogLevel::Error, message);
    }
}
