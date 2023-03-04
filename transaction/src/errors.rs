use radix_engine_interface::data::manifest::model::*;
use sbor::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeaderValidationError {
    UnknownVersion(u8),
    InvalidEpochRange,
    EpochRangeTooLarge,
    InvalidNetwork,
    InvalidCostUnitLimit,
    InvalidTipBps,
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

#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
pub enum ManifestIdAllocationError {
    OutOfID,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManifestIdValidationError {
    IdAllocationError(ManifestIdAllocationError),
    BucketNotFound(ManifestBucket),
    ProofNotFound(ManifestProof),
    BucketLocked(ManifestBucket),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CallDataValidationError {
    DecodeError(DecodeError),
    IdValidationError(ManifestIdValidationError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionValidationError {
    TransactionTooLarge,
    SerializationError(EncodeError),
    DeserializationError(DecodeError),
    IntentHashRejected,
    HeaderValidationError(HeaderValidationError),
    SignatureValidationError(SignatureValidationError),
    IdValidationError(ManifestIdValidationError),
    CallDataValidationError(CallDataValidationError),
}

impl From<EncodeError> for TransactionValidationError {
    fn from(err: EncodeError) -> Self {
        Self::SerializationError(err)
    }
}
