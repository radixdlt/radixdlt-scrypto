use super::*;
use crate::internal_prelude::*;

//=================================================================================
// NOTE:
// See versioned.rs for tests and a demonstration for the calculation of hashes etc
//=================================================================================

/// This should really be `SignedTransactionIntentV1`, but keeping the old name to avoid refactoring in node.
#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor, ScryptoDescribe)]
pub struct SignedIntentV1 {
    pub intent: IntentV1,
    pub intent_signatures: IntentSignaturesV1,
}

define_transaction_payload!(
    SignedIntentV1,
    RawSignedTransactionIntent,
    PreparedSignedIntentV1 {
        intent: PreparedIntentV1,
        intent_signatures: PreparedIntentSignaturesV1,
    },
    TransactionDiscriminator::V1SignedIntent,
);

impl HasTransactionIntentHash for PreparedSignedIntentV1 {
    fn transaction_intent_hash(&self) -> TransactionIntentHash {
        self.intent.transaction_intent_hash()
    }
}

impl HasSignedTransactionIntentHash for PreparedSignedIntentV1 {
    fn signed_transaction_intent_hash(&self) -> SignedTransactionIntentHash {
        SignedTransactionIntentHash::from_hash(self.summary.hash)
    }
}
