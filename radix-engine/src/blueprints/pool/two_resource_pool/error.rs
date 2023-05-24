use crate::errors::ApplicationError;
use crate::errors::RuntimeError;
use radix_engine_common::types::*;
use radix_engine_common::ScryptoSbor;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum TwoResourcePoolError {
    PoolsDoNotSupportNonFungibleResources {
        resource_address: ResourceAddress,
    },
    IllegalState,
    InvalidPoolUnitResource {
        expected: ResourceAddress,
        actual: ResourceAddress,
    },
    FailedToFindVaultOfResource {
        resource_address: ResourceAddress,
    },
    ResourceDoesNotBelongToPool {
        resource_address: ResourceAddress,
    },
    SameResourceError,
}

impl From<TwoResourcePoolError> for RuntimeError {
    fn from(error: TwoResourcePoolError) -> Self {
        Self::ApplicationError(ApplicationError::TwoResourcePoolError(error))
    }
}
