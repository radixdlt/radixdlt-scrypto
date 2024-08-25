use crate::internal_prelude::*;

// This file is for concepts which are version-independent

define_raw_transaction_payload!(RawTransactionIntent);
define_wrapped_hash!(
    /// A hash of the primary intent of a transaction, used as the transaction id.
    /// The engine guarantees each intent hash can only be committed once.
    TransactionIntentHash
);

pub trait HasTransactionIntentHash {
    fn transaction_intent_hash(&self) -> TransactionIntentHash;
}

define_raw_transaction_payload!(RawSignedTransactionIntent);
define_wrapped_hash!(SignedTransactionIntentHash);

pub trait HasSignedTransactionIntentHash {
    fn signed_intent_hash(&self) -> SignedTransactionIntentHash;
}

define_raw_transaction_payload!(RawNotarizedTransaction);
define_wrapped_hash!(NotarizedTransactionHash);

pub trait HasNotarizedTransactionHash {
    fn notarized_transaction_hash(&self) -> NotarizedTransactionHash;
}

define_raw_transaction_payload!(RawSubintent);
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
define_raw_transaction_payload!(RawPartialTransaction);
define_raw_transaction_payload!(RawSignedPartialTransaction);

/// Note - Because transaction hashes do _not_ have a reserved first byte,
/// we can't encode them to bech32m unless we know their type.
///
/// Therefore this type has to be an enum to ensure we maintain the knowledge
/// of the underlying intent type, to allow the intent hash to be bech32m encoded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Sbor)]
pub enum IntentHash {
    Transaction(TransactionIntentHash),
    Sub(SubintentHash),
}

impl IntentHash {
    pub fn as_hash(&self) -> &Hash {
        match self {
            IntentHash::Transaction(hash) => hash.as_hash(),
            IntentHash::Sub(hash) => hash.as_hash(),
        }
    }
}

define_raw_transaction_payload!(RawSystemTransaction);
define_wrapped_hash!(SystemTransactionHash);

pub trait HasSystemTransactionHash {
    fn system_transaction_hash(&self) -> SystemTransactionHash;
}

define_raw_transaction_payload!(RawFlashTransaction);
define_wrapped_hash!(FlashTransactionHash);

pub trait HasFlashTransactionHash {
    fn flash_transaction_hash(&self) -> FlashTransactionHash;
}
