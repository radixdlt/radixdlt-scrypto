use crate::internal_prelude::*;

// This file is for concepts which are version-independent

define_raw_transaction_payload!(RawIntent);
define_wrapped_hash!(
    /// A hash of the intent, used as the transaction id.
    /// The engine guarantees each intent hash can only be committed once.
    IntentHash
);

pub trait HasIntentHash {
    fn intent_hash(&self) -> IntentHash;
}

define_raw_transaction_payload!(RawSignedIntent);
define_wrapped_hash!(SignedIntentHash);

pub trait HasSignedIntentHash {
    fn signed_intent_hash(&self) -> SignedIntentHash;
}

define_raw_transaction_payload!(RawNotarizedTransaction);
define_wrapped_hash!(NotarizedTransactionHash);

pub trait HasNotarizedTransactionHash {
    fn notarized_transaction_hash(&self) -> NotarizedTransactionHash;
}

define_raw_transaction_payload!(RawSystemTransaction);
define_wrapped_hash!(SystemTransactionHash);

pub trait HasSystemTransactionHash {
    fn system_transaction_hash(&self) -> SystemTransactionHash;
}
