use crate::engine::{InvokeError, SystemApi};
use crate::fee::FeeReserve;
use crate::types::*;
use crate::wasm::{WasmEngine, WasmInstance};


#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub enum GlobalRENode {
    Component(scrypto::component::Component),
    Package(PackageAddress),
    Resource(ResourceAddress),
}

impl GlobalRENode {
    pub fn node_id(&self) -> RENodeId {
        match self {
            GlobalRENode::Package(package_address) => RENodeId::Package(*package_address),
            GlobalRENode::Component(component) => RENodeId::Component(component.0),
            GlobalRENode::Resource(resource_address) => RENodeId::ResourceManager(*resource_address),
        }
    }
}