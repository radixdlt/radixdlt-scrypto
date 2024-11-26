use crate::internal_prelude::*;
use sbor::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeaderValidationError {
    InvalidEpochRange,
    InvalidTimestampRange,
    InvalidNetwork,
    InvalidTip,
    NoValidEpochRangeAcrossAllIntents,
    NoValidTimestampRangeAcrossAllIntents,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignatureValidationError {
    TooManySignatures { total: usize, limit: usize },
    InvalidIntentSignature,
    InvalidNotarySignature,
    DuplicateSigner,
    NotaryIsSignatorySoShouldNotAlsoBeASigner,
    SerializationError(EncodeError),
    IncorrectNumberOfSubintentSignatureBatches,
}

impl SignatureValidationError {
    pub fn located(
        self,
        location: TransactionValidationErrorLocation,
    ) -> TransactionValidationError {
        TransactionValidationError::SignatureValidationError(location, self)
    }
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
    SubintentStructureError(TransactionValidationErrorLocation, SubintentStructureError),
    IntentValidationError(TransactionValidationErrorLocation, IntentValidationError),
    SignatureValidationError(TransactionValidationErrorLocation, SignatureValidationError),
}

pub enum IntentSpecifier {
    RootTransactionIntent(TransactionIntentHash),
    RootSubintent(SubintentHash),
    NonRootSubintent(SubintentIndex, SubintentHash),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionValidationErrorLocation {
    RootTransactionIntent(TransactionIntentHash),
    RootSubintent(SubintentHash),
    NonRootSubintent(SubintentIndex, SubintentHash),
    AcrossTransaction,
    Unlocatable,
}

impl TransactionValidationErrorLocation {
    pub fn for_root(intent_hash: IntentHash) -> Self {
        match intent_hash {
            IntentHash::Transaction(hash) => Self::RootTransactionIntent(hash),
            IntentHash::Subintent(hash) => Self::RootSubintent(hash),
        }
    }
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntentValidationError {
    ManifestBasicValidatorError(ManifestBasicValidatorError),
    ManifestValidationError(ManifestValidationError),
    InvalidMessage(InvalidMessageError),
    HeaderValidationError(HeaderValidationError),
    TooManyReferences { total: usize, limit: usize },
}

impl From<HeaderValidationError> for IntentValidationError {
    fn from(value: HeaderValidationError) -> Self {
        Self::HeaderValidationError(value)
    }
}

impl From<InvalidMessageError> for IntentValidationError {
    fn from(value: InvalidMessageError) -> Self {
        Self::InvalidMessage(value)
    }
}

impl From<ManifestBasicValidatorError> for IntentValidationError {
    fn from(value: ManifestBasicValidatorError) -> Self {
        Self::ManifestBasicValidatorError(value)
    }
}

impl From<ManifestValidationError> for IntentValidationError {
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
pub enum SubintentStructureError {
    DuplicateSubintent,
    SubintentHasMultipleParents,
    ChildSubintentNotIncludedInTransaction(SubintentHash),
    SubintentExceedsMaxDepth,
    SubintentIsNotReachableFromTheTransactionIntent,
    MismatchingYieldChildAndYieldParentCountsForSubintent,
}

impl SubintentStructureError {
    pub fn for_unindexed(self) -> TransactionValidationError {
        TransactionValidationError::SubintentStructureError(
            TransactionValidationErrorLocation::Unlocatable,
            self,
        )
    }

    pub fn for_subintent(
        self,
        index: SubintentIndex,
        hash: SubintentHash,
    ) -> TransactionValidationError {
        TransactionValidationError::SubintentStructureError(
            TransactionValidationErrorLocation::NonRootSubintent(index, hash),
            self,
        )
    }
}
