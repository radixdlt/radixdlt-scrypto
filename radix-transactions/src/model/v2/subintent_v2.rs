use super::*;
use crate::internal_prelude::*;

//=================================================================================
// NOTE:
// See versioned.rs for tests and a demonstration for the calculation of hashes etc
//=================================================================================

/// A Subintent is a distinct concept to a [`TransactionIntentV2`].
///
/// * A subintent has to have a parent in a transaction.
/// * A subintent is only "committed" on failure.
/// * A subintent can't pay fees.
///
/// If you are looking to construct a subintent, use a [`PartialTransactionV2Builder`],
/// which builds a [`SignedPartialTransactionV2`].
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
