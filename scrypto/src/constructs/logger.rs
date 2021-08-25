use crate::constructs::*;
use crate::kernel::*;
use crate::rust::borrow::ToOwned;
use crate::rust::string::String;

/// A logger for dumping messages.
#[derive(Debug)]
pub struct Logger {}

impl Logger {
    pub fn log(level: Level, message: String) {
        let input = EmitLogInput {
            level: match level {
                Level::Error => "ERROR".to_owned(),
                Level::Warn => "WARN".to_owned(),
                Level::Info => "INFO".to_owned(),
                Level::Debug => "DEBUG".to_owned(),
                Level::Trace => "TRACE".to_owned(),
            },
            message,
        };
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
