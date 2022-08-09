use sbor::rust::string::String;
use scrypto::core::Receiver;
use scrypto::engine::types::*;
use scrypto::prelude::TypeName;

pub enum ExecutionEntity {
    Function(TypeName),
    Method(Receiver, ExecutionState),
}

pub enum ExecutionState {
    Consumed(RENodeId),
    RENodeRef(RENodeId),
    // TODO: Can remove this and replace useage with REActor
    Component(PackageAddress, String, ComponentAddress),
}
