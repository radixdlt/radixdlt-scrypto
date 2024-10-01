use super::*;
use crate::internal_prelude::*;

//=================================================================================
// NOTE:
// See versioned.rs for tests and a demonstration for the calculation of hashes etc
//=================================================================================

/// This should really be `TransactionIntentV1`, but keeping the old name to avoid refactoring in node.
#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor, ScryptoDescribe)]
pub struct IntentV1 {
    pub header: TransactionHeaderV1,
    pub instructions: InstructionsV1,
    pub blobs: BlobsV1,
    pub message: MessageV1,
}

define_transaction_payload!(
    IntentV1,
    RawTransactionIntent,
    PreparedIntentV1 {
        header: PreparedTransactionHeaderV1,
        instructions: PreparedInstructionsV1,
        blobs: PreparedBlobsV1,
        message: PreparedMessageV1,
    },
    TransactionDiscriminator::V1Intent,
);

impl HasTransactionIntentHash for PreparedIntentV1 {
    fn transaction_intent_hash(&self) -> TransactionIntentHash {
        TransactionIntentHash::from_hash(self.summary.hash)
    }
}
