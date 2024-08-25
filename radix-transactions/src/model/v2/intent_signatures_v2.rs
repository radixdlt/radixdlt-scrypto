use super::*;
use crate::internal_prelude::*;

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor, ScryptoDescribe)]
#[sbor(transparent)]
pub struct IntentSignaturesV2 {
    pub signatures: Vec<IntentSignatureV1>,
}

pub type PreparedIntentSignaturesV2 = SummarizedRawValueBody<IntentSignaturesV2>;

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor, ScryptoDescribe)]
#[sbor(transparent)]
pub struct MultipleIntentSignaturesV2 {
    pub by_subintent: Vec<IntentSignaturesV2>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PreparedMultipleIntentSignaturesV2 {
    pub by_subintent: Vec<PreparedIntentSignaturesV2>,
    pub summary: Summary,
}

impl HasSummary for PreparedMultipleIntentSignaturesV2 {
    fn get_summary(&self) -> &Summary {
        &self.summary
    }
}

impl TransactionPreparableFromValue for PreparedMultipleIntentSignaturesV2 {
    fn prepare_from_value(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError> {
        let (by_subintent, summary) = ConcatenatedDigest::prepare_from_sbor_array::<
            Vec<PreparedIntentSignaturesV2>,
            V2_MAX_NUMBER_OF_SUBINTENTS_IN_TRANSACTION,
        >(decoder, ValueType::IntentSignatures)?;

        Ok(Self {
            by_subintent,
            summary,
        })
    }
}
