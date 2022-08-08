use sbor::rust::string::ToString;
use sbor::rust::vec::Vec;
use sbor::*;

use crate::engine::types::{PackageAddress, RENodeId};

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub enum TypeName {
    Package,
    ResourceManager,
    TransactionProcessor,
    Blueprint(PackageAddress, String),
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub enum Receiver {
    Consumed(RENodeId),
    NativeRENodeRef(RENodeId),
    Scrypto(RENodeId),
    AuthZoneRef,
}

// TODO: Remove and replace with real HeapRENodes
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub enum ScryptoRENode {
    Component(PackageAddress, String, Vec<u8>),
    KeyValueStore,
}
