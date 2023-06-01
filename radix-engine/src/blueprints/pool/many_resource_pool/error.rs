use crate::errors::ApplicationError;
use crate::errors::RuntimeError;
use radix_engine_common::types::*;
use radix_engine_common::ScryptoSbor;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum ManyResourcePoolError {
    NonFungibleResourcesAreNotAccepted {
        resource_address: ResourceAddress,
    },
    NonZeroPoolUnitSupplyButZeroReserves,
    InvalidPoolUnitResource {
        expected: ResourceAddress,
        actual: ResourceAddress,
    },
    ResourceDoesNotBelongToPool {
        resource_address: ResourceAddress,
    },
    PoolCreationWithSameResource,
    ContributionOfEmptyBucketError,
}

impl From<ManyResourcePoolError> for RuntimeError {
    fn from(error: ManyResourcePoolError) -> Self {
        Self::ApplicationError(ApplicationError::ManyResourcePoolError(error))
    }
}
