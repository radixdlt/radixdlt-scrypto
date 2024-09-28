use crate::manifest::ManifestValidationError;
use radix_common::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StaticResourceMovementsError {
    DecimalAmountIsNegative,
    NonFungibleIdsSpecifiedAgainstFungibleResource,
    AssertionBoundsInvalid,
    AssertionCannotBeSatisfied,
    TakeCannotBeSatisfied,
    DecimalOverflow,
    DuplicateNonFungibleId,
    WorktopEndsWithKnownResourcesPresent,
    ManifestValidationError(ManifestValidationError),
}

impl From<ManifestValidationError> for StaticResourceMovementsError {
    fn from(value: ManifestValidationError) -> Self {
        Self::ManifestValidationError(value)
    }
}
