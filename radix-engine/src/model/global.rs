use crate::types::*;

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub enum GlobalRENode {
    Component(ComponentAddress),
    Package(PackageAddress),
    Resource(ResourceAddress),
}

impl GlobalRENode {
    pub fn node_id(&self) -> RENodeId {
        match self {
            GlobalRENode::Package(package_address) => RENodeId::Package(*package_address),
            GlobalRENode::Component(component_address) => RENodeId::Component(*component_address),
            GlobalRENode::Resource(resource_address) => RENodeId::ResourceManager(*resource_address),
        }
    }
}
