use crate::model::SysCallTrace;
use crate::types::*;

#[derive(Debug, Clone)]
#[scrypto(TypeId, Encode, Decode)]
pub enum TrackedEvent {
    SysCallTrace(SysCallTrace),
}
