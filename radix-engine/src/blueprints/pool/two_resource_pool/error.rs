use crate::errors::ApplicationError;
use crate::errors::RuntimeError;
use radix_engine_common::types::*;
use radix_engine_common::ScryptoSbor;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum TwoResourcePoolError {
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

impl From<TwoResourcePoolError> for RuntimeError {
    fn from(error: TwoResourcePoolError) -> Self {
        Self::ApplicationError(ApplicationError::TwoResourcePoolError(error))
    }
}
