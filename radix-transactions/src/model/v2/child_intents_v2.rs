use crate::internal_prelude::*;

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor, ScryptoDescribe)]
#[sbor(transparent)]
pub struct ChildIntentsV2 {
    pub children: Vec<SubintentHash>,
}

impl TransactionPartialPrepare for ChildIntentsV2 {
    type Prepared = PreparedChildIntentsV2;
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PreparedChildIntentsV2 {
    pub children: Vec<SubintentHash>,
    pub summary: Summary,
}

impl_has_summary!(PreparedChildIntentsV2);

impl TransactionPreparableFromValueBody for PreparedChildIntentsV2 {
    fn prepare_from_value_body(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError> {
        let (hashes, summary) = ConcatenatedDigest::prepare_from_sbor_array_value_body::<
            Vec<RawHash>,
            V2_MAX_NUMBER_OF_CHILD_SUBINTENTS_IN_INTENT,
        >(decoder, ValueType::ChildIntentConstraint)?;

        Ok(Self {
            children: hashes
                .into_iter()
                .map(|h| SubintentHash::from_hash(h.hash))
                .collect(),
            summary,
        })
    }

    fn value_kind() -> ManifestValueKind {
        ManifestValueKind::Array
    }
}
