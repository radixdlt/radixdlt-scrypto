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
    AddressNotFound(u32),
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
    InvalidMessage(InvalidMessageError),
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
