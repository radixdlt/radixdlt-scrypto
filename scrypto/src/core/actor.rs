use sbor::*;

use crate::component::*;
use crate::rust::string::String;

/// Represents the running entity.
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub enum Actor {
    Blueprint(String),

    Component(String, ComponentId),
}
