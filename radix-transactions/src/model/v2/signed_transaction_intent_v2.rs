use super::*;
use crate::internal_prelude::*;

//=================================================================================
// NOTE:
// See versioned.rs for tests and a demonstration for the calculation of hashes etc
//=================================================================================

/// A [`SignedTransactionIntentV2`] is an inner model for a [`NotarizedTransactionV2`].
///
/// It includes two parts:
/// * The [`TransactionIntentV2`] which contains a representention of an intent tree
/// with a root transaction intent, and other subintent descendents. These subintents are
/// flattened into an array in the model.
/// * It also includes intent signatures, some for this transaction intent, and separately,
/// an array of signatures for each each flattened subintents.
///
/// ## Similar models
///
/// A [`SignedPartialTransactionV2`] is a similar structure for a fully signed partial subtree
/// of a transaction, but with a subintent root. Whilst useful for constructing a
/// transaction, it doesn't appear under a [`NotarizedTransactionV2`] because the subintents
/// get flattened.
#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor, ScryptoDescribe)]
pub struct SignedTransactionIntentV2 {
    pub transaction_intent: TransactionIntentV2,
    pub transaction_intent_signatures: IntentSignaturesV2,
    pub non_root_subintent_signatures: NonRootSubintentSignaturesV2,
}

define_transaction_payload!(
    SignedTransactionIntentV2,
    RawSignedTransactionIntent,
    PreparedSignedTransactionIntentV2 {
        transaction_intent: PreparedTransactionIntentV2,
        transaction_intent_signatures: PreparedIntentSignaturesV2,
        non_root_subintent_signatures: PreparedNonRootSubintentSignaturesV2,
    },
    TransactionDiscriminator::V2SignedTransactionIntent,
);

impl HasTransactionIntentHash for PreparedSignedTransactionIntentV2 {
    fn transaction_intent_hash(&self) -> TransactionIntentHash {
        self.transaction_intent.transaction_intent_hash()
    }
}

impl HasSignedTransactionIntentHash for PreparedSignedTransactionIntentV2 {
    fn signed_transaction_intent_hash(&self) -> SignedTransactionIntentHash {
        SignedTransactionIntentHash::from_hash(self.summary.hash)
    }
}

impl HasNonRootSubintentHashes for PreparedSignedTransactionIntentV2 {
    fn non_root_subintent_hashes(&self) -> Vec<SubintentHash> {
        self.transaction_intent.non_root_subintent_hashes()
    }
}
