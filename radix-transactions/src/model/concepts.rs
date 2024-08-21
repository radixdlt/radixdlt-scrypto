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

pub trait HasSubIntentHash {
    fn subintent_hash(&self) -> SubintentHash;
}

define_raw_transaction_payload!(RawSignedSubintent);
define_wrapped_hash!(SignedSubintentHash);

pub trait HasSignedSubintentHash {
    fn signed_subintent_hash(&self) -> SignedSubintentHash;
}

pub enum IntentHash {
    Transaction(TransactionIntentHash),
    Sub(SubintentHash),
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
