use crate::internal_prelude::*;

// This file is for concepts which are version-independent

define_raw_transaction_payload!(RawTransactionIntent, TransactionPayloadKind::Other);
define_wrapped_hash!(
    /// A hash of the primary intent of a transaction, used as the transaction id.
    /// The engine guarantees each intent hash can only be committed once.
    TransactionIntentHash
);

pub trait HasTransactionIntentHash {
    fn transaction_intent_hash(&self) -> TransactionIntentHash;
}

define_raw_transaction_payload!(RawSignedTransactionIntent, TransactionPayloadKind::Other);
define_wrapped_hash!(SignedTransactionIntentHash);

pub trait HasSignedTransactionIntentHash {
    fn signed_transaction_intent_hash(&self) -> SignedTransactionIntentHash;
}

define_raw_transaction_payload!(
    RawNotarizedTransaction,
    TransactionPayloadKind::CompleteUserTransaction
);
define_wrapped_hash!(NotarizedTransactionHash);

impl RawNotarizedTransaction {
    pub fn into_typed(&self) -> Result<UserTransaction, DecodeError> {
        manifest_decode(self.as_slice())
    }

    pub fn prepare(
        &self,
        settings: &PreparationSettings,
    ) -> Result<PreparedUserTransaction, PrepareError> {
        PreparedUserTransaction::prepare(self, settings)
    }

    pub fn validate(
        &self,
        validator: &TransactionValidator,
    ) -> Result<ValidatedUserTransaction, TransactionValidationError> {
        self.prepare(validator.preparation_settings())?
            .validate(validator)
    }

    pub fn prepare_as_known_v2(
        &self,
        settings: &PreparationSettings,
    ) -> Result<PreparedNotarizedTransactionV2, PrepareError> {
        PreparedNotarizedTransactionV2::prepare(self, settings)
    }

    pub fn validate_as_known_v2(
        &self,
        validator: &TransactionValidator,
    ) -> Result<ValidatedNotarizedTransactionV2, TransactionValidationError> {
        self.prepare_as_known_v2(validator.preparation_settings())?
            .validate(validator)
    }
}

pub trait ResolveAsRawNotarizedTransaction {
    type Intermediate: AsRef<RawNotarizedTransaction>;

    fn resolve_raw_notarized_transaction(self) -> Self::Intermediate;
}

impl AsRef<RawNotarizedTransaction> for RawNotarizedTransaction {
    fn as_ref(&self) -> &RawNotarizedTransaction {
        self
    }
}

impl<T: AsRef<RawNotarizedTransaction>> ResolveAsRawNotarizedTransaction for T {
    type Intermediate = Self;

    fn resolve_raw_notarized_transaction(self) -> Self::Intermediate {
        self
    }
}

impl IntoExecutable for RawNotarizedTransaction {
    type Error = TransactionValidationError;

    fn into_executable(
        self,
        validator: &TransactionValidator,
    ) -> Result<ExecutableTransaction, Self::Error> {
        let executable = self.validate(validator)?.create_executable();
        Ok(executable)
    }
}

pub trait HasNotarizedTransactionHash {
    fn notarized_transaction_hash(&self) -> NotarizedTransactionHash;
}

pub trait HasNonRootSubintentHashes {
    /// ## Validity Note
    /// Preparable but invalid transactions may contain non-root subintents with duplicate [`SubintentHash`]es.
    /// Therefore we return a `Vec` instead of an `IndexSet` here.
    fn non_root_subintent_hashes(&self) -> Vec<SubintentHash>;
}

define_raw_transaction_payload!(RawSubintent, TransactionPayloadKind::Other);
define_wrapped_hash!(
    /// A hash of the subintent.
    /// The engine guarantees each intent hash can only be committed once.
    SubintentHash
);

pub trait HasSubintentHash {
    fn subintent_hash(&self) -> SubintentHash;
}

// There are no associated hashes for these things, because they don't need them.
// A solver can work out their own passing strategy
define_raw_transaction_payload!(RawPartialTransaction, TransactionPayloadKind::Other);
define_raw_transaction_payload!(RawSignedPartialTransaction, TransactionPayloadKind::Other);
define_raw_transaction_payload!(RawPreviewTransaction, TransactionPayloadKind::Other);

/// Note - Because transaction hashes do _not_ have a reserved first byte,
/// we can't encode them to bech32m unless we know their type.
///
/// Therefore this type has to be an enum to ensure we maintain the knowledge
/// of the underlying intent type, to allow the intent hash to be bech32m encoded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Sbor)]
pub enum IntentHash {
    Transaction(TransactionIntentHash),
    Subintent(SubintentHash),
}

impl From<TransactionIntentHash> for IntentHash {
    fn from(value: TransactionIntentHash) -> Self {
        Self::Transaction(value)
    }
}

impl From<SubintentHash> for IntentHash {
    fn from(value: SubintentHash) -> Self {
        Self::Subintent(value)
    }
}

impl IntentHash {
    pub fn is_for_subintent(&self) -> bool {
        match self {
            IntentHash::Transaction(_) => false,
            IntentHash::Subintent(_) => true,
        }
    }

    pub fn as_hash(&self) -> &Hash {
        match self {
            IntentHash::Transaction(hash) => hash.as_hash(),
            IntentHash::Subintent(hash) => hash.as_hash(),
        }
    }

    pub fn into_hash(self) -> Hash {
        match self {
            IntentHash::Transaction(hash) => hash.into_hash(),
            IntentHash::Subintent(hash) => hash.into_hash(),
        }
    }

    pub fn to_nullification(self, expiry_epoch: Epoch) -> IntentHashNullification {
        match self {
            IntentHash::Transaction(tx_intent_hash) => IntentHashNullification::TransactionIntent {
                intent_hash: tx_intent_hash,
                expiry_epoch,
            },
            IntentHash::Subintent(subintent_hash) => IntentHashNullification::Subintent {
                intent_hash: subintent_hash,
                expiry_epoch,
            },
        }
    }
}

define_raw_transaction_payload!(RawSystemTransaction, TransactionPayloadKind::Other);
define_wrapped_hash!(SystemTransactionHash);

pub trait HasSystemTransactionHash {
    fn system_transaction_hash(&self) -> SystemTransactionHash;
}

define_raw_transaction_payload!(RawFlashTransaction, TransactionPayloadKind::Other);
define_wrapped_hash!(FlashTransactionHash);

pub trait HasFlashTransactionHash {
    fn flash_transaction_hash(&self) -> FlashTransactionHash;
}
