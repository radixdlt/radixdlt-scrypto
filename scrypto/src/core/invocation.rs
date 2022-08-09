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
    Scrypto {
        package_address: PackageAddress,
        blueprint_name: String,
        method_name: String,
    },
    Native(String),
}

impl Function {
    pub fn fn_ident(&self) -> &str {
        match self {
            Function::Scrypto { method_name, .. } | Function::Native(method_name) => &method_name,
        }
    }
}

// TODO: Remove and replace with real HeapRENodes
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub enum ScryptoRENode {
    Component(PackageAddress, String, Vec<u8>),
    KeyValueStore,
}
