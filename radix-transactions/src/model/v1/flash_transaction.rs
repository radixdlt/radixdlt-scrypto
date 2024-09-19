use super::*;
use crate::internal_prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
pub struct FlashTransactionV1 {
    pub name: String,
    pub state_updates: StateUpdates,
}

define_raw_transaction_payload!(RawFlashTransaction, TransactionPayloadKind::Other);

pub struct PreparedFlashTransactionV1 {
    pub name: String,
    pub state_updates: StateUpdates,
    pub summary: Summary,
}

impl TransactionPayload for FlashTransactionV1 {
    type Prepared = PreparedFlashTransactionV1;
    type Raw = RawFlashTransaction;
}

impl_has_summary!(PreparedFlashTransactionV1);

#[allow(deprecated)]
impl TransactionPreparableFromValue for PreparedFlashTransactionV1 {
    fn prepare_from_value(decoder: &mut TransactionDecoder) -> Result<Self, PrepareError> {
        let ((name, state_updates), summary) = ConcatenatedDigest::prepare_transaction_payload::<(
            SummarizedRawFullValue<String>,
            SummarizedRawFullValue<StateUpdates>,
        )>(
            decoder,
            TransactionDiscriminator::V1Flash,
            ExpectedHeaderKind::TupleWithValueKind,
        )?;
        Ok(Self {
            name: name.inner,
            state_updates: state_updates.inner,
            summary,
        })
    }
}

#[allow(deprecated)]
impl PreparedTransaction for PreparedFlashTransactionV1 {
    type Raw = RawFlashTransaction;

    fn prepare_from_transaction_enum(
        decoder: &mut TransactionDecoder,
    ) -> Result<Self, PrepareError> {
        let ((name, state_updates), summary) = ConcatenatedDigest::prepare_transaction_payload::<(
            SummarizedRawFullValue<String>,
            SummarizedRawFullValue<StateUpdates>,
        )>(
            decoder,
            TransactionDiscriminator::V1Flash,
            ExpectedHeaderKind::EnumWithValueKind,
        )?;
        Ok(Self {
            name: name.inner,
            state_updates: state_updates.inner,
            summary,
        })
    }
}

impl HasFlashTransactionHash for PreparedFlashTransactionV1 {
    fn flash_transaction_hash(&self) -> FlashTransactionHash {
        FlashTransactionHash(self.summary.hash)
    }
}
