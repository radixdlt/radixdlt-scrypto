use scrypto::types::*;
use sbor::*;

/// Represents the running entity.
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub enum Actor {
    Blueprint(Address, String),

    Component(Address),
}
