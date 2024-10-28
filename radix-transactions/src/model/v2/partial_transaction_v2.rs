use super::*;
use crate::internal_prelude::*;

//=================================================================================
// NOTE:
// See versioned.rs for tests and a demonstration for the calculation of hashes etc
//=================================================================================

/// An analogue of a [`TransactionIntentV2`], except with a subintent at the root.
///
/// This is intended to represent an incomplete sub-tree of the transaction, and be
/// a canonical model for building, storing and transferring this subtree.
///
/// The corresponding signed model is a [`SignedPartialTransactionV2`].
#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor, ScryptoDescribe)]
pub struct PartialTransactionV2 {
    pub root_subintent: SubintentV2,
    pub non_root_subintents: NonRootSubintentsV2,
}

define_transaction_payload!(
    PartialTransactionV2,
    RawPartialTransaction,
    PreparedPartialTransactionV2 {
        root_subintent: PreparedSubintentV2,
        non_root_subintents: PreparedNonRootSubintentsV2,
    },
    TransactionDiscriminator::V2PartialTransaction,
);

impl PreparedPartialTransactionV2 {
    pub fn non_root_subintent_hashes(&self) -> impl Iterator<Item = SubintentHash> + '_ {
        self.non_root_subintents
            .subintents
            .iter()
            .map(|s| s.subintent_hash())
    }
}

impl HasSubintentHash for PreparedPartialTransactionV2 {
    fn subintent_hash(&self) -> SubintentHash {
        self.root_subintent.subintent_hash()
    }
}
