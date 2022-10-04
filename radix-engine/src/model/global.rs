use crate::types::*;

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub enum GlobalAddressSubstate {
    Component(scrypto::component::Component),
    // TODO: Decide whether these should also be wrapped
    /*
    Package(PackageAddress),
    Resource(ResourceAddress),
     */
}

#[derive(Debug)]
pub struct GlobalRENode {
    pub address: GlobalAddressSubstate,
}

impl GlobalRENode {
    pub fn node_deref(&self) -> RENodeId {
        match &self.address {
            GlobalAddressSubstate::Component(component) => RENodeId::Component(component.0),
            /*
            GlobalRENode::Package(package_address) => RENodeId::Package(*package_address),
            GlobalRENode::Resource(resource_address) => {
                RENodeId::ResourceManager(*resource_address)
            }
             */
        }
    }
}
