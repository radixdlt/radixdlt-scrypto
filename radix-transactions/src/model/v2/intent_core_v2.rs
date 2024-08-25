use super::*;
use crate::internal_prelude::*;

//=================================================================================
// NOTE:
// See versioned.rs for tests and a demonstration for the calculation of hashes etc
//=================================================================================

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor, ScryptoDescribe)]
pub struct IntentCoreV2 {
    pub core_header: IntentHeaderV2,
    pub instructions: InstructionsV2,
    pub blobs: BlobsV1,
    pub message: MessageV2, // Increase size of the key
    pub child_intent_constraints: ChildIntentConstraintsV2,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PreparedIntentCoreV2 {
    pub core_header: PreparedIntentHeaderV2,
    pub instructions: PreparedInstructionsV2,
    pub blobs: PreparedBlobsV1,
    pub message: PreparedMessageV2,
    pub child_intent_constraints: PreparedChildIntentConstraintsV2,
    pub summary: Summary,
}

impl HasSummary for PreparedIntentCoreV2 {
    fn get_summary(&self) -> &Summary {
        &self.summary
    }
}

impl TransactionPreparableFromValue for PreparedIntentCoreV2 {
    fn prepare_from_value(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError> {
        // When embedded as an child, it's SBOR encoded as a struct
        let ((core_header, instructions, blobs, message, child_intent_constraints), summary) =
            ConcatenatedDigest::prepare_from_sbor_tuple(decoder)?;
        Ok(Self {
            core_header,
            instructions,
            blobs,
            message,
            child_intent_constraints,
            summary,
        })
    }
}
