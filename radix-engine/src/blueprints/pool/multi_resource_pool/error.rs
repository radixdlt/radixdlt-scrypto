use crate::errors::ApplicationError;
use crate::errors::RuntimeError;
use radix_engine_common::types::*;
use radix_engine_common::ScryptoSbor;
use sbor::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum MultiResourcePoolError {
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
    MissingOrEmptyBuckets {
        resource_addresses: BTreeSet<ResourceAddress>,
    },
    PoolCreationWithSameResource,
    ContributionOfEmptyBucketError,
    CantCreatePoolWithLessThanOneResource,
}

impl From<MultiResourcePoolError> for RuntimeError {
    fn from(error: MultiResourcePoolError) -> Self {
        Self::ApplicationError(ApplicationError::MultiResourcePoolError(error))
    }
}
