use crate::sbor::rust::string::String;
use crate::types::Level;

pub trait ClientLoggerApi<E> {
    fn log_message(&mut self, level: Level, message: String) -> Result<(), E>;
}
