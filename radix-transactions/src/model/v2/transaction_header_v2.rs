use crate::internal_prelude::*;

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor, ScryptoDescribe)]
pub struct TransactionHeaderV2 {
    pub notary_public_key: PublicKey,
    pub notary_is_signatory: bool,
    pub tip_basis_points: u32,
}

impl TransactionPartialPrepare for TransactionHeaderV2 {
    type Prepared = PreparedTransactionHeaderV2;
}

pub type PreparedTransactionHeaderV2 = SummarizedRawValueBody<TransactionHeaderV2>;
