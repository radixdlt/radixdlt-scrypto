use super::*;
use crate::internal_prelude::*;

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor, ScryptoDescribe)]
#[sbor(transparent)]
pub struct IntentSignaturesV2 {
    pub signatures: Vec<IntentSignatureV1>,
}

impl IntentSignaturesV2 {
    pub fn none() -> Self {
        Self {
            signatures: Vec::new(),
        }
    }

    pub fn new(signatures: Vec<IntentSignatureV1>) -> Self {
        Self { signatures }
    }
}

impl TransactionPartialPrepare for IntentSignaturesV2 {
    type Prepared = PreparedIntentSignaturesV2;
}

pub type PreparedIntentSignaturesV2 = SummarizedRawValueBody<IntentSignaturesV2>;

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor, ScryptoDescribe)]
#[sbor(transparent)]
pub struct NonRootSubintentSignaturesV2 {
    pub by_subintent: Vec<IntentSignaturesV2>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PreparedNonRootSubintentSignaturesV2 {
    pub by_subintent: Vec<PreparedIntentSignaturesV2>,
    pub summary: Summary,
}

impl_has_summary!(PreparedNonRootSubintentSignaturesV2);

impl TransactionPreparableFromValueBody for PreparedNonRootSubintentSignaturesV2 {
    fn prepare_from_value_body(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError> {
        let max_subintents_per_transaction = decoder.settings().max_subintents_per_transaction;
        let (by_subintent, summary) = ConcatenatedDigest::prepare_from_sbor_array_value_body::<
            Vec<PreparedIntentSignaturesV2>,
        >(
            decoder,
            ValueType::SubintentSignatureBatches,
            max_subintents_per_transaction,
        )?;

        Ok(Self {
            by_subintent,
            summary,
        })
    }

    fn value_kind() -> ManifestValueKind {
        ManifestValueKind::Array
    }
}
