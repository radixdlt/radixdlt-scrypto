use crate::types::*;

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub enum GlobalRENode {
    Component(scrypto::component::Component),
    Resource(ResourceAddress),
    Package(PackageAddress),
}

impl GlobalRENode {
    pub fn node_deref(&self) -> RENodeId {
        match self {
            GlobalRENode::Component(component) => RENodeId::Component(component.0),
            GlobalRENode::Resource(resource_address) => {
                RENodeId::ResourceManager(*resource_address)
            }
            GlobalRENode::Package(package_address) => RENodeId::Package(*package_address),
        }
    }
}
