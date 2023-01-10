use crate::model::SysCallTrace;
use crate::types::*;

#[derive(Debug, Clone)]
#[scrypto(Categorize, Encode, Decode)]
pub enum TrackedEvent {
    SysCallTrace(SysCallTrace),
}
