use crate::manifest::ManifestValidationError;
use radix_common::prelude::*;

#[derive(Clone, Debug)]
pub enum StaticResourceMovementsError {
    DecimalOverflow,
    NonFungibleIdsTakeOnFungibleResource,
    NonFungibleIdsAssertionOnFungibleResource,
    AccountWithdrawNonFungiblesOnAFungibleResource,
    AccountLockerWithdrawNonFungiblesOnAFungibleResource,
    BucketDoesntExist(ManifestBucket),
    ManifestValidationError(ManifestValidationError),
}

impl<'a> From<ManifestValidationError> for StaticResourceMovementsError {
    fn from(value: ManifestValidationError) -> Self {
        Self::ManifestValidationError(value)
    }
}
