use crate::manifest::ManifestValidationError;
use radix_common::prelude::*;

#[derive(Clone, Debug)]
pub enum StaticResourceMovementsError<'a> {
    DecimalOverflow,
    NonFungibleIdsTakeOnFungibleResource,
    NonFungibleIdsAssertionOnFungibleResource,
    AccountWithdrawNonFungiblesOnAFungibleResource,
    AccountLockerWithdrawNonFungiblesOnAFungibleResource,
    BucketDoesntExist(ManifestBucket),
    ManifestValidationError(ManifestValidationError<'a>),
}

impl<'a> From<ManifestValidationError<'a>> for StaticResourceMovementsError<'a> {
    fn from(value: ManifestValidationError<'a>) -> Self {
        Self::ManifestValidationError(value)
    }
}
