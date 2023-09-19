use crate::engine::scrypto_env::ScryptoVmV1Api;
use radix_engine_interface::types::Level;
use sbor::rust::string::String;

/// A utility for logging messages.
#[derive(Debug)]
pub struct Logger {}

impl Logger {
    /// Emits a TRACE message.
    pub fn trace(message: String) {
        ScryptoVmV1Api::sys_log(Level::Trace, message);
    }

    /// Emits a DEBUG message.
    pub fn debug(message: String) {
        ScryptoVmV1Api::sys_log(Level::Debug, message);
    }

    /// Emits an INFO message.
    pub fn info(message: String) {
        ScryptoVmV1Api::sys_log(Level::Info, message);
    }

    /// Emits a WARN message.
    pub fn warn(message: String) {
        ScryptoVmV1Api::sys_log(Level::Warn, message);
    }

    /// Emits an ERROR message.
    pub fn error(message: String) {
        ScryptoVmV1Api::sys_log(Level::Error, message);
    }
}
