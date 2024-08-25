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
    pub root_intent: PartialTransactionV2,
    pub root_intent_signatures: IntentSignaturesV2,
    pub subintent_signatures: MultipleIntentSignaturesV2,
}

impl TransactionPayload for SignedPartialTransactionV2 {
    type Prepared = PreparedSignedPartialTransactionV2;
    type Raw = RawSignedPartialTransaction;
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PreparedSignedPartialTransactionV2 {
    pub root_intent: PreparedPartialTransactionV2,
    pub root_intent_signatures: PreparedIntentSignaturesV2,
    pub subintent_signatures: PreparedMultipleIntentSignaturesV2,
    pub summary: Summary,
}

impl HasSummary for PreparedSignedPartialTransactionV2 {
    fn get_summary(&self) -> &Summary {
        &self.summary
    }
}

impl TransactionPreparableFromValue for PreparedSignedPartialTransactionV2 {
    fn prepare_from_value(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError> {
        // When embedded as an child, it's SBOR encoded as a struct
        let ((root_intent, root_intent_signatures, subintent_signatures), summary) =
            ConcatenatedDigest::prepare_from_transaction_child_struct(
                decoder,
                TransactionDiscriminator::V2SignedPartialTransaction,
            )?;
        Ok(Self {
            root_intent,
            root_intent_signatures,
            subintent_signatures,
            summary,
        })
    }
}

impl TransactionPayloadPreparable for PreparedSignedPartialTransactionV2 {
    type Raw = RawSignedPartialTransaction;

    fn prepare_for_payload(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError> {
        // When embedded as full payload, it's SBOR encoded as an enum
        let ((root_intent, root_intent_signatures, subintent_signatures), summary) =
            ConcatenatedDigest::prepare_from_transaction_payload_enum(
                decoder,
                TransactionDiscriminator::V2SignedPartialTransaction,
            )?;
        Ok(Self {
            root_intent,
            root_intent_signatures,
            subintent_signatures,
            summary,
        })
    }
}

impl HasSubintentHash for PreparedSignedPartialTransactionV2 {
    fn subintent_hash(&self) -> SubintentHash {
        self.root_intent.subintent_hash()
    }
}
