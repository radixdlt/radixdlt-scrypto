use crate::manifest::ManifestValidationError;
use radix_common::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StaticResourceMovementsError {
    DecimalAmountIsNegative,
    NonFungibleIdsSpecifiedAgainstFungibleResource,
    ConstraintBoundsInvalid,
    AssertionCannotBeSatisfied,
    TakeCannotBeSatisfied,
    DecimalOverflow,
    DuplicateNonFungibleId,
    WorktopEndsWithKnownResourcesPresent,
    NativeArgumentsEncodeError(EncodeError),
    NativeArgumentsDecodeError(DecodeError),
    UnknownNativeBlueprint {
        package: PackageAddress,
        blueprint: String,
    },
    UnknownNativeMethod {
        package: PackageAddress,
        blueprint: String,
        method: String,
    },
    UnknownNativeFunction {
        package: PackageAddress,
        blueprint: String,
        function: String,
    },
    ManifestValidationError(ManifestValidationError),
}

impl From<ManifestValidationError> for StaticResourceMovementsError {
    fn from(value: ManifestValidationError) -> Self {
        Self::ManifestValidationError(value)
    }
}
