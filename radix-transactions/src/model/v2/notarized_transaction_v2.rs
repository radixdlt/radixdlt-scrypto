use super::*;
use crate::internal_prelude::*;

//=================================================================================
// NOTE:
// See versioned.rs for tests and a demonstration for the calculation of hashes etc
//=================================================================================

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor, ScryptoDescribe, ScryptoSborAssertion)]
#[sbor_assert(
    fixed("FILE:notarized_transaction_v2_schema.txt"),
    settings(allow_name_changes)
)]
pub struct NotarizedTransactionV2 {
    pub signed_intent: SignedTransactionIntentV2,
    pub notary_signature: NotarySignatureV1,
}

impl NotarizedTransactionV2 {
    pub fn prepare_and_validate(
        &self,
        validator: &TransactionValidator,
    ) -> Result<ValidatedNotarizedTransactionV2, TransactionValidationError> {
        self.prepare(validator.preparation_settings())?
            .validate(validator)
    }
}

define_transaction_payload!(
    NotarizedTransactionV2,
    RawNotarizedTransaction,
    PreparedNotarizedTransactionV2 {
        signed_intent: PreparedSignedTransactionIntentV2,
        notary_signature: PreparedNotarySignatureV1,
    },
    TransactionDiscriminator::V2Notarized,
);

impl PreparedNotarizedTransactionV2 {
    pub fn validate(
        self,
        validator: &TransactionValidator,
    ) -> Result<ValidatedNotarizedTransactionV2, TransactionValidationError> {
        validator.validate_notarized_v2(self)
    }
}

impl IntoExecutable for NotarizedTransactionV2 {
    type Error = TransactionValidationError;

    fn into_executable(
        self,
        validator: &TransactionValidator,
    ) -> Result<ExecutableTransaction, Self::Error> {
        let executable = self.prepare_and_validate(validator)?.get_executable();
        Ok(executable)
    }
}

impl HasTransactionIntentHash for PreparedNotarizedTransactionV2 {
    fn transaction_intent_hash(&self) -> TransactionIntentHash {
        self.signed_intent.transaction_intent_hash()
    }
}

impl HasSignedTransactionIntentHash for PreparedNotarizedTransactionV2 {
    fn signed_transaction_intent_hash(&self) -> SignedTransactionIntentHash {
        self.signed_intent.signed_transaction_intent_hash()
    }
}

impl HasNotarizedTransactionHash for PreparedNotarizedTransactionV2 {
    fn notarized_transaction_hash(&self) -> NotarizedTransactionHash {
        NotarizedTransactionHash::from_hash(self.summary.hash)
    }
}
