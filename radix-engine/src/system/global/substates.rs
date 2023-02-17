use crate::types::*;

// TODO: clean up after `Owned(RENodeId)`?
#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum GlobalSubstate {
    Component(ComponentId),
    Resource(ResourceManagerId),
    Package(PackageId),
    EpochManager(EpochManagerId),
    Validator(ValidatorId),
    Clock(ClockId),
    AccessController(AccessControllerId),
    Identity(IdentityId),
    Account(AccountId),
}

impl GlobalSubstate {
    pub fn node_deref(&self) -> RENodeId {
        match self {
            GlobalSubstate::Component(id) => RENodeId::Component(*id),
            GlobalSubstate::Resource(id) => RENodeId::ResourceManager(*id),
            GlobalSubstate::Package(id) => RENodeId::Package(*id),
            GlobalSubstate::EpochManager(id) => RENodeId::EpochManager(*id),
            GlobalSubstate::Identity(id) => RENodeId::Identity(*id),
            GlobalSubstate::Validator(id) => RENodeId::Validator(*id),
            GlobalSubstate::Clock(id) => RENodeId::Clock(*id),
            GlobalSubstate::Account(id) => RENodeId::Account(*id),
            GlobalSubstate::AccessController(id) => RENodeId::AccessController(*id),
        }
    }
}
