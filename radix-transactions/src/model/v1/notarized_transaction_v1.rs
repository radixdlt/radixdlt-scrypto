use super::*;
use crate::internal_prelude::*;

//=================================================================================
// NOTE:
// See versioned.rs for tests and a demonstration for the calculation of hashes etc
//=================================================================================

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor, ScryptoDescribe, ScryptoSborAssertion)]
#[sbor_assert(
    fixed("FILE:notarized_transaction_v1_schema.txt"),
    settings(allow_name_changes)
)]
pub struct NotarizedTransactionV1 {
    pub signed_intent: SignedIntentV1,
    pub notary_signature: NotarySignatureV1,
}

impl NotarizedTransactionV1 {
    pub fn prepare_and_validate(
        &self,
        validator: &TransactionValidator,
    ) -> Result<ValidatedNotarizedTransactionV1, TransactionValidationError> {
        self.prepare(validator.preparation_settings())?
            .validate(validator)
    }
}

impl IntoExecutable for NotarizedTransactionV1 {
    type Error = TransactionValidationError;

    fn into_executable(
        self,
        validator: &TransactionValidator,
    ) -> Result<ExecutableTransaction, Self::Error> {
        let executable = self.prepare_and_validate(validator)?.get_executable();
        Ok(executable)
    }
}

define_transaction_payload!(
    NotarizedTransactionV1,
    RawNotarizedTransaction,
    PreparedNotarizedTransactionV1 {
        signed_intent: PreparedSignedIntentV1,
        notary_signature: PreparedNotarySignatureV1,
    },
    TransactionDiscriminator::V1Notarized,
);

impl PreparedNotarizedTransactionV1 {
    pub fn validate(
        self,
        validator: &TransactionValidator,
    ) -> Result<ValidatedNotarizedTransactionV1, TransactionValidationError> {
        validator.validate_notarized_v1(self)
    }
}

impl HasTransactionIntentHash for PreparedNotarizedTransactionV1 {
    fn transaction_intent_hash(&self) -> TransactionIntentHash {
        self.signed_intent.transaction_intent_hash()
    }
}

impl HasSignedTransactionIntentHash for PreparedNotarizedTransactionV1 {
    fn signed_transaction_intent_hash(&self) -> SignedTransactionIntentHash {
        self.signed_intent.signed_transaction_intent_hash()
    }
}

impl HasNotarizedTransactionHash for PreparedNotarizedTransactionV1 {
    fn notarized_transaction_hash(&self) -> NotarizedTransactionHash {
        NotarizedTransactionHash::from_hash(self.summary.hash)
    }
}
