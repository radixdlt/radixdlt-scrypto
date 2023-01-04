use crate::types::*;

#[derive(Debug, Clone, PartialEq, Eq)]
#[scrypto(TypeId, Encode, Decode)]
pub struct LoggerSubstate {
    pub logs: Vec<(Level, String)>,
}
