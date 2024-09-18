use super::*;
use crate::internal_prelude::*;

//=================================================================================
// NOTE:
// See versioned.rs for tests and a demonstration for the calculation of hashes etc
//=================================================================================

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor, ScryptoDescribe)]
#[sbor(transparent)]
pub struct SubintentsV2(pub Vec<SubintentV2>);

impl TransactionPartialPrepare for SubintentsV2 {
    type Prepared = PreparedSubintentsV2;
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PreparedSubintentsV2 {
    pub subintents: Vec<PreparedSubintentV2>,
    pub summary: Summary,
}

impl_has_summary!(PreparedSubintentsV2);

impl TransactionPreparableFromValueBody for PreparedSubintentsV2 {
    fn prepare_from_value_body(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError> {
        let max_subintents_per_transaction = decoder.settings().max_subintents_per_transaction;
        let (subintents, summary) =
            ConcatenatedDigest::prepare_from_sbor_array_value_body::<Vec<PreparedSubintentV2>>(
                decoder,
                ValueType::Subintent,
                max_subintents_per_transaction,
            )?;

        Ok(Self {
            subintents: subintents.into(),
            summary,
        })
    }

    fn value_kind() -> ManifestValueKind {
        ManifestValueKind::Array
    }
}
