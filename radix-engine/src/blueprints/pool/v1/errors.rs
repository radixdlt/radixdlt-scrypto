use crate::errors::*;
use crate::internal_prelude::*;

pub mod one_resource_pool {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
    pub enum Error {
        NonFungibleResourcesAreNotAccepted {
            resource_address: ResourceAddress,
        },
        NonZeroPoolUnitSupplyButZeroReserves,
        InvalidPoolUnitResource {
            expected: ResourceAddress,
            actual: ResourceAddress,
        },
        ContributionOfEmptyBucketError,
        DecimalOverflowError,
        InvalidGetRedemptionAmount,
        ZeroPoolUnitsMinted,
        RedeemedZeroTokens,
        ResourceDoesNotBelongToPool {
            resource_address: ResourceAddress,
        },
    }

    impl From<Error> for RuntimeError {
        fn from(error: Error) -> Self {
            Self::ApplicationError(ApplicationError::OneResourcePoolError(error))
        }
    }
}

pub mod two_resource_pool {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
    pub enum Error {
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
        DecimalOverflowError,
        InvalidGetRedemptionAmount,
        ZeroPoolUnitsMinted,
        LargerContributionRequiredToMeetRatio,
    }

    impl From<Error> for RuntimeError {
        fn from(error: Error) -> Self {
            Self::ApplicationError(ApplicationError::TwoResourcePoolError(error))
        }
    }
}

pub mod multi_resource_pool {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
    pub enum Error {
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
            resource_addresses: IndexSet<ResourceAddress>,
        },
        PoolCreationWithSameResource,
        ContributionOfEmptyBucketError,
        CantCreatePoolWithLessThanOneResource,
        DecimalOverflowError,
        InvalidGetRedemptionAmount,
        NoMinimumRatio,
        ZeroPoolUnitsMinted,
        LargerContributionRequiredToMeetRatio,
    }

    impl From<Error> for RuntimeError {
        fn from(error: Error) -> Self {
            Self::ApplicationError(ApplicationError::MultiResourcePoolError(error))
        }
    }
}
