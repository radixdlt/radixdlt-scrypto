use crate::blueprints::resource::*;
use crate::errors::*;
use radix_engine_interface::prelude::*;

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
        FungibleResourceManagerError(FungibleResourceManagerError),
        NonFungibleResourceManagerError(NonFungibleResourceManagerError),
        BucketError(BucketError),
        VaultError(VaultError),
    }

    impl From<Error> for RuntimeError {
        fn from(error: Error) -> Self {
            Self::ApplicationError(ApplicationError::OneResourcePoolError(error))
        }
    }

    impl From<BucketError> for Error {
        fn from(error: BucketError) -> Self {
            Self::BucketError(error)
        }
    }

    impl From<VaultError> for Error {
        fn from(error: VaultError) -> Self {
            Self::VaultError(error)
        }
    }

    pub fn remap_application_error(runtime_error: RuntimeError) -> RuntimeError {
        match runtime_error {
            RuntimeError::ApplicationError(ApplicationError::BucketError(error)) => {
                RuntimeError::ApplicationError(ApplicationError::OneResourcePoolError(
                    Error::BucketError(error),
                ))
            }
            RuntimeError::ApplicationError(ApplicationError::VaultError(error)) => {
                RuntimeError::ApplicationError(ApplicationError::OneResourcePoolError(
                    Error::VaultError(error),
                ))
            }
            RuntimeError::ApplicationError(ApplicationError::FungibleResourceManagerError(
                error,
            )) => RuntimeError::ApplicationError(ApplicationError::OneResourcePoolError(
                Error::FungibleResourceManagerError(error),
            )),
            RuntimeError::ApplicationError(ApplicationError::NonFungibleResourceManagerError(
                error,
            )) => RuntimeError::ApplicationError(ApplicationError::OneResourcePoolError(
                Error::NonFungibleResourceManagerError(error),
            )),
            _ => runtime_error,
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
        FungibleResourceManagerError(FungibleResourceManagerError),
        NonFungibleResourceManagerError(NonFungibleResourceManagerError),
        BucketError(BucketError),
        VaultError(VaultError),
    }

    impl From<Error> for RuntimeError {
        fn from(error: Error) -> Self {
            Self::ApplicationError(ApplicationError::TwoResourcePoolError(error))
        }
    }

    pub fn remap_application_error(runtime_error: RuntimeError) -> RuntimeError {
        match runtime_error {
            RuntimeError::ApplicationError(ApplicationError::BucketError(error)) => {
                RuntimeError::ApplicationError(ApplicationError::TwoResourcePoolError(
                    Error::BucketError(error),
                ))
            }
            RuntimeError::ApplicationError(ApplicationError::VaultError(error)) => {
                RuntimeError::ApplicationError(ApplicationError::TwoResourcePoolError(
                    Error::VaultError(error),
                ))
            }
            RuntimeError::ApplicationError(ApplicationError::FungibleResourceManagerError(
                error,
            )) => RuntimeError::ApplicationError(ApplicationError::TwoResourcePoolError(
                Error::FungibleResourceManagerError(error),
            )),
            RuntimeError::ApplicationError(ApplicationError::NonFungibleResourceManagerError(
                error,
            )) => RuntimeError::ApplicationError(ApplicationError::TwoResourcePoolError(
                Error::NonFungibleResourceManagerError(error),
            )),
            _ => runtime_error,
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
        FungibleResourceManagerError(FungibleResourceManagerError),
        NonFungibleResourceManagerError(NonFungibleResourceManagerError),
        BucketError(BucketError),
        VaultError(VaultError),
    }

    impl From<Error> for RuntimeError {
        fn from(error: Error) -> Self {
            Self::ApplicationError(ApplicationError::MultiResourcePoolError(error))
        }
    }

    impl From<BucketError> for Error {
        fn from(error: BucketError) -> Self {
            Self::BucketError(error)
        }
    }

    impl From<VaultError> for Error {
        fn from(error: VaultError) -> Self {
            Self::VaultError(error)
        }
    }

    pub fn remap_application_error(runtime_error: RuntimeError) -> RuntimeError {
        match runtime_error {
            RuntimeError::ApplicationError(ApplicationError::BucketError(error)) => {
                RuntimeError::ApplicationError(ApplicationError::MultiResourcePoolError(
                    Error::BucketError(error),
                ))
            }
            RuntimeError::ApplicationError(ApplicationError::VaultError(error)) => {
                RuntimeError::ApplicationError(ApplicationError::MultiResourcePoolError(
                    Error::VaultError(error),
                ))
            }
            RuntimeError::ApplicationError(ApplicationError::FungibleResourceManagerError(
                error,
            )) => RuntimeError::ApplicationError(ApplicationError::MultiResourcePoolError(
                Error::FungibleResourceManagerError(error),
            )),
            RuntimeError::ApplicationError(ApplicationError::NonFungibleResourceManagerError(
                error,
            )) => RuntimeError::ApplicationError(ApplicationError::MultiResourcePoolError(
                Error::NonFungibleResourceManagerError(error),
            )),
            _ => runtime_error,
        }
    }
}
