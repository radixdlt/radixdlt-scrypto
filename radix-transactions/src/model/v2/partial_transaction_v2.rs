use super::*;
use crate::internal_prelude::*;

//=================================================================================
// NOTE:
// See versioned.rs for tests and a demonstration for the calculation of hashes etc
//=================================================================================

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor, ScryptoDescribe)]
pub struct PartialTransactionV2 {
    pub root_intent: SubintentV2,
    pub subintents: SubintentsV2,
}

impl TransactionPayload for PartialTransactionV2 {
    type Prepared = PreparedPartialTransactionV2;
    type Raw = RawPartialTransaction;
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PreparedPartialTransactionV2 {
    pub root_intent: PreparedSubintentV2,
    pub subintents: PreparedSubintentsV2,
    pub summary: Summary,
}

impl HasSummary for PreparedPartialTransactionV2 {
    fn get_summary(&self) -> &Summary {
        &self.summary
    }
}

impl TransactionPreparableFromValue for PreparedPartialTransactionV2 {
    fn prepare_from_value(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError> {
        // When embedded as an child, it's SBOR encoded as a struct
        let ((root_intent, subintents), summary) =
            ConcatenatedDigest::prepare_from_transaction_child_struct(
                decoder,
                TransactionDiscriminator::V2PartialTransaction,
            )?;
        Ok(Self {
            root_intent,
            subintents,
            summary,
        })
    }
}

impl TransactionPayloadPreparable for PreparedPartialTransactionV2 {
    type Raw = RawPartialTransaction;

    fn prepare_for_payload(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError> {
        // When embedded as full payload, it's SBOR encoded as an enum
        let ((root_intent, subintents), summary) =
            ConcatenatedDigest::prepare_from_transaction_payload_enum(
                decoder,
                TransactionDiscriminator::V2PartialTransaction,
            )?;
        Ok(Self {
            root_intent,
            subintents,
            summary,
        })
    }
}

impl HasSubintentHash for PreparedPartialTransactionV2 {
    fn subintent_hash(&self) -> SubintentHash {
        self.root_intent.subintent_hash()
    }
}
