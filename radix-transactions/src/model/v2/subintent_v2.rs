use super::*;
use crate::internal_prelude::*;

//=================================================================================
// NOTE:
// See versioned.rs for tests and a demonstration for the calculation of hashes etc
//=================================================================================

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor, ScryptoDescribe)]
pub struct SubintentV2 {
    pub intent_core: IntentCoreV2,
}

impl TransactionPayload for SubintentV2 {
    type Prepared = PreparedSubintentV2;
    type Raw = RawSubintent;
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PreparedSubintentV2 {
    pub intent_core: PreparedIntentCoreV2,
    pub summary: Summary,
}

impl HasSummary for PreparedSubintentV2 {
    fn get_summary(&self) -> &Summary {
        &self.summary
    }
}

impl TransactionPreparableFromValue for PreparedSubintentV2 {
    fn prepare_from_value(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError> {
        // When embedded as an child, it's SBOR encoded as a struct
        let ((intent_core,), summary) = ConcatenatedDigest::prepare_from_transaction_child_struct(
            decoder,
            TransactionDiscriminator::V2Subintent,
        )?;
        Ok(Self {
            intent_core,
            summary,
        })
    }
}

impl TransactionPayloadPreparable for PreparedSubintentV2 {
    type Raw = RawSubintent;

    fn prepare_for_payload(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError> {
        // When embedded as full payload, it's SBOR encoded as an enum
        let ((intent_core,), summary) = ConcatenatedDigest::prepare_from_transaction_payload_enum(
            decoder,
            TransactionDiscriminator::V2Subintent,
        )?;
        Ok(Self {
            intent_core,
            summary,
        })
    }
}

impl TransactionPreparableFromValueBody for PreparedSubintentV2 {
    fn prepare_from_value_body(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError> {
        // When embedded as an child body, it's SBOR encoded as a struct body (without value kind)
        let ((intent_core,), summary) =
            ConcatenatedDigest::prepare_from_transaction_child_struct_body(
                decoder,
                TransactionDiscriminator::V2Subintent,
            )?;
        Ok(Self {
            intent_core,
            summary,
        })
    }

    fn value_kind() -> ManifestValueKind {
        ManifestValueKind::Tuple
    }
}

impl HasSubintentHash for PreparedSubintentV2 {
    fn subintent_hash(&self) -> SubintentHash {
        SubintentHash::from_hash(self.summary.hash)
    }
}
