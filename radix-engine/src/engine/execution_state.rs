use sbor::rust::string::String;
use scrypto::core::Receiver;
use scrypto::engine::types::*;
use scrypto::prelude::TypeName;

use crate::model::*;

pub enum ExecutionEntity<'a> {
    Function(TypeName),
    Method(Receiver, ExecutionState<'a>),
}

pub enum ExecutionState<'a> {
    Consumed(RENodeId),
    AuthZone(&'a mut AuthZone),
    RENodeRef(RENodeId),
    // TODO: Can remove this and replace useage with REActor
    Component(PackageAddress, String, ComponentAddress),
}
