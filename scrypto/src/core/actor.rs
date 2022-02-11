use sbor::*;

use crate::core::*;
use crate::rust::string::String;

/// Represents the running entity.
#[derive(Debug, Clone, TypeId, Encode, Decode, Describe)]
pub enum Actor {
    Blueprint(PackageRef, String),

    Component(ComponentRef),
}
