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
    Ref(RENodeId),
    AuthZoneRef,
}

#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub enum Function {
    Scrypto(String),
    Native(String),
}

impl Function {
    pub fn fn_ident(&self) -> &str {
        match self {
            Function::Scrypto(fn_ident) | Function::Native(fn_ident) => &fn_ident,
        }
    }
}

// TODO: Remove and replace with real HeapRENodes
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub enum ScryptoRENode {
    Component(PackageAddress, String, Vec<u8>),
    KeyValueStore,
}
