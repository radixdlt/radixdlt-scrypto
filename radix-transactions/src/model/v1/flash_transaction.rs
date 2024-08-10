use super::*;
use crate::internal_prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
pub struct FlashTransactionV1 {
    pub name: String,
    pub state_updates: StateUpdates,
}

pub struct PreparedFlashTransactionV1 {
    pub name: String,
    pub state_updates: StateUpdates,
    pub summary: Summary,
}

impl TransactionPayload for FlashTransactionV1 {
    type Prepared = PreparedFlashTransactionV1;
    type Raw = RawFlashTransactionV1;
}

define_raw_transaction_payload!(RawFlashTransactionV1);

impl HasSummary for PreparedFlashTransactionV1 {
    fn get_summary(&self) -> &Summary {
        &self.summary
    }
}

impl HasFlashTransactionHash for PreparedFlashTransactionV1 {
    fn flash_transaction_hash(&self) -> FlashTransactionHash {
        FlashTransactionHash(self.summary.hash)
    }
}

impl TransactionFullChildPreparable for PreparedFlashTransactionV1 {
    fn prepare_as_full_body_child(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError> {
        let ((name, state_updates), summary) =
            ConcatenatedDigest::prepare_from_transaction_child_struct::<(
                SummarizedRawFullBody<String>,
                SummarizedRawFullBody<StateUpdates>,
            )>(decoder, TransactionDiscriminator::V1Flash)?;
        Ok(Self {
            name: name.inner,
            state_updates: state_updates.inner,
            summary,
        })
    }
}

impl TransactionPayloadPreparable for PreparedFlashTransactionV1 {
    type Raw = RawFlashTransactionV1;

    fn prepare_for_payload(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError> {
        let ((name, state_updates), summary) =
            ConcatenatedDigest::prepare_from_transaction_payload_enum::<(
                SummarizedRawFullBody<String>,
                SummarizedRawFullBody<StateUpdates>,
            )>(decoder, TransactionDiscriminator::V1Flash)?;
        Ok(Self {
            name: name.inner,
            state_updates: state_updates.inner,
            summary,
        })
    }
}
