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
