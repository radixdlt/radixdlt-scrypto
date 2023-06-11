use crate::internal_prelude::*;
use sbor::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeaderValidationError {
    UnknownVersion(u8),
    InvalidEpochRange,
    EpochRangeTooLarge,
    InvalidNetwork,
    InvalidCostUnitLimit,
    InvalidTipPercentage,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignatureValidationError {
    TooManySignatures,
    InvalidIntentSignature,
    InvalidNotarySignature,
    DuplicateSigner,
    SerializationError(EncodeError),
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
    NamedAddressNotFound(u32),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CallDataValidationError {
    DecodeError(DecodeError),
    IdValidationError(ManifestIdValidationError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionValidationError {
    TransactionTooLarge,
    EncodeError(EncodeError),
    PrepareError(PrepareError),
    HeaderValidationError(HeaderValidationError),
    SignatureValidationError(SignatureValidationError),
    IdValidationError(ManifestIdValidationError),
    CallDataValidationError(CallDataValidationError),
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
