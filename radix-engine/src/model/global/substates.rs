use crate::types::*;
use radix_engine_interface::api::types::{EpochManagerId, PackageId, RENodeId, ResourceManagerId};

// TODO: clean up after `Owned(RENodeId)`?
#[derive(Debug, Clone, PartialEq, Eq)]
#[scrypto(TypeId, Encode, Decode)]
pub enum GlobalAddressSubstate {
    Component(scrypto::component::Component),
    Resource(ResourceManagerId),
    Package(PackageId),
    // We may consider using another enum here so we can map system addresses to nodes of different type.
    System(EpochManagerId),
}

impl GlobalAddressSubstate {
    pub fn node_deref(&self) -> RENodeId {
        match self {
            GlobalAddressSubstate::Component(component) => RENodeId::Component(component.0),
            GlobalAddressSubstate::Resource(id) => RENodeId::ResourceManager(*id),
            GlobalAddressSubstate::Package(id) => RENodeId::Package(*id),
            GlobalAddressSubstate::System(id) => RENodeId::EpochManager(*id),
        }
    }
}
