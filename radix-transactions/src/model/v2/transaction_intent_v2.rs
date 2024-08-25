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

impl TransactionPayload for TransactionIntentV2 {
    type Prepared = PreparedTransactionIntentV2;
    type Raw = RawTransactionIntent;
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PreparedTransactionIntentV2 {
    pub root_header: PreparedTransactionHeaderV2,
    pub root_intent_core: PreparedIntentCoreV2,
    pub subintents: PreparedSubintentsV2,
    pub summary: Summary,
}

impl HasSummary for PreparedTransactionIntentV2 {
    fn get_summary(&self) -> &Summary {
        &self.summary
    }
}

impl TransactionPreparableFromValue for PreparedTransactionIntentV2 {
    fn prepare_from_value(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError> {
        // When embedded as an child, it's SBOR encoded as a struct
        let ((root_header, root_intent_core, subintents), summary) =
            ConcatenatedDigest::prepare_from_transaction_child_struct(
                decoder,
                TransactionDiscriminator::V2TransactionIntent,
            )?;
        Ok(Self {
            root_header,
            root_intent_core,
            subintents,
            summary,
        })
    }
}

impl TransactionPayloadPreparable for PreparedTransactionIntentV2 {
    type Raw = RawTransactionIntent;

    fn prepare_for_payload(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError> {
        // When embedded as full payload, it's SBOR encoded as an enum
        let ((root_header, root_intent_core, subintents), summary) =
            ConcatenatedDigest::prepare_from_transaction_payload_enum(
                decoder,
                TransactionDiscriminator::V2TransactionIntent,
            )?;
        Ok(Self {
            root_header,
            root_intent_core,
            subintents,
            summary,
        })
    }
}

impl HasTransactionIntentHash for PreparedTransactionIntentV2 {
    fn transaction_intent_hash(&self) -> TransactionIntentHash {
        TransactionIntentHash::from_hash(self.summary.hash)
    }
}
