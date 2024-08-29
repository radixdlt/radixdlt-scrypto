use crate::internal_prelude::*;
use radix_common::types::Epoch;
use radix_common::{crypto::PublicKey, ManifestSbor};

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor, ScryptoDescribe)]
pub struct TransactionHeaderV1 {
    pub network_id: u8,
    pub start_epoch_inclusive: Epoch,
    pub end_epoch_exclusive: Epoch,
    pub nonce: u32,
    pub notary_public_key: PublicKey,
    pub notary_is_signatory: bool,
    pub tip_percentage: u16,
}

#[allow(deprecated)]
pub type PreparedTransactionHeaderV1 = SummarizedRawFullValue<TransactionHeaderV1>;
