use super::*;
use crate::internal_prelude::*;

//=================================================================================
// NOTE:
// See versioned.rs for tests and a demonstration for the calculation of hashes etc
//=================================================================================

/// The main fields of an intent, used in both a Subintent and a TransactionIntent.
///
/// The instructions are put last so that it can be sensibly streamed into a manifest.
#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor, ScryptoDescribe)]
pub struct IntentCoreV2 {
    pub header: IntentHeaderV2,
    pub blobs: BlobsV1,
    pub message: MessageV2,
    pub children: ChildSubintentSpecifiersV2,
    pub instructions: InstructionsV2,
}

impl TransactionPartialPrepare for IntentCoreV2 {
    type Prepared = PreparedIntentCoreV2;
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PreparedIntentCoreV2 {
    pub header: PreparedIntentHeaderV2,
    pub blobs: PreparedBlobsV1,
    pub message: PreparedMessageV2,
    pub children: PreparedChildSubintentSpecifiersV2,
    pub instructions: PreparedInstructionsV2,
    pub summary: Summary,
}

impl_has_summary!(PreparedIntentCoreV2);

impl TransactionPreparableFromValueBody for PreparedIntentCoreV2 {
    fn prepare_from_value_body(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError> {
        if !decoder.settings().v2_transactions_permitted {
            return Err(PrepareError::TransactionTypeNotSupported);
        }
        // When embedded as an child, it's SBOR encoded as a struct
        let ((header, blobs, message, children, instructions), summary) =
            ConcatenatedDigest::prepare_from_sbor_tuple_value_body(decoder)?;
        Ok(Self {
            header,
            instructions,
            blobs,
            message,
            children,
            summary,
        })
    }

    fn value_kind() -> ManifestValueKind {
        ManifestValueKind::Tuple
    }
}
