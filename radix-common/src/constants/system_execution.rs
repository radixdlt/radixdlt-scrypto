use crate::constants::SYSTEM_EXECUTION_RESOURCE;
use crate::data::scrypto::model::NonFungibleLocalId;
use crate::types::*;

/// For definition @see SYSTEM_EXECUTION_RESOURCE
#[derive(Debug, Clone, Copy)]
pub enum SystemExecution {
    Protocol = 0,
    Validator = 1,
}

impl SystemExecution {
    pub fn proof(self) -> NonFungibleGlobalId {
        self.into()
    }
}

impl From<SystemExecution> for NonFungibleGlobalId {
    fn from(val: SystemExecution) -> Self {
        NonFungibleGlobalId::new(
            SYSTEM_EXECUTION_RESOURCE,
            NonFungibleLocalId::integer(val as u64),
        )
    }
}
