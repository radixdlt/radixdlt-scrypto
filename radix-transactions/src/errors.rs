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

impl<'a> ContextualDisplay<TransactionHashDisplayContext<'a>> for TransactionValidationError {
    type Error = fmt::Error;

    fn contextual_format(
        &self,
        f: &mut fmt::Formatter,
        context: &TransactionHashDisplayContext<'a>,
    ) -> Result<(), Self::Error> {
        match self {
            Self::TransactionVersionNotPermitted(arg0) => f
                .debug_tuple("TransactionVersionNotPermitted")
                .field(arg0)
                .finish(),
            Self::TransactionTooLarge => write!(f, "TransactionTooLarge"),
            Self::EncodeError(arg0) => f.debug_tuple("EncodeError").field(arg0).finish(),
            Self::PrepareError(arg0) => f.debug_tuple("PrepareError").field(arg0).finish(),
            Self::SubintentStructureError(arg0, arg1) => f
                .debug_tuple("SubintentStructureError")
                .field(&arg0.debug_as_display(*context))
                .field(&arg1.debug_as_display(*context))
                .finish(),
            Self::IntentValidationError(arg0, arg1) => f
                .debug_tuple("IntentValidationError")
                .field(&arg0.debug_as_display(*context))
                .field(arg1)
                .finish(),
            Self::SignatureValidationError(arg0, arg1) => f
                .debug_tuple("SignatureValidationError")
                .field(&arg0.debug_as_display(*context))
                .field(arg1)
                .finish(),
        }
    }
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

impl<'a> ContextualDisplay<TransactionHashDisplayContext<'a>>
    for TransactionValidationErrorLocation
{
    type Error = fmt::Error;

    fn contextual_format(
        &self,
        f: &mut fmt::Formatter,
        context: &TransactionHashDisplayContext<'a>,
    ) -> Result<(), Self::Error> {
        // Copied from the auto-generated `Debug` implementation, and tweaked
        match self {
            Self::RootTransactionIntent(arg0) => f
                .debug_tuple("RootTransactionIntent")
                .field(&arg0.debug_as_display(*context))
                .finish(),
            Self::RootSubintent(arg0) => f
                .debug_tuple("RootSubintent")
                .field(&arg0.debug_as_display(*context))
                .finish(),
            Self::NonRootSubintent(arg0, arg1) => f
                .debug_tuple("NonRootSubintent")
                .field(arg0)
                .field(&arg1.debug_as_display(*context))
                .finish(),
            Self::AcrossTransaction => write!(f, "AcrossTransaction"),
            Self::Unlocatable => write!(f, "Unlocatable"),
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

impl<'a> ContextualDisplay<TransactionHashDisplayContext<'a>> for SubintentStructureError {
    type Error = fmt::Error;

    fn contextual_format(
        &self,
        f: &mut fmt::Formatter,
        context: &TransactionHashDisplayContext<'a>,
    ) -> Result<(), Self::Error> {
        // Copied from the auto-generated `Debug` implementation, and tweaked
        match self {
            Self::DuplicateSubintent => write!(f, "DuplicateSubintent"),
            Self::SubintentHasMultipleParents => write!(f, "SubintentHasMultipleParents"),
            Self::ChildSubintentNotIncludedInTransaction(arg0) => f
                .debug_tuple("ChildSubintentNotIncludedInTransaction")
                .field(&arg0.debug_as_display(*context))
                .finish(),
            Self::SubintentExceedsMaxDepth => write!(f, "SubintentExceedsMaxDepth"),
            Self::SubintentIsNotReachableFromTheTransactionIntent => {
                write!(f, "SubintentIsNotReachableFromTheTransactionIntent")
            }
            Self::MismatchingYieldChildAndYieldParentCountsForSubintent => {
                write!(f, "MismatchingYieldChildAndYieldParentCountsForSubintent")
            }
        }
    }
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
