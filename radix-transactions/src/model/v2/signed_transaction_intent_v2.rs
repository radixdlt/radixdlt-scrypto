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

impl TransactionPayload for SignedTransactionIntentV2 {
    type Prepared = PreparedSignedTransactionIntentV2;
    type Raw = RawSignedTransactionIntent;
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PreparedSignedTransactionIntentV2 {
    pub root_intent: PreparedTransactionIntentV2,
    pub root_intent_signatures: PreparedIntentSignaturesV1,
    pub subintent_signatures: PreparedMultipleIntentSignaturesV2,
    pub summary: Summary,
}

impl HasSummary for PreparedSignedTransactionIntentV2 {
    fn get_summary(&self) -> &Summary {
        &self.summary
    }
}

impl TransactionPreparableFromValue for PreparedSignedTransactionIntentV2 {
    fn prepare_from_value(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError> {
        // When embedded as an child, it's SBOR encoded as a struct
        let ((root_intent, root_intent_signatures, subintent_signatures), summary) =
            ConcatenatedDigest::prepare_from_transaction_child_struct(
                decoder,
                TransactionDiscriminator::V2SignedTransactionIntent,
            )?;
        Ok(Self {
            root_intent,
            root_intent_signatures,
            subintent_signatures,
            summary,
        })
    }
}

impl TransactionPayloadPreparable for PreparedSignedTransactionIntentV2 {
    type Raw = RawSignedTransactionIntent;

    fn prepare_for_payload(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError> {
        // When embedded as full payload, it's SBOR encoded as an enum
        let ((root_intent, root_intent_signatures, subintent_signatures), summary) =
            ConcatenatedDigest::prepare_from_transaction_payload_enum(
                decoder,
                TransactionDiscriminator::V2SignedTransactionIntent,
            )?;
        Ok(Self {
            root_intent,
            root_intent_signatures,
            subintent_signatures,
            summary,
        })
    }
}

impl HasTransactionIntentHash for PreparedSignedTransactionIntentV2 {
    fn transaction_intent_hash(&self) -> TransactionIntentHash {
        self.root_intent.transaction_intent_hash()
    }
}

impl HasSignedTransactionIntentHash for PreparedSignedTransactionIntentV2 {
    fn signed_intent_hash(&self) -> SignedTransactionIntentHash {
        SignedTransactionIntentHash::from_hash(self.summary.hash)
    }
}
