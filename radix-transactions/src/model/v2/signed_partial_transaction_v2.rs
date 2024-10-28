use super::*;
use crate::internal_prelude::*;

//=================================================================================
// NOTE:
// See versioned.rs for tests and a demonstration for the calculation of hashes etc
//=================================================================================

/// An analogue of a [`SignedTransactionIntentV2`], except with a [`SubintentV2`] at the root.
///
/// This is intended to represent a fully signed, incomplete sub-tree of the transaction,
/// and be a canonical model for building, storing and transferring this signed subtree.
///
/// It contains an unsigned [`PartialTransactionV2`], and signatures for the root subintent,
/// and each non-root subintent.
///
/// It can be prepared and validated using [`self.prepare_and_validate(validator)`][Self::prepare_and_validate],
/// just like a [`NotarizedTransactionV2`]. Its validated form is a [`ValidatedSignedPartialTransactionV2`].
///
/// It can be built with a [`PartialTransactionV2Builder`].
#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor, ScryptoDescribe)]
pub struct SignedPartialTransactionV2 {
    pub partial_transaction: PartialTransactionV2,
    pub root_subintent_signatures: IntentSignaturesV2,
    pub non_root_subintent_signatures: NonRootSubintentSignaturesV2,
}

impl SignedPartialTransactionV2 {
    pub fn prepare_and_validate(
        &self,
        validator: &TransactionValidator,
    ) -> Result<ValidatedSignedPartialTransactionV2, TransactionValidationError> {
        self.prepare(validator.preparation_settings())?
            .validate(validator)
    }
}

define_transaction_payload!(
    SignedPartialTransactionV2,
    RawSignedPartialTransaction,
    PreparedSignedPartialTransactionV2 {
        partial_transaction: PreparedPartialTransactionV2,
        root_subintent_signatures: PreparedIntentSignaturesV2,
        non_root_subintent_signatures: PreparedNonRootSubintentSignaturesV2,
    },
    TransactionDiscriminator::V2SignedPartialTransaction,
);

impl HasSubintentHash for PreparedSignedPartialTransactionV2 {
    fn subintent_hash(&self) -> SubintentHash {
        self.partial_transaction.subintent_hash()
    }
}

impl PreparedSignedPartialTransactionV2 {
    pub fn non_root_subintent_hashes(&self) -> impl Iterator<Item = SubintentHash> + '_ {
        self.partial_transaction.non_root_subintent_hashes()
    }

    pub fn validate(
        self,
        validator: &TransactionValidator,
    ) -> Result<ValidatedSignedPartialTransactionV2, TransactionValidationError> {
        validator.validate_signed_partial_transaction_v2(self)
    }
}
