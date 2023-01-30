use radix_engine_interface::api::blueprints::logger::Level;

use crate::types::*;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct LoggerSubstate {
    pub logs: Vec<(Level, String)>,
}
