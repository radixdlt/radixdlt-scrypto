use crate::types::*;

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub enum GlobalAddressSubstate {
    Component(scrypto::component::Component),
    SystemComponent(scrypto::component::Component),
    Resource(ResourceAddress),
    Package(PackageAddress),
}

#[derive(Debug)]
pub struct GlobalRENode {
    pub address: GlobalAddressSubstate,
}

impl GlobalRENode {
    pub fn node_deref(&self) -> RENodeId {
        match &self.address {
            GlobalAddressSubstate::Component(component) => RENodeId::Component(component.0),
            GlobalAddressSubstate::SystemComponent(component) => RENodeId::System(component.0),
            GlobalAddressSubstate::Resource(resource_address) => {
                RENodeId::ResourceManager(*resource_address)
            }
            GlobalAddressSubstate::Package(package_address) => RENodeId::Package(*package_address),
        }
    }
}
