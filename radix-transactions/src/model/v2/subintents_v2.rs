use super::*;
use crate::internal_prelude::*;

//=================================================================================
// NOTE:
// See versioned.rs for tests and a demonstration for the calculation of hashes etc
//=================================================================================

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor, ScryptoDescribe)]
#[sbor(transparent)]
pub struct SubintentsV2(Vec<SubintentV2>);

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PreparedSubintentsV2 {
    subintents_by_hash: Rc<IndexMap<SubintentHash, PreparedSubintentV2>>,
    summary: Summary,
}

impl HasSummary for PreparedSubintentsV2 {
    fn get_summary(&self) -> &Summary {
        &self.summary
    }
}

impl TransactionPreparableFromValue for PreparedSubintentsV2 {
    fn prepare_from_value(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError> {
        let (subintents, summary) = ConcatenatedDigest::prepare_from_sbor_array::<
            Vec<PreparedSubintentV2>,
            V2_MAX_NUMBER_OF_SUBINTENTS_IN_TRANSACTION,
        >(decoder, ValueType::Subintent)?;

        let mut subintents_by_hash = index_map_with_capacity(subintents.len());
        for subintent in subintents {
            subintents_by_hash.insert(subintent.subintent_hash(), subintent);
        }

        Ok(Self {
            subintents_by_hash: Rc::new(subintents_by_hash),
            summary,
        })
    }
}
