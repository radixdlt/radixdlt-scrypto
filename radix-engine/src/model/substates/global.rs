use crate::types::*;

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub enum GlobalAddressSubstate {
    Component(scrypto::component::Component),
    Resource(ResourceManagerId),
    Package(PackageId),
    System(EpochManagerId), // TODO: clean up after `Owned(RENodeId)`?
}

impl GlobalAddressSubstate {
    pub fn node_deref(&self) -> RENodeId {
        match self {
            GlobalAddressSubstate::Component(component) => RENodeId::Component(component.0),
            GlobalAddressSubstate::Resource(id) => RENodeId::ResourceManager(*id),
            GlobalAddressSubstate::Package(id) => RENodeId::Package(*id),
            GlobalAddressSubstate::System(id) => RENodeId::System(*id),
        }
    }
}
