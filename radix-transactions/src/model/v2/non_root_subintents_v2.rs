use super::*;
use crate::internal_prelude::*;

//=================================================================================
// NOTE:
// See versioned.rs for tests and a demonstration for the calculation of hashes etc
//=================================================================================

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor, ScryptoDescribe)]
#[sbor(transparent)]
pub struct NonRootSubintentsV2(pub Vec<SubintentV2>);

impl TransactionPartialPrepare for NonRootSubintentsV2 {
    type Prepared = PreparedNonRootSubintentsV2;
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PreparedNonRootSubintentsV2 {
    pub subintents: Vec<PreparedSubintentV2>,
    pub summary: Summary,
}

impl_has_summary!(PreparedNonRootSubintentsV2);

impl HasNonRootSubintentHashes for PreparedNonRootSubintentsV2 {
    fn non_root_subintent_hashes(&self) -> Vec<SubintentHash> {
        // This can be shorter than `self.subintents` if the transaction is invalid,
        // but this is OK as per the definition on the trait.
        self.subintents.iter().map(|s| s.subintent_hash()).collect()
    }
}

impl TransactionPreparableFromValueBody for PreparedNonRootSubintentsV2 {
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
