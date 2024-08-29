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

impl TransactionPayload for SignedIntentV1 {
    type Prepared = PreparedSignedIntentV1;
    type Raw = RawSignedTransactionIntent;
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PreparedSignedIntentV1 {
    pub intent: PreparedIntentV1,
    pub intent_signatures: PreparedIntentSignaturesV1,
    pub summary: Summary,
}

impl_has_summary!(PreparedSignedIntentV1);

impl TransactionPreparableFromValue for PreparedSignedIntentV1 {
    fn prepare_from_value(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError> {
        // When embedded as an child, it's SBOR encoded as a struct
        let ((intent, intent_signatures), summary) =
            ConcatenatedDigest::prepare_from_transaction_child_struct(
                decoder,
                TransactionDiscriminator::V1SignedIntent,
            )?;
        Ok(Self {
            intent,
            intent_signatures,
            summary,
        })
    }
}

impl TransactionPayloadPreparable for PreparedSignedIntentV1 {
    type Raw = RawSignedTransactionIntent;

    fn prepare_for_payload(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError> {
        // When embedded as full payload, it's SBOR encoded as an enum
        let ((intent, intent_signatures), summary) =
            ConcatenatedDigest::prepare_from_transaction_payload_enum(
                decoder,
                TransactionDiscriminator::V1SignedIntent,
            )?;
        Ok(Self {
            intent,
            intent_signatures,
            summary,
        })
    }
}

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
