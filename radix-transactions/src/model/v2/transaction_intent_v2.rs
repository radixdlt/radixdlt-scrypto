use super::*;
use crate::internal_prelude::*;

//=================================================================================
// NOTE:
// See versioned.rs for tests and a demonstration for the calculation of hashes etc
//=================================================================================

/// This should really be `TransactionIntentV1`, but keeping the old name to avoid refactoring in node.
#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor, ScryptoDescribe)]
pub struct TransactionIntentV2 {
    pub root_header: TransactionHeaderV2,
    pub root_intent_core: IntentCoreV2,
    pub subintents: SubintentsV2,
}

transaction_payload_v2!(
    TransactionIntentV2,
    RawTransactionIntent,
    PreparedTransactionIntentV2 {
        root_header: PreparedTransactionHeaderV2,
        root_intent_core: PreparedIntentCoreV2,
        subintents: PreparedSubintentsV2,
    },
    TransactionDiscriminator::V2TransactionIntent,
);

impl HasTransactionIntentHash for PreparedTransactionIntentV2 {
    fn transaction_intent_hash(&self) -> TransactionIntentHash {
        TransactionIntentHash::from_hash(self.summary.hash)
    }
}
