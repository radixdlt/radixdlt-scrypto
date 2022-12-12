use crate::model::SysCallTrace;
use crate::types::*;

#[derive(Debug)]
pub enum Event {
    Tracked(TrackedEvent),
}

#[derive(Debug, Clone)]
#[scrypto(TypeId, Encode, Decode)]
pub enum TrackedEvent {
    SysCallTrace(SysCallTrace),
}
