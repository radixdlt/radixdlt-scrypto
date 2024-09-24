use super::*;
use crate::internal_prelude::*;

//=================================================================================
// NOTE:
// See versioned.rs for tests and a demonstration for the calculation of hashes etc
//=================================================================================

/// This isn't actually included in any submitted transaction, but it's useful to
/// have a canonical encoding of a full subintent tree with its associated signatures,
/// to enable passing around transaction parts.
#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor, ScryptoDescribe)]
pub struct SignedPartialTransactionV2 {
    pub partial_transaction: PartialTransactionV2,
    pub root_intent_signatures: IntentSignaturesV2,
    pub subintent_signatures: MultipleIntentSignaturesV2,
}

impl From<SignedPartialTransactionV2> for (SignedPartialTransactionV2, TransactionObjectNames) {
    fn from(value: SignedPartialTransactionV2) -> Self {
        let object_names = TransactionObjectNames::unknown_with_subintent_count(
            value.subintent_signatures.by_subintent.len(),
        );
        (value, object_names)
    }
}

define_transaction_payload!(
    SignedPartialTransactionV2,
    RawSignedPartialTransaction,
    PreparedSignedPartialTransactionV2 {
        root_intent: PreparedPartialTransactionV2,
        root_intent_signatures: PreparedIntentSignaturesV2,
        subintent_signatures: PreparedMultipleIntentSignaturesV2,
    },
    TransactionDiscriminator::V2SignedPartialTransaction,
);

impl HasSubintentHash for PreparedSignedPartialTransactionV2 {
    fn subintent_hash(&self) -> SubintentHash {
        self.root_intent.subintent_hash()
    }
}
