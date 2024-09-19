use super::*;
use crate::internal_prelude::*;

//=================================================================================
// NOTE:
// See versioned.rs for tests and a demonstration for the calculation of hashes etc
//=================================================================================

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor, ScryptoDescribe)]
pub struct PartialTransactionV2 {
    pub root_intent: SubintentV2,
    pub subintents: SubintentsV2,
}

define_transaction_payload!(
    PartialTransactionV2,
    RawPartialTransaction,
    PreparedPartialTransactionV2 {
        root_intent: PreparedSubintentV2,
        subintents: PreparedSubintentsV2,
    },
    TransactionDiscriminator::V2PartialTransaction,
);

impl HasSubintentHash for PreparedPartialTransactionV2 {
    fn subintent_hash(&self) -> SubintentHash {
        self.root_intent.subintent_hash()
    }
}
