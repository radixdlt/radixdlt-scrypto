use crate::types::*;

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub enum GlobalAddressSubstate {
    Component(scrypto::component::Component),
    SystemComponent(scrypto::component::Component),
    Resource(ResourceManagerId),
    Package(PackageId),
}

impl GlobalAddressSubstate {
    pub fn node_deref(&self) -> RENodeId {
        match self {
            GlobalAddressSubstate::Component(component) => RENodeId::Component(component.0),
            GlobalAddressSubstate::SystemComponent(component) => RENodeId::System(component.0),
            GlobalAddressSubstate::Resource(id) => RENodeId::ResourceManager(*id),
            GlobalAddressSubstate::Package(id) => RENodeId::Package(*id),
        }
    }
}

#[derive(Debug)]
pub struct GlobalRENode {
    pub address: GlobalAddressSubstate,
}

impl GlobalRENode {
    pub fn node_deref(&self) -> RENodeId {
        self.address.node_deref()
    }
}
