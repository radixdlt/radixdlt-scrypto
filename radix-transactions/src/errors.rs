use crate::internal_prelude::*;
use sbor::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeaderValidationError {
    UnknownVersion(u8),
    InvalidEpochRange,
    InvalidTimestampRange,
    InvalidNetwork,
    InvalidCostUnitLimit,
    InvalidTip,
    NoValidEpochRangeAcrossAllIntents,
    NoValidTimestampRangeAcrossAllIntents,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignatureValidationError {
    InvalidIntentSignature,
    InvalidNotarySignature,
    DuplicateSigner,
    SerializationError(EncodeError),
    IncorrectNumberOfSubintentSignatureBatches,
}

impl From<EncodeError> for SignatureValidationError {
    fn from(err: EncodeError) -> Self {
        Self::SerializationError(err)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManifestIdValidationError {
    BucketNotFound(ManifestBucket),
    ProofNotFound(ManifestProof),
    BucketLocked(ManifestBucket),
    AddressReservationNotFound(ManifestAddressReservation),
    AddressNotFound(ManifestNamedAddress),
    IntentNotFound(ManifestNamedIntent),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManifestBasicValidatorError {
    ManifestIdValidationError(ManifestIdValidationError),
}

impl From<ManifestIdValidationError> for ManifestBasicValidatorError {
    fn from(value: ManifestIdValidationError) -> Self {
        Self::ManifestIdValidationError(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionValidationError {
    TransactionVersionNotPermitted(usize),
    TransactionTooLarge,
    EncodeError(EncodeError),
    PrepareError(PrepareError),
    HeaderValidationError(HeaderValidationError),
    SignatureValidationError(SignatureValidationError),
    ManifestBasicValidatorError(ManifestBasicValidatorError),
    ManifestValidationError(ManifestValidationError),
    InvalidMessage(InvalidMessageError),
    SubintentError(SubintentValidationError),
    Other(String),
    TooManySignatures {
        total: usize,
        limit: usize,
    },
    TooManySignaturesForIntent {
        index: usize,
        total: usize,
        limit: usize,
    },
    TooManyReferences {
        total: usize,
        limit: usize,
    },
    TooManyReferencesForIntent {
        index: usize,
        total: usize,
        limit: usize,
    },
}

impl From<PrepareError> for TransactionValidationError {
    fn from(value: PrepareError) -> Self {
        Self::PrepareError(value)
    }
}

impl From<EncodeError> for TransactionValidationError {
    fn from(value: EncodeError) -> Self {
        Self::EncodeError(value)
    }
}

impl From<InvalidMessageError> for TransactionValidationError {
    fn from(value: InvalidMessageError) -> Self {
        Self::InvalidMessage(value)
    }
}

impl From<SubintentValidationError> for TransactionValidationError {
    fn from(value: SubintentValidationError) -> Self {
        Self::SubintentError(value)
    }
}

impl From<SignatureValidationError> for TransactionValidationError {
    fn from(value: SignatureValidationError) -> Self {
        Self::SignatureValidationError(value)
    }
}

impl From<HeaderValidationError> for TransactionValidationError {
    fn from(value: HeaderValidationError) -> Self {
        Self::HeaderValidationError(value)
    }
}

impl From<ManifestBasicValidatorError> for TransactionValidationError {
    fn from(value: ManifestBasicValidatorError) -> Self {
        Self::ManifestBasicValidatorError(value)
    }
}

impl From<ManifestValidationError> for TransactionValidationError {
    fn from(value: ManifestValidationError) -> Self {
        Self::ManifestValidationError(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InvalidMessageError {
    PlaintextMessageTooLong {
        actual: usize,
        permitted: usize,
    },
    MimeTypeTooLong {
        actual: usize,
        permitted: usize,
    },
    EncryptedMessageTooLong {
        actual: usize,
        permitted: usize,
    },
    NoDecryptors,
    MismatchingDecryptorCurves {
        actual: CurveType,
        expected: CurveType,
    },
    TooManyDecryptors {
        actual: usize,
        permitted: usize,
    },
    NoDecryptorsForCurveType {
        curve_type: CurveType,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubintentValidationError {
    DuplicateSubintent(SubintentHash),
    SubintentHasMultipleParents(SubintentHash),
    ChildSubintentNotIncludedInTransaction(SubintentHash),
    SubintentExceedsMaxDepth(SubintentHash),
    SubintentIsNotReachableFromTheTransactionIntent(SubintentHash),
    MismatchingYieldChildAndYieldParentCountsForSubintent(SubintentHash),
}
