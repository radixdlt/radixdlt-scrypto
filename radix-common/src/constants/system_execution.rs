use crate::constants::SYSTEM_EXECUTION_RESOURCE;
use crate::data::scrypto::model::NonFungibleLocalId;
use crate::types::*;

/// For definition @see SYSTEM_EXECUTION_RESOURCE
#[derive(Debug, Clone)]
pub enum SystemExecution {
    Protocol = 0,
    Validator = 1,
}

impl Into<NonFungibleGlobalId> for SystemExecution {
    fn into(self) -> NonFungibleGlobalId {
        NonFungibleGlobalId::new(
            SYSTEM_EXECUTION_RESOURCE,
            NonFungibleLocalId::integer(self as u64),
        )
    }
}
