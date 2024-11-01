use super::*;
use crate::internal_prelude::*;

//=================================================================================
// NOTE:
// See versioned.rs for tests and a demonstration for the calculation of hashes etc
//=================================================================================

/// This is the root intent of a [`NotarizedTransactionV2`].
///
/// It may have 0 or more [`SubintentV2`] children, which themselves may have children...
/// This forms an intent tree, which is subject to a depth limit, and a limit on the
/// total number of intents.
///
/// All the subintent descendents are flattened and are stored in the `non_root_subintents`
/// field. To be valid, these intents must form a tree, with the transaction intent at its root.
/// There are a few reasons for the flattening:
/// * It means the models have a fixed depth, allowing avoiding recursion.
/// * It aligns with the runtime model.
/// * It allows the signatures to be attached in one layer as per the [`SignedTransactionIntentV2`].
///
/// ## Similar models
///
///  A [`PartialTransactionV2`] is a similar structure for a partial subtree
/// of a transaction, but with a subintent root. Whilst useful for constructing a
/// transaction, it doesn't appear under a [`NotarizedTransactionV2`] because the subintents
/// get flattened.
#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor, ScryptoDescribe)]
pub struct TransactionIntentV2 {
    pub transaction_header: TransactionHeaderV2,
    pub root_intent_core: IntentCoreV2,
    pub non_root_subintents: NonRootSubintentsV2,
}

define_transaction_payload!(
    TransactionIntentV2,
    RawTransactionIntent,
    PreparedTransactionIntentV2 {
        transaction_header: PreparedTransactionHeaderV2,
        root_intent_core: PreparedIntentCoreV2,
        non_root_subintents: PreparedNonRootSubintentsV2,
    },
    TransactionDiscriminator::V2TransactionIntent,
);

impl HasTransactionIntentHash for PreparedTransactionIntentV2 {
    fn transaction_intent_hash(&self) -> TransactionIntentHash {
        TransactionIntentHash::from_hash(self.summary.hash)
    }
}

impl HasNonRootSubintentHashes for PreparedTransactionIntentV2 {
    fn non_root_subintent_hashes(&self) -> Vec<SubintentHash> {
        self.non_root_subintents.non_root_subintent_hashes()
    }
}
