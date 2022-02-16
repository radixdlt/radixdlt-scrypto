use crate::rust::string::String;
use crate::types::*;
use sbor::*;

/// Represents the running entity.
#[derive(Debug, Clone, TypeId, Encode, Decode, Describe)]
pub enum Actor {
    Blueprint(Address, String),

    Component(Address),
}
