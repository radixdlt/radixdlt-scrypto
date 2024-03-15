use radix_engine_common::types::Epoch;
use radix_engine_common::{crypto::PublicKey, ManifestSbor};

use crate::model::SummarizedRawFullBody;

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor)]
pub struct TransactionHeaderV1 {
    pub network_id: u8,
    pub start_epoch_inclusive: Epoch,
    pub end_epoch_exclusive: Epoch,
    pub nonce: u32,
    pub notary_public_key: PublicKey,
    pub notary_is_signatory: bool,
    pub tip_percentage: u16,
}

pub type PreparedTransactionHeaderV1 = SummarizedRawFullBody<TransactionHeaderV1>;
