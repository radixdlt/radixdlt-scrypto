use super::*;
use crate::internal_prelude::*;

//=================================================================================
// NOTE:
// See versioned.rs for tests and a demonstration for the calculation of hashes etc
//=================================================================================

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor, ScryptoDescribe)]
pub struct SubintentV2 {
    pub intent_core: IntentCoreV2,
}

define_transaction_payload!(
    SubintentV2,
    RawSubintent,
    PreparedSubintentV2 {
        intent_core: PreparedIntentCoreV2,
    },
    TransactionDiscriminator::V2Subintent,
);

impl HasSubintentHash for PreparedSubintentV2 {
    fn subintent_hash(&self) -> SubintentHash {
        SubintentHash::from_hash(self.summary.hash)
    }
}
