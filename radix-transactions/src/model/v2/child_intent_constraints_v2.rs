use crate::internal_prelude::*;

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor, ScryptoDescribe)]
#[sbor(transparent)]
pub struct ChildIntentConstraintsV2(Vec<ChildIntentConstraintV2>);

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PreparedChildIntentConstraintsV2 {
    constraints: Vec<PreparedChildIntentConstraintV2>,
    summary: Summary,
}

impl HasSummary for PreparedChildIntentConstraintsV2 {
    fn get_summary(&self) -> &Summary {
        &self.summary
    }
}

impl TransactionPreparableFromValue for PreparedChildIntentConstraintsV2 {
    fn prepare_from_value(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError> {
        let (constraints, summary) = ConcatenatedDigest::prepare_from_sbor_array::<
            Vec<PreparedChildIntentConstraintV2>,
            V2_MAX_NUMBER_OF_CHILD_SUBINTENTS_IN_INTENT,
        >(decoder, ValueType::ChildIntentConstraint)?;

        Ok(Self {
            constraints,
            summary,
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor, ScryptoDescribe)]
pub enum ChildIntentConstraintV2 {
    Any,
    Fixed(SubintentHash),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PreparedChildIntentConstraintV2 {
    constraint: ChildIntentConstraintV2,
    summary: Summary,
}

impl HasSummary for PreparedChildIntentConstraintV2 {
    fn get_summary(&self) -> &Summary {
        &self.summary
    }
}

impl TransactionPreparableFromValueBody for PreparedChildIntentConstraintV2 {
    fn prepare_from_value_body(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError> {
        let before_offset = decoder.get_offset();
        let constraint = decoder.decode_deeper_body_with_value_kind(ValueKind::Enum)?;
        let after_offset = decoder.get_offset();
        let summary = Summary {
            effective_length: after_offset - before_offset,
            total_bytes_hashed: 0,
            hash: match &constraint {
                ChildIntentConstraintV2::Any => Hash([0; Hash::LENGTH]),
                ChildIntentConstraintV2::Fixed(intent_hash) => *intent_hash.as_hash(),
            },
        };
        Ok(Self {
            constraint,
            summary,
        })
    }

    fn value_kind() -> ManifestValueKind {
        ManifestValueKind::Enum
    }
}
