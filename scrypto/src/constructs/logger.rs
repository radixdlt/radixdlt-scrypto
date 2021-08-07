extern crate alloc;
use alloc::string::String;

use crate::kernel::*;
use crate::types::*;

/// A logger for dumping messages.
pub struct Logger {}

impl Logger {
    pub fn log(level: Level, message: String) {
        let input = EmitLogInput { level, message };
        let _: EmitLogOutput = call_kernel(EMIT_LOG, input);
    }

    pub fn trace(message: String) {
        Self::log(Level::Trace, message);
    }

    pub fn debug(message: String) {
        Self::log(Level::Debug, message);
    }

    pub fn info(message: String) {
        Self::log(Level::Info, message);
    }

    pub fn warn(message: String) {
        Self::log(Level::Warn, message);
    }

    pub fn error(message: String) {
        Self::log(Level::Error, message);
    }
}
