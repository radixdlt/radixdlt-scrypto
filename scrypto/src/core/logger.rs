use crate::kernel::*;
use crate::rust::string::String;

/// A utility for logging messages.
#[derive(Debug)]
pub struct Logger {}

impl Logger {
    pub fn log(level: LogLevel, message: String) {
        let input = EmitLogInput { level, message };
        let _: EmitLogOutput = call_kernel(EMIT_LOG, input);
    }

    pub fn trace(message: String) {
        Self::log(LogLevel::Trace, message);
    }

    pub fn debug(message: String) {
        Self::log(LogLevel::Debug, message);
    }

    pub fn info(message: String) {
        Self::log(LogLevel::Info, message);
    }

    pub fn warn(message: String) {
        Self::log(LogLevel::Warn, message);
    }

    pub fn error(message: String) {
        Self::log(LogLevel::Error, message);
    }
}
