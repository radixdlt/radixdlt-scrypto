use crate::types::*;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct LoggerSubstate {
    pub logs: Vec<(Level, String)>,
}
