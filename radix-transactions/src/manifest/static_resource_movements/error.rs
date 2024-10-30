use super::*;
use crate::manifest::ManifestValidationError;
use radix_common::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StaticResourceMovementsError {
    DecimalAmountIsNegative,
    BoundsInvalidForResourceKind,
    ConstraintBoundsInvalid,
    AssertionCannotBeSatisfied,
    TakeCannotBeSatisfied,
    DecimalOverflow,
    DuplicateNonFungibleId,
    WorktopEndsWithKnownResourcesPresent,
    ManifestValidationError(ManifestValidationError),
    NotAResourceAddress(GlobalAddress),
    TypedManifestNativeInvocationError(TypedManifestNativeInvocationError),
    AggregatedBalanceChangeWithdrawDoesNotSupportUnknownResources,
    UnexpectedBoundsForNetWithdraw,
}

impl From<ManifestValidationError> for StaticResourceMovementsError {
    fn from(value: ManifestValidationError) -> Self {
        Self::ManifestValidationError(value)
    }
}

impl From<BoundAdjustmentError> for StaticResourceMovementsError {
    fn from(value: BoundAdjustmentError) -> Self {
        match value {
            BoundAdjustmentError::DecimalOverflow => StaticResourceMovementsError::DecimalOverflow,
            BoundAdjustmentError::TakeCannotBeSatisfied => {
                StaticResourceMovementsError::TakeCannotBeSatisfied
            }
        }
    }
}

impl From<TypedManifestNativeInvocationError> for StaticResourceMovementsError {
    fn from(value: TypedManifestNativeInvocationError) -> Self {
        Self::TypedManifestNativeInvocationError(value)
    }
}
