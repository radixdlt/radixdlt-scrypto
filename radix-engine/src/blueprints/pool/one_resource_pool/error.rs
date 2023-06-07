use crate::errors::ApplicationError;
use crate::errors::RuntimeError;
use radix_engine_common::types::*;
use radix_engine_common::ScryptoSbor;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum OneResourcePoolError {
    NonFungibleResourcesAreNotAccepted {
        resource_address: ResourceAddress,
    },
    NonZeroPoolUnitSupplyButZeroReserves,
    InvalidPoolUnitResource {
        expected: ResourceAddress,
        actual: ResourceAddress,
    },
    ContributionOfEmptyBucketError,
}

impl From<OneResourcePoolError> for RuntimeError {
    fn from(error: OneResourcePoolError) -> Self {
        Self::ApplicationError(ApplicationError::OneResourcePoolError(error))
    }
}
