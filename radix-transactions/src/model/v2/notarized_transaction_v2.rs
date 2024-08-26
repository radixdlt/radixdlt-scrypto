use super::*;
use crate::internal_prelude::*;

//=================================================================================
// NOTE:
// See versioned.rs for tests and a demonstration for the calculation of hashes etc
//=================================================================================

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor, ScryptoDescribe, ScryptoSborAssertion)]
#[sbor_assert(
    fixed("FILE:notarized_transaction_v2_schema.txt"),
    settings(allow_name_changes)
)]
pub struct NotarizedTransactionV2 {
    pub signed_intent: SignedTransactionIntentV2,
    pub notary_signature: NotarySignatureV1,
}

impl TransactionPayload for NotarizedTransactionV2 {
    type Prepared = PreparedNotarizedTransactionV2;
    type Raw = RawNotarizedTransaction;
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PreparedNotarizedTransactionV2 {
    pub signed_intent: PreparedSignedTransactionIntentV2,
    pub notary_signature: PreparedNotarySignatureV1,
    pub summary: Summary,
}

impl HasSummary for PreparedNotarizedTransactionV2 {
    fn get_summary(&self) -> &Summary {
        &self.summary
    }
}

impl TransactionPreparableFromValue for PreparedNotarizedTransactionV2 {
    fn prepare_from_value(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError> {
        // When embedded as an child, it's SBOR encoded as a struct
        let ((signed_intent, notary_signature), summary) =
            ConcatenatedDigest::prepare_from_transaction_child_struct(
                decoder,
                TransactionDiscriminator::V2Notarized,
            )?;
        Ok(Self {
            signed_intent,
            notary_signature,
            summary,
        })
    }
}

impl TransactionPayloadPreparable for PreparedNotarizedTransactionV2 {
    type Raw = RawNotarizedTransaction;

    fn prepare_for_payload(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError> {
        // When embedded as full payload, it's SBOR encoded as an enum
        let ((signed_intent, notary_signature), summary) =
            ConcatenatedDigest::prepare_from_transaction_payload_enum(
                decoder,
                TransactionDiscriminator::V2Notarized,
            )?;
        Ok(Self {
            signed_intent,
            notary_signature,
            summary,
        })
    }
}

impl HasTransactionIntentHash for PreparedNotarizedTransactionV2 {
    fn transaction_intent_hash(&self) -> TransactionIntentHash {
        self.signed_intent.transaction_intent_hash()
    }
}

impl HasSignedTransactionIntentHash for PreparedNotarizedTransactionV2 {
    fn signed_intent_hash(&self) -> SignedTransactionIntentHash {
        self.signed_intent.signed_intent_hash()
    }
}

impl HasNotarizedTransactionHash for PreparedNotarizedTransactionV2 {
    fn notarized_transaction_hash(&self) -> NotarizedTransactionHash {
        NotarizedTransactionHash::from_hash(self.summary.hash)
    }
}
