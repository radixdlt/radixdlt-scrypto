use super::*;
use crate::internal_prelude::*;

//=================================================================================
// NOTE:
// See versioned.rs for tests and a demonstration for the calculation of hashes etc
//=================================================================================

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor, ScryptoDescribe)]
pub struct SignedTransactionIntentV2 {
    pub root_intent: TransactionIntentV2,
    pub root_intent_signatures: IntentSignaturesV1,
    pub subintent_signatures: MultipleIntentSignaturesV2,
}

transaction_payload_v2!(
    SignedTransactionIntentV2,
    RawSignedTransactionIntent,
    PreparedSignedTransactionIntentV2 {
        root_intent: PreparedTransactionIntentV2,
        root_intent_signatures: PreparedIntentSignaturesV1,
        subintent_signatures: PreparedMultipleIntentSignaturesV2,
    },
    TransactionDiscriminator::V2SignedTransactionIntent,
);

impl HasTransactionIntentHash for PreparedSignedTransactionIntentV2 {
    fn transaction_intent_hash(&self) -> TransactionIntentHash {
        self.root_intent.transaction_intent_hash()
    }
}

impl HasSignedTransactionIntentHash for PreparedSignedTransactionIntentV2 {
    fn signed_transaction_intent_hash(&self) -> SignedTransactionIntentHash {
        SignedTransactionIntentHash::from_hash(self.summary.hash)
    }
}
