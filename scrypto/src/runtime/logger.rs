use sbor::rust::string::String;

/// A utility for logging messages.
#[derive(Debug)]
pub struct Logger {}

#[allow(unused_variables)]
impl Logger {
    /// Emits a trace message.
    pub fn trace(message: String) {
        #[cfg(feature = "log-trace")]
        crate::engine::scrypto_env::ScryptoVmV1Api::sys_log(
            radix_engine_interface::types::Level::Trace,
            message,
        );
    }

    /// Emits a debug message.
    pub fn debug(message: String) {
        #[cfg(feature = "log-debug")]
        crate::engine::scrypto_env::ScryptoVmV1Api::sys_log(
            radix_engine_interface::types::Level::Debug,
            message,
        );
    }

    /// Emits an info message.
    pub fn info(message: String) {
        #[cfg(feature = "log-info")]
        crate::engine::scrypto_env::ScryptoVmV1Api::sys_log(
            radix_engine_interface::types::Level::Info,
            message,
        );
    }

    /// Emits a warn message.
    pub fn warn(message: String) {
        #[cfg(feature = "log-warn")]
        crate::engine::scrypto_env::ScryptoVmV1Api::sys_log(
            radix_engine_interface::types::Level::Warn,
            message,
        );
    }

    /// Emits an error message.
    pub fn error(message: String) {
        #[cfg(feature = "log-error")]
        crate::engine::scrypto_env::ScryptoVmV1Api::sys_log(
            radix_engine_interface::types::Level::Error,
            message,
        );
    }
}
