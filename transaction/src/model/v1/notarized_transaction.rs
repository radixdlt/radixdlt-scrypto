use super::*;
use crate::internal_prelude::*;

//=================================================================================
// NOTE:
// See versioned.rs for tests and a demonstration for the calculation of hashes etc
//=================================================================================

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor)]
pub struct NotarizedTransactionV1 {
    pub signed_intent: SignedIntentV1,
    pub notary_signature: NotarySignatureV1,
}

impl TransactionPayload for NotarizedTransactionV1 {
    type Versioned = SborFixedEnumVariant<{ TransactionDiscriminator::V1Notarized as u8 }, Self>;
    type Prepared = PreparedNotarizedTransactionV1;
    type Raw = RawNotarizedTransaction;
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PreparedNotarizedTransactionV1 {
    pub signed_intent: PreparedSignedIntentV1,
    pub notary_signature: PreparedNotarySignatureV1,
    pub summary: Summary,
}

impl HasSummary for PreparedNotarizedTransactionV1 {
    fn get_summary(&self) -> &Summary {
        &self.summary
    }
}

impl TransactionFullChildPreparable for PreparedNotarizedTransactionV1 {
    fn prepare_as_full_body_child(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError> {
        // When embedded as an child, it's SBOR encoded as a struct
        let ((signed_intent, notary_signature), summary) =
            ConcatenatedDigest::prepare_from_transaction_child_struct(
                decoder,
                TransactionDiscriminator::V1Notarized,
            )?;
        Ok(Self {
            signed_intent,
            notary_signature,
            summary,
        })
    }
}

impl TransactionPayloadPreparable for PreparedNotarizedTransactionV1 {
    type Raw = RawNotarizedTransaction;

    fn prepare_for_payload(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError> {
        // When embedded as full payload, it's SBOR encoded as an enum
        let ((signed_intent, notary_signature), summary) =
            ConcatenatedDigest::prepare_from_transaction_payload_enum(
                decoder,
                TransactionDiscriminator::V1Notarized,
            )?;
        Ok(Self {
            signed_intent,
            notary_signature,
            summary,
        })
    }
}

impl HasIntentHash for PreparedNotarizedTransactionV1 {
    fn intent_hash(&self) -> IntentHash {
        self.signed_intent.intent_hash()
    }
}

impl HasSignedIntentHash for PreparedNotarizedTransactionV1 {
    fn signed_intent_hash(&self) -> SignedIntentHash {
        self.signed_intent.signed_intent_hash()
    }
}

impl HasNotarizedTransactionHash for PreparedNotarizedTransactionV1 {
    fn notarized_transaction_hash(&self) -> NotarizedTransactionHash {
        NotarizedTransactionHash::from_hash(self.summary.hash)
    }
}
