use crate::kernel::module::KernelModule;

use radix_engine_interface::types::Level;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;

#[derive(Debug, Clone)]
pub struct LoggerModule(Vec<(Level, String)>);

impl Default for LoggerModule {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl LoggerModule {
    pub fn add_log(&mut self, level: Level, message: String) {
        self.0.push((level, message))
    }

    pub fn logs(self) -> Vec<(Level, String)> {
        self.0
    }
}

impl KernelModule for LoggerModule {}
