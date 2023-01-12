use crate::types::*;
use radix_engine_interface::api::types::{PackageId, RENodeId, ResourceManagerId};

// TODO: clean up after `Owned(RENodeId)`?
#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum GlobalAddressSubstate {
    Component(ComponentId),
    Resource(ResourceManagerId),
    Package(PackageId),
    EpochManager(EpochManagerId),
    Clock(ClockId),
}

impl GlobalAddressSubstate {
    pub fn node_deref(&self) -> RENodeId {
        match self {
            GlobalAddressSubstate::Component(id) => RENodeId::Component(*id),
            GlobalAddressSubstate::Resource(id) => RENodeId::ResourceManager(*id),
            GlobalAddressSubstate::Package(id) => RENodeId::Package(*id),
            GlobalAddressSubstate::EpochManager(id) => RENodeId::EpochManager(*id),
            GlobalAddressSubstate::Clock(id) => RENodeId::Clock(*id),
        }
    }
}
