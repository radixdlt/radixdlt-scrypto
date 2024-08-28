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

impl TransactionPayload for IntentV1 {
    type Prepared = PreparedIntentV1;
    type Raw = RawTransactionIntent;
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PreparedIntentV1 {
    pub header: PreparedTransactionHeaderV1,
    pub instructions: PreparedInstructionsV1,
    pub blobs: PreparedBlobsV1,
    pub message: PreparedMessageV1,
    pub summary: Summary,
}

impl_has_summary!(PreparedIntentV1);

impl TransactionPreparableFromValue for PreparedIntentV1 {
    fn prepare_from_value(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError> {
        // When embedded as an child, it's SBOR encoded as a struct
        let ((header, instructions, blobs, attachments), summary) =
            ConcatenatedDigest::prepare_from_transaction_child_struct(
                decoder,
                TransactionDiscriminator::V1Intent,
            )?;
        Ok(Self {
            header,
            instructions,
            blobs,
            message: attachments,
            summary,
        })
    }
}

impl TransactionPayloadPreparable for PreparedIntentV1 {
    type Raw = RawTransactionIntent;

    fn prepare_for_payload(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError> {
        // When embedded as full payload, it's SBOR encoded as an enum
        let ((header, instructions, blobs, attachments), summary) =
            ConcatenatedDigest::prepare_from_transaction_payload_enum(
                decoder,
                TransactionDiscriminator::V1Intent,
            )?;
        Ok(Self {
            header,
            instructions,
            blobs,
            message: attachments,
            summary,
        })
    }
}

impl HasTransactionIntentHash for PreparedIntentV1 {
    fn transaction_intent_hash(&self) -> TransactionIntentHash {
        TransactionIntentHash::from_hash(self.summary.hash)
    }
}
