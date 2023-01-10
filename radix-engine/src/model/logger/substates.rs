use crate::types::*;

#[derive(Debug, Clone, PartialEq, Eq)]
#[scrypto(Categorize, Encode, Decode)]
pub struct LoggerSubstate {
    pub logs: Vec<(Level, String)>,
}
