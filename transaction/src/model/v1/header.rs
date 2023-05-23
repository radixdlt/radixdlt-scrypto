use radix_engine_common::{crypto::PublicKey, ManifestSbor};

use crate::model::SummarizedRawFullBody;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))] // For toolkit
#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor)]
pub struct TransactionHeaderV1 {
    pub network_id: u8,
    pub start_epoch_inclusive: u32,
    pub end_epoch_exclusive: u32,
    pub nonce: u32,
    pub notary_public_key: PublicKey,
    pub notary_is_signatory: bool,
    pub tip_percentage: u16,
}

pub type PreparedTransactionHeaderV1 = SummarizedRawFullBody<TransactionHeaderV1>;
